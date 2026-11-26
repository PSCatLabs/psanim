//! Looking for pops: what `psanim check` does with adjacent frames.
//!
//! A pop is a spike in time. Frame pairs are compared tile by tile, and a
//! pair is flagged where a tile's change is far above the same tile's change
//! in the pairs either side of it. Steady fast motion is not a spike, and a
//! small element appearing in a busy frame is, because the comparison is
//! against the tile's own history, not the frame's mean.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::{Duration, Instant};

use psanim::{Movie, raster};

use crate::Fail;

/// Consecutive frames rendered for one chunk of the sweep.
const CHUNK: u64 = 24;
/// Frames kept: a sample's six-frame window overlaps its neighbour's.
const KEEP: usize = 8;
/// The most tiles a change can cover and still be a small element (a
/// letter, a digit, a light) rather than a cut.
const SMALL: usize = 8;

/// A frame rendered and kept for the pairs it takes part in.
pub struct Cached {
    pub index: u64,
    pub time: f64,
    /// Milliseconds to render it (one tick's worth, for a sequential movie).
    pub millis: f64,
    pub flat: Vec<u8>,
}

/// The change between a frame and the next.
pub struct Pair {
    /// Mean difference over the whole frame (`raster::difference`).
    pub motion: f64,
    tiles: Vec<f32>,
}

/// A pair of frames where something pops.
#[derive(Clone)]
pub struct Pop {
    /// The pair is (`frame`, `frame + 1`).
    pub frame: u64,
    /// Bounding box of the spiking tiles in canvas units, y up:
    /// `[x0, y0, x1, y1]`.
    pub area: [f64; 4],
    pub tiles: usize,
    pub total_tiles: usize,
    /// The spiking tiles as (column, row).
    cells: Vec<(usize, usize)>,
    /// The largest spiking tile's change, and its change either side.
    pub change: f32,
    pub around: f32,
}

pub struct Scanner {
    matte: [u8; 3],
    w: usize,
    h: usize,
    tile: usize,
    cols: usize,
    rows: usize,
    /// Canvas units per pixel.
    unit: f64,
    canvas_h: f64,
    sequential: bool,
    last: u64,
    frames: VecDeque<Cached>,
    pairs: BTreeMap<u64, Pair>,
    judged: BTreeSet<u64>,
    spikes: BTreeMap<u64, Pop>,
}

/// What a scan found, after the judgements that need neighbours in time.
pub struct Findings {
    /// Pairs judged with both neighbours judged too: what the scan covers.
    pub covered: usize,
    /// Isolated pops: one pair, one place.
    pub pops: Vec<Pop>,
    /// Places that change in steps again and again (a counter, a readout,
    /// typing), one entry per place: bounding box in canvas units, and the
    /// frames after which it changed.
    pub repeated: Vec<([f64; 4], Vec<u64>)>,
}

impl Scanner {
    /// A scanner for frames up to index `last`.
    pub fn new(movie: &Movie, matte: [u8; 3], last: u64) -> Scanner {
        let meta = movie.meta();
        let (w, h) = (meta.pixel_width as usize, meta.pixel_height as usize);
        // About twenty tiles across, never finer than 8 pixels.
        let tile = (w / 20).max(8);
        let (cols, rows) = raster::tile_grid(w, h, tile);
        Scanner {
            matte,
            w,
            h,
            tile,
            cols,
            rows,
            unit: f64::from(meta.width) / w.max(1) as f64,
            canvas_h: f64::from(meta.height),
            sequential: meta.mode != psanim::Mode::Pure,
            last,
            frames: VecDeque::new(),
            pairs: BTreeMap::new(),
            judged: BTreeSet::new(),
            spikes: BTreeMap::new(),
        }
    }

    pub fn get(&self, index: u64) -> Option<&Cached> {
        self.frames.iter().find(|f| f.index == index)
    }

    /// The whole-frame change from `index` to the next frame, if both have
    /// been rendered.
    pub fn motion(&self, index: u64) -> Option<f64> {
        self.pairs.get(&index).map(|p| p.motion)
    }

