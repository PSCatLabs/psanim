//! Rendering many frames, on as many threads as the movie allows.
//!
//! An interpreter cannot cross threads, so each worker loads its own
//! [`Movie`] from the source and renders the frames it is handed. That is
//! sound because a pure movie's frames depend on nothing but their time:
//! any worker gives the same pixels for the same frame. A stateful or
//! persistent movie's frames depend on the ones before, so it renders on
//! one thread, in order.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;

use crate::engine::{Meta, Mode, Movie, Options};
use crate::error::{Error, Phase};

/// A frame handed to a sink: owned pixels.
pub struct Rendered {
    pub index: u64,
    /// Seconds.
    pub time: f64,
    pub width: u32,
    pub height: u32,
    /// Premultiplied RGBA, rows top to bottom.
    pub premultiplied: Vec<u8>,
    /// Interpreter steps the frame took, when a step budget is set.
    pub steps: u64,
}

/// How many threads a render of `meta` on `requested` threads will use
/// (0 asks for one per core): one for a movie whose frames depend on each
/// other, never more than there are frames.
pub fn effective_threads(meta: &Meta, requested: usize, frames: usize) -> usize {
    if meta.mode != Mode::Pure {
        return 1;
    }
    let n = if requested == 0 {
        std::thread::available_parallelism().map_or(1, |n| n.get())
    } else {
        requested
    };
    n.clamp(1, frames.max(1))
}