    /// Render frame `index` unless it is already kept, and measure the pair
    /// it completes.
    pub fn advance(&mut self, movie: &mut Movie, index: u64) -> Result<(), Fail> {
        if self.get(index).is_some() {
            return Ok(());
        }
        let started = Instant::now();
        let mut flat = if self.frames.len() >= KEEP {
            self.frames.pop_front().map(|f| f.flat).unwrap_or_default()
        } else {
            Vec::new()
        };
        let frame = movie.render_frame(index)?;
        let time = frame.time;
        raster::flatten_onto(frame.premultiplied, self.matte, &mut flat);
        let mut millis = started.elapsed().as_secs_f64() * 1e3;
        if self.sequential {
            millis /= movie.last_frame_ticks().max(1) as f64;
        }
        if index > 0
            && let Some(prev) = self.get(index - 1)
        {
            let pair = Pair {
                motion: raster::difference(&prev.flat, &flat),
                tiles: raster::tile_differences(&prev.flat, &flat, self.w, self.h, self.tile),
            };
            self.pairs.insert(index - 1, pair);
        }
        // Keep the pairs near this stretch; a jump to another one drops
        // the old.
        self.pairs.retain(|k, _| k.abs_diff(index) <= 8);
        self.frames.push_back(Cached {
            index,
            time,
            millis,
            flat,
        });
        Ok(())
    }