/// Render the frames `indices` of the movie in `source` and hand each to
/// `sink`, in the order given, on up to `threads` threads (0 for one per
/// core). `sink` returning an error stops the render, as does the
/// interrupt flag in `options`.
///
/// Returns the movie's metadata.
pub fn render_frames<F>(
    source: &str,
    options: &Options,
    indices: &[u64],
    threads: usize,
    mut sink: F,
) -> Result<Meta, Error>
where
    F: FnMut(Rendered) -> Result<(), String>,
{
    // One movie here, on the calling thread: it reports load errors once,
    // and it is the only renderer when the movie cannot be parallel.
    let mut first = Movie::load(source, options)?;
    let meta = first.meta().clone();
    let threads = effective_threads(&meta, threads, indices.len());
    let sink_error = |msg: String| Error::new(Phase::Load, "output", msg);
    if threads <= 1 {
        for &index in indices {
            let frame = first.render_frame(index)?;
            let rendered = Rendered {
                index,
                time: frame.time,
                width: frame.width,
                height: frame.height,
                premultiplied: frame.premultiplied.to_vec(),
                steps: 0,
            };
            let steps = first.last_frame_steps();
            sink(Rendered { steps, ..rendered }).map_err(sink_error)?;
        }
        return Ok(meta);
    }
    drop(first);

    // Workers pull positions from a shared counter and send frames back;
    // this thread puts them in order for the sink. `ahead` bounds how far
    // the workers may run past the frame the sink is waiting for, so a slow
    // frame cannot make the reorder buffer grow without limit.
    let next = Arc::new(AtomicUsize::new(0));
    let delivered = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let ahead = threads * 3;
    let (tx, rx) = mpsc::sync_channel::<(usize, Result<Rendered, Error>)>(threads);
    let result = std::thread::scope(|scope| {
        for _ in 0..threads {
            let (tx, next, delivered, stop) =
                (tx.clone(), next.clone(), delivered.clone(), stop.clone());
            scope.spawn(move || {
                let mut movie = match Movie::load(source, options) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = tx.send((usize::MAX, Err(e)));
                        return;
                    }
                };
                loop {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    let pos = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&index) = indices.get(pos) else {
                        return;
                    };
                    while pos > delivered.load(Ordering::Relaxed) + ahead {
                        if stop.load(Ordering::Relaxed) {
                            return;
                        }
                        std::thread::yield_now();
                        std::thread::sleep(std::time::Duration::from_micros(200));
                    }
                    let out = movie.render_frame(index).map(|f| Rendered {
                        index,
                        time: f.time,
                        width: f.width,
                        height: f.height,
                        premultiplied: f.premultiplied.to_vec(),
                        steps: 0,
                    });
                    let steps = movie.last_frame_steps();
                    if tx
                        .send((pos, out.map(|r| Rendered { steps, ..r })))
                        .is_err()
                    {
                        return;
                    }
                }
            });
        }
        drop(tx);
        let mut waiting: BTreeMap<usize, Rendered> = BTreeMap::new();
        let mut want = 0usize;
        let mut outcome = Ok(());
        for (pos, frame) in rx {
            match frame {
                Err(e) => {
                    outcome = Err(e);
                    break;
                }
                Ok(f) => {
                    waiting.insert(pos, f);
                }
            }
            while let Some(f) = waiting.remove(&want) {
                if let Err(msg) = sink(f) {
                    outcome = Err(sink_error(msg));
                    break;
                }
                want += 1;
                delivered.store(want, Ordering::Relaxed);
            }
            if outcome.is_err() {
                break;
            }
        }
        // Stops the workers, and unblocks any that is waiting to send.
        stop.store(true, Ordering::Relaxed);
        outcome
    });
    result.map(|()| meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "<< /Size [48 32] /Duration 2 /FPS 12 >> animation
        /scene { 24 16 translate now 200 mul rotate 1 setgray -18 -3 36 6 rectfill } def";

    fn collect(threads: usize, indices: &[u64]) -> Vec<(u64, Vec<u8>)> {
        let mut out = Vec::new();
        render_frames(SRC, &Options::default(), indices, threads, |f| {
            out.push((f.index, f.premultiplied));
            Ok(())
        })
        .unwrap();
        out
    }

    #[test]
    fn parallel_render_matches_serial_and_keeps_order() {
        let all: Vec<u64> = (0..24).collect();
        let serial = collect(1, &all);
        let parallel = collect(4, &all);
        assert_eq!(serial.len(), 24);
        assert!(serial.iter().map(|f| f.0).eq(0..24));
        assert_eq!(serial, parallel);
        // An arbitrary selection, in the order asked for.
        let picked = collect(3, &[20, 2, 11, 2]);
        assert_eq!(
            picked.iter().map(|f| f.0).collect::<Vec<_>>(),
            [20, 2, 11, 2]
        );
        assert_eq!(picked[1].1, serial[2].1);
    }

    #[test]
    fn a_frame_error_stops_the_render_and_is_reported() {
        let src = "<< /Size [8 8] /Duration 2 /FPS 10 >> animation /scene { frame 7 eq { nosuchword } if } def";
        let all: Vec<u64> = (0..20).collect();
        for threads in [1, 4] {
            let mut got = Vec::new();
            let e = render_frames(src, &Options::default(), &all, threads, |f| {
                got.push(f.index);
                Ok(())
            })
            .unwrap_err();
            assert_eq!(
                e.phase,
                Phase::Frame {
                    index: 7,
                    time: 0.7
                }
            );
            assert!(got.iter().all(|k| *k < 7), "{got:?}");
        }
    }

    #[test]
    fn a_sink_error_stops_the_render() {
        let all: Vec<u64> = (0..24).collect();
        let mut n = 0;
        let e = render_frames(SRC, &Options::default(), &all, 4, |_| {
            n += 1;
            if n == 3 {
                Err("disk full".into())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(
            (e.kind.as_str(), e.message.as_str()),
            ("output", "disk full")
        );
        assert_eq!(n, 3);
    }

    #[test]
    fn stateful_movies_render_on_one_thread() {
        let src = "<< /Size [8 8] /Duration 1 /FPS 10 >> animation /x 0 def
                   /step { /x x 1 add def } def /scene { x 10 div setgray 0 0 8 8 rectfill } def";
        let movie = Movie::load(src, &Options::default()).unwrap();
        assert_eq!(effective_threads(movie.meta(), 8, 10), 1);
        let mut shades = Vec::new();
        render_frames(
            src,
            &Options::default(),
            &(0..10).collect::<Vec<_>>(),
            8,
            |f| {
                shades.push(f.premultiplied[0]);
                Ok(())
            },
        )
        .unwrap();
        assert!(shades.windows(2).all(|w| w[0] < w[1]), "{shades:?}");
    }
}