    /// Judge the pair (`p`, `p + 1`) against the pairs either side, if all
    /// three have been measured. Each pair is judged once.
    pub fn judge(&mut self, p: u64) {
        let (Some(before), Some(at), Some(after)) = (
            p.checked_sub(1).and_then(|q| self.pairs.get(&q)),
            self.pairs.get(&p),
            self.pairs.get(&(p + 1)),
        ) else {
            return;
        };
        if !self.judged.insert(p) {
            return;
        }
        let spikes = raster::pop_tiles(&before.tiles, &at.tiles, &after.tiles);
        if spikes.is_empty() {
            return;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        let (mut change, mut around) = (0f32, 0f32);
        for &t in &spikes {
            let (cx, cy) = (t % self.cols, t / self.cols);
            x0 = x0.min(cx * self.tile);
            y0 = y0.min(cy * self.tile);
            x1 = x1.max(((cx + 1) * self.tile).min(self.w));
            y1 = y1.max(((cy + 1) * self.tile).min(self.h));
            if at.tiles[t] > change {
                change = at.tiles[t];
                around = before.tiles[t].max(after.tiles[t]);
            }
        }
        let u = self.unit;
        let cells = spikes
            .iter()
            .map(|t| (t % self.cols, t / self.cols))
            .collect();
        self.spikes.insert(
            p,
            Pop {
                frame: p,
                // Pixels run down, canvas units run up.
                area: [
                    x0 as f64 * u,
                    (self.canvas_h - y1 as f64 * u).max(0.0) + 0.0,
                    x1 as f64 * u,
                    self.canvas_h - y0 as f64 * u,
                ],
                tiles: spikes.len(),
                total_tiles: self.cols * self.rows,
                cells,
                change,
                around,
            },
        );
    }

    /// Sort what was judged into isolated pops and repeated changes.
    ///
    /// A pair is only decided when the pairs either side of it were judged
    /// too. A spike with spikes next to it in the pairs before or after is
    /// motion (a wipe, a fast sweep), not a pop: an edge crossing the frame
    /// changes each tile for one pair only, tile after tile. And a place
    /// that spikes more than once is a readout or typing, not an accident.
    pub fn findings(&self) -> Findings {
        let covered = |p: u64| {
            p.checked_sub(1).is_some_and(|q| self.judged.contains(&q))
                && self.judged.contains(&p)
                && self.judged.contains(&(p + 1))
        };
        let near = |a: &Pop, other: Option<&Pop>| {
            other.is_some_and(|b| {
                a.cells.iter().any(|(ax, ay)| {
                    b.cells
                        .iter()
                        .any(|(bx, by)| ax.abs_diff(*bx) <= 3 && ay.abs_diff(*by) <= 3)
                })
            })
        };
        let alone: Vec<&Pop> = self
            .spikes
            .values()
            .filter(|p| covered(p.frame))
            .filter(|p| {
                !near(p, p.frame.checked_sub(1).and_then(|q| self.spikes.get(&q)))
                    && !near(p, self.spikes.get(&(p.frame + 1)))
            })
            .collect();
        // Places that recur: connected groups of small changes whose areas
        // touch, within a tile of each other (a counter, typed letters, a
        // row of lights coming on one by one). Changes that fill much of the
        // frame are cuts and stay as they are.
        let m = self.tile as f64 * self.unit;
        let small = |p: &Pop| p.tiles <= SMALL;
        let near_area = |a: &Pop, b: &Pop| {
            a.area[0] < b.area[2] + m
                && b.area[0] < a.area[2] + m
                && a.area[1] < b.area[3] + m
                && b.area[1] < a.area[3] + m
        };
        let mut group: Vec<usize> = (0..alone.len()).collect();
        for i in 0..alone.len() {
            for j in i + 1..alone.len() {
                if small(alone[i]) && small(alone[j]) && near_area(alone[i], alone[j]) {
                    let (gi, gj) = (group[i], group[j]);
                    for g in group.iter_mut().filter(|g| **g == gj) {
                        *g = gi;
                    }
                }
            }
        }
        let mut pops = Vec::new();
        let mut repeated: Vec<([f64; 4], Vec<u64>)> = Vec::new();
        for (i, pop) in alone.iter().enumerate() {
            let members: Vec<usize> = (0..alone.len()).filter(|j| group[*j] == group[i]).collect();
            if members.len() < 2 {
                pops.push((*pop).clone());
            } else if members[0] == i {
                let mut area = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
                for &j in &members {
                    area[0] = area[0].min(alone[j].area[0]);
                    area[1] = area[1].min(alone[j].area[1]);
                    area[2] = area[2].max(alone[j].area[2]);
                    area[3] = area[3].max(alone[j].area[3]);
                }
                repeated.push((area, members.iter().map(|j| alone[*j].frame).collect()));
            }
        }
        Findings {
            // Over the judged pairs only: a short range late in a very long
            // movie must not walk every frame before it.
            covered: self
                .judged
                .iter()
                .filter(|p| **p <= self.last && covered(**p))
                .count(),
            pops,
            repeated,
        }
    }

    /// Render frames `lo..=hi` in order and judge every pair they allow:
    /// pairs `lo + 1` to `hi - 2`, of which `lo + 2` to `hi - 3` can be
    /// decided (they have judged neighbours).
    pub fn window(
        &mut self,
        movie: &mut Movie,
        lo: u64,
        hi: u64,
        deadline: Option<Instant>,
    ) -> Result<(), Fail> {
        for i in lo..=hi.min(self.last) {
            if deadline.is_some_and(|d| Instant::now() >= d) {
                break;
            }
            // A movie with history cannot go back a frame without replaying
            // from a checkpoint: what is no longer kept is left out.
            if self.sequential
                && self.get(i).is_none()
                && self.frames.back().is_some_and(|f| i < f.index)
            {
                continue;
            }
            self.advance(movie, i)?;
            if i >= lo + 3 {
                self.judge(i - 2);
            }
        }
        Ok(())
    }

    /// Scan consecutive stretches of `first..=self.last`, `max_chunks` of
    /// them, spread over the movie in an order that covers it evenly at
    /// every point (the start, the middle, the quarters...). The set is a
    /// function of the movie alone, so the same movie gives the same
    /// findings on every machine; `ceiling` is only a safety against
    /// frames so slow that the set would take minutes. Returns whether the
    /// ceiling cut the scan short (when its findings depend on the speed of
    /// the machine). Only for a movie whose frames can be rendered in any
    /// order.
    pub fn sweep(
        &mut self,
        movie: &mut Movie,
        first: u64,
        max_chunks: u64,
        ceiling: Duration,
    ) -> Result<bool, Fail> {
        if self.sequential || self.last < first + 3 {
            return Ok(false);
        }
        let deadline = Instant::now() + ceiling;
        let chunks = (self.last - first).div_ceil(CHUNK);
        let m = chunks.next_power_of_two();
        let bits = m.trailing_zeros();
        let mut done = 0;
        for i in 0..m {
            let k = if bits == 0 {
                0
            } else {
                i.reverse_bits() >> (64 - bits)
            };
            if k >= chunks {
                continue;
            }
            if done >= max_chunks {
                break;
            }
            if Instant::now() >= deadline {
                return Ok(true);
            }
            let s = first + k * CHUNK;
            // A few frames past the stretch, so its last pairs have neighbours.
            self.window(movie, s, s + CHUNK + 4, Some(deadline))?;
            done += 1;
        }
        Ok(Instant::now() >= deadline)
    }
}
