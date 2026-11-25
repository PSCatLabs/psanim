//! `psanim`: preview, check and render PostScript animations.

mod args;
mod encode;
mod scan;

use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use args::{Common, number_list};
use encode::{Encoder, Format};
use psanim::render::{Rendered, effective_threads, render_frames};
use psanim::sheet::{self, Still};
use psanim::{Error, Meta, Movie, Scale, raster};
use serde_json::json;

// Counts heap bytes so `--max-memory-mb` and `--untrusted` can cap them.
#[global_allocator]
static ALLOC: pscat::TrackingAllocator = pscat::TrackingAllocator;

const HELP: &str = "\
psanim: programmable animation in PostScript

USAGE
  psanim <command> <program.ps> [options]

COMMANDS
  info     Show what the program declares: size, duration, frames, mode, parameters.
  check    Load the program and render a sample of frames small. Reports errors,
           blank frames, a movie that never moves, abrupt jumps, and the cost of
           the heaviest frame. The first thing to run on a new or edited program.
  frame    Render one frame to a PNG:  psanim frame a.ps --at 2.5 -o still.png
  sheet    Render several frames onto one labelled contact sheet PNG.
  render   Render the movie to .mp4, .webm, .gif, or a directory of PNGs.
  bench    Time loading and rendering, per frame and on all cores.
  api      Print the animation vocabulary. `psanim api tween` shows one word.
  help     This text.

SELECTING FRAMES
  --at T[,T...]       times in seconds          (frame, sheet)
  --frame K[,K...]    frame indices             (frame, sheet)
  --count N           N frames spread evenly    (sheet; default 12)
  --from T --to T     a time range              (sheet, render, check)
  --seam N            the last N frames then the first N: a loop's join   (sheet)

SIZE AND TIMING
  --scale F           pixels per canvas unit (0.5 for a half-size preview)
  --width PX          scale to this output width (or --height PX)
  --fps N             override the declared frame rate
  --duration S        override the declared duration
  --seed N            override the declared seed
  --motion-blur N     average N moments per frame (0 turns declared blur off)
  --shutter F         fraction of the frame the shutter is open (default 0.5)

INPUTS
  --param NAME=VALUE  a value for the program's `param` (repeatable)
  --signals FILE.csv  signals and events for `signal` / `sinceevent` (repeatable)

OUTPUT
  -o, --output PATH   where to write
  --json              machine-readable result on stdout; errors as JSON too
  -q, --quiet         no progress on stderr
  -j, --threads N     render threads (default: one per core)
  --crf N             video quality, lower is better (default 16 for mp4, 28 for webm)
  --cols N            contact sheet columns (default 4)
  --thumb PX          contact sheet thumbnail width (default 320)

SAFETY
  --untrusted         for programs you did not write: bounded steps, time and
                      memory, and no file reads outside the built-in libraries
  --max-steps N       cap interpreter steps for setup and for each frame
  --max-seconds S     cap wall-clock time for setup and for each frame
  --max-memory-mb N   cap heap growth
  --read-root DIR     let the program read files under DIR (default: its own directory)

A program never writes files, whatever the options.

EXIT STATUS
  0 success   1 the program has an error   2 bad usage   3 output failed
";

/// A failure with its exit status and, for program errors, the full error.
enum Fail {
    Usage(String),
    Program(Box<Error>),
    Output(String),
}

impl From<Error> for Fail {
    fn from(e: Error) -> Fail {
        if e.kind == "output" {
            Fail::Output(e.message)
        } else {
            Fail::Program(Box::new(e))
        }
    }
}

fn usage(msg: impl Into<String>) -> Fail {
    Fail::Usage(msg.into())
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let json_errors = argv.iter().any(|a| a == "--json");
    let Some(command) = argv.first() else {
        print!("{HELP}");
        return ExitCode::from(2);
    };
    let rest = &argv[1..];
    let result = match command.as_str() {
        "help" | "--help" | "-h" => {
            print!("{HELP}");
            Ok(())
        }
        "--version" | "version" => {
            println!("psanim {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "info" => cmd_info(rest),
        "check" => cmd_check(rest),
        "frame" => cmd_frame(rest),
        "sheet" => cmd_sheet(rest),
        "render" => cmd_render(rest),
        "bench" => cmd_bench(rest),
        "api" => cmd_api(rest),
        other => Err(usage(format!(
            "unknown command '{other}' (see `psanim help`)"
        ))),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(fail) => {
            let (code, body) = match &fail {
                Fail::Usage(m) => (
                    2,
                    json!({"ok": false, "error": {"phase": "usage", "kind": "usage", "message": m}}),
                ),
                Fail::Program(e) => (1, json!({"ok": false, "error": e.to_json()})),
                Fail::Output(m) => (
                    3,
                    json!({"ok": false, "error": {"phase": "output", "kind": "output", "message": m}}),
                ),
            };
            if json_errors {
                println!("{body}");
            } else {
                match &fail {
                    Fail::Usage(m) => eprintln!("psanim: {m}"),
                    Fail::Program(e) => eprintln!("psanim: {e}"),
                    Fail::Output(m) => eprintln!("psanim: {m}"),
                }
            }
            ExitCode::from(code)
        }
    }
}

fn parse(rest: &[String], known: &[&str]) -> Result<Common, Fail> {
    Common::parse(rest, known).map_err(usage)
}

fn name_of(path: &Path) -> String {
    path.file_stem()
        .map_or_else(|| "movie".to_string(), |s| s.to_string_lossy().into_owned())
}

/// The colour transparent frames are shown over in video and thumbnails.
fn matte(meta: &Meta) -> [u8; 3] {
    meta.background
        .map_or([0, 0, 0], |c| c.map(|v| (v * 255.0).round() as u8))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), Fail> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| Fail::Output(format!("cannot create {}: {e}", parent.display())))?;
    }
    std::fs::write(path, bytes)
        .map_err(|e| Fail::Output(format!("cannot write {}: {e}", path.display())))
}

/// The frames `--at`, `--frame`, `--count`, `--from`/`--to` select.
fn select_frames(c: &Common, meta: &Meta, default_count: usize) -> Result<Vec<u64>, Fail> {
    let last = meta.frames.saturating_sub(1);
    if let Some(list) = c.extra.get("frame") {
        return Ok(number_list(list, "--frame")
            .map_err(usage)?
            .iter()
            .map(|k| (*k as u64).min(last))
            .collect());
    }
    if let Some(n) = c.extra_num::<u64>("seam").map_err(usage)? {
        // The last n frames, then the first n: the join of a loop.
        let n = n.clamp(1, meta.frames / 2);
        return Ok((meta.frames - n..meta.frames).chain(0..n).collect());
    }
    if let Some(list) = c.extra.get("at") {
        return Ok(number_list(list, "--at")
            .map_err(usage)?
            .iter()
            .map(|t| meta.frame_at(*t))
            .collect());
    }
    let (first, end) = range(c, meta)?;
    let count: usize = c
        .extra_num("count")
        .map_err(usage)?
        .unwrap_or(default_count);
    Ok(sheet::spread(end - first, count)
        .into_iter()
        .map(|k| k + first)
        .collect())
}

/// `--from`/`--to` as a half-open frame range.
fn range(c: &Common, meta: &Meta) -> Result<(u64, u64), Fail> {
    let from: f64 = c.extra_num("from").map_err(usage)?.unwrap_or(0.0);
    let to: f64 = c.extra_num("to").map_err(usage)?.unwrap_or(meta.duration);
    let first = ((from * meta.fps - 1e-9).ceil().max(0.0) as u64).min(meta.frames);
    let end = ((to * meta.fps - 1e-9).ceil().max(0.0) as u64).clamp(first, meta.frames);
    if first >= end {
        return Err(usage(format!(
            "--from {from} --to {to} selects no frames of a {}s movie",
            meta.duration
        )));
    }
    Ok((first, end))
}

fn cmd_info(rest: &[String]) -> Result<(), Fail> {
    let c = parse(rest, &[])?;
    let (path, source) = c.source().map_err(usage)?;
    let movie = Movie::load(&source, &c.options(Scale::Factor(1.0)).map_err(usage)?)?;
    let m = movie.meta();
    if c.json {
        println!(
            "{}",
            json!({"ok": true, "file": path, "movie": m.to_json()})
        );
        return Ok(());
    }
    println!("{}", m.title.as_deref().unwrap_or(&name_of(&path)));
    if let Some(d) = &m.description {
        println!("  {d}");
    }
    println!("  canvas      {} x {} units", m.width, m.height);
    println!(
        "  output      {} x {} px (scale {})",
        m.pixel_width, m.pixel_height, m.scale
    );
    println!(
        "  duration    {} s at {} fps = {} frames",
        m.duration, m.fps, m.frames
    );
    println!(
        "  mode        {}{}",
        m.mode.name(),
        match m.mode {
            psanim::Mode::Pure => " (any frame, any order, in parallel)".to_string(),
            _ => format!(" ({} ticks/s, frames in order)", m.tick_rate),
        }
    );
    println!("  seed        {}", m.seed);
    println!(
        "  background  {}",
        m.background.map_or("transparent".to_string(), |c| format!(
            "{} {} {}",
            c[0], c[1], c[2]
        ))
    );
    println!("  loop        {}", m.looping);
    if let Some(mb) = m.motion_blur {
        println!(
            "  motion blur {} samples, shutter {}",
            mb.samples, mb.shutter
        );
    }
    for (k, v) in &m.params {
        println!("  param       {k} (default {v})");
    }
    Ok(())
}

fn cmd_frame(rest: &[String]) -> Result<(), Fail> {
    let c = parse(rest, &["at", "frame"])?;
    let (path, source) = c.source().map_err(usage)?;
    let mut movie = Movie::load(&source, &c.options(Scale::Factor(1.0)).map_err(usage)?)?;
    let meta = movie.meta().clone();
    let frames = select_frames(&c, &meta, 1)?;
    let index = *frames.first().ok_or_else(|| usage("no frame selected"))?;
    let out = c
        .output
        .clone()
        .unwrap_or_else(|| format!("{}_{index:05}.png", name_of(&path)).into());
    let started = Instant::now();
    let rendered = movie.render_frame(index).map(|f| {
        (
            f.to_png(),
            f.premultiplied.to_vec(),
            f.width,
            f.height,
            f.time,
        )
    });
    let (png, premultiplied, w, h, time) = match rendered {
        Ok(r) => r,
        Err(e) => {
            // What it printed before failing is the best clue to why.
            let printed = movie.take_output();
            if !c.json && !printed.is_empty() {
                eprint!("{printed}");
                if !printed.ends_with('\n') {
                    eprintln!();
                }
            }
            return Err(e.into());
        }
    };
    let elapsed = started.elapsed().as_secs_f64();
    let mut flat = Vec::new();
    raster::flatten_onto(&premultiplied, matte(&meta), &mut flat);
    let stats = raster::stats(&flat, w as usize, h as usize);
    write_file(&out, &png)?;
    // What the program printed (`==`, `print`, `pstack`): the debugger.
    let printed = movie.take_output();
    if !c.json && !printed.is_empty() {
        eprint!("{printed}");
        if !printed.ends_with('\n') {
            eprintln!();
        }
    }
    if c.json {
        println!(
            "{}",
            json!({"ok": true, "output": out, "frame": index, "time": time, "width": w, "height": h,
                   "render_seconds": elapsed, "stats": stats_json(&stats), "printed": printed})
        );
    } else if !c.quiet {
        eprintln!(
            "wrote {} (frame {index}, t={time:.3}s, {w}x{h}, {:.1} ms)",
            out.display(),
            elapsed * 1e3
        );
    }
    Ok(())
}

fn stats_json(s: &raster::Stats) -> serde_json::Value {
    let r3 = |v: f64| (v * 1000.0).round() / 1000.0;
    json!({"luma": r3(s.luma), "coverage": r3(s.coverage), "bbox": s.bbox,
           "mean_rgb": [r3(s.mean_rgb[0]), r3(s.mean_rgb[1]), r3(s.mean_rgb[2])]})
}

fn cmd_sheet(rest: &[String]) -> Result<(), Fail> {
    let c = parse(
        rest,
        &[
            "at", "frame", "count", "from", "to", "cols", "thumb", "seam",
        ],
    )?;
    let (path, source) = c.source().map_err(usage)?;
    let thumb: u32 = c.extra_num("thumb").map_err(usage)?.unwrap_or(320);
    // Render at thumbnail size directly unless a size was asked for: a
    // sheet is a preview, and small frames are fast.
    let opts = c.options(Scale::Width(thumb)).map_err(usage)?;
    let probe = Movie::load(&source, &opts)?;
    let meta = probe.meta().clone();
    drop(probe);
    let frames = select_frames(&c, &meta, 12)?;
    let cols: usize = c.extra_num("cols").map_err(usage)?.unwrap_or(4);
    let out = c
        .output
        .clone()
        .unwrap_or_else(|| format!("{}_sheet.png", name_of(&path)).into());
    let started = Instant::now();
    let mut stills = Vec::new();
    render_frames(&source, &opts, &frames, c.threads, |f: Rendered| {
        stills.push(Still {
            index: f.index,
            time: f.time,
            width: f.width,
            height: f.height,
            premultiplied: f.premultiplied,
        });
        Ok(())
    })?;
    let title = sheet::default_title(&meta, &name_of(&path));
    let page =
        sheet::compose(&stills, cols, thumb, Some(&title), matte(&meta)).map_err(Fail::Output)?;
    let png = page.to_png().map_err(|e| Fail::Output(e.to_string()))?;
    write_file(&out, &png)?;
    let elapsed = started.elapsed().as_secs_f64();
    if c.json {
        println!(
            "{}",
            json!({"ok": true, "output": out, "frames": frames, "width": page.width(), "height": page.height(),
                   "seconds": elapsed})
        );
    } else if !c.quiet {
        eprintln!(
            "wrote {} ({} frames, {}x{}, {:.2} s)",
            out.display(),
            frames.len(),
            page.width(),
            page.height(),
            elapsed
        );
    }
    Ok(())
}

fn cmd_render(rest: &[String]) -> Result<(), Fail> {
    let c = parse(rest, &["from", "to", "crf"])?;
    let (path, source) = c.source().map_err(usage)?;
    let opts = c.options(Scale::Factor(1.0)).map_err(usage)?;
    let out = c
        .output
        .clone()
        .unwrap_or_else(|| format!("{}.mp4", name_of(&path)).into());
    let format = Format::from_path(&out).map_err(usage)?;
    let probe = Movie::load(&source, &opts)?;
    let meta = probe.meta().clone();
    drop(probe);
    let (first, end) = range(&c, &meta)?;
    let frames: Vec<u64> = (first..end).collect();
    let threads = effective_threads(&meta, c.threads, frames.len());
    let crf: Option<u32> = c.extra_num("crf").map_err(usage)?;
    let mut encoder = Encoder::start(
        format,
        &out,
        meta.pixel_width,
        meta.pixel_height,
        meta.fps,
        crf,
        meta.looping,
    )
    .map_err(Fail::Output)?;
    let matte = matte(&meta);
    let started = Instant::now();
    let total = frames.len();
    let mut done = 0usize;
    let mut flat = Vec::new();
    let mut last_report = Instant::now();
    // A progress line is for a person at a terminal; in a pipe or a log it
    // is noise, so only the final summary is printed there.
    let show = !c.quiet && !c.json && std::io::IsTerminal::is_terminal(&std::io::stderr());
    render_frames(&source, &opts, &frames, threads, |f: Rendered| {
        raster::flatten_onto(&f.premultiplied, matte, &mut flat);
        encoder.write(&flat, || {
            psanim::Page::from_premultiplied_rgba8(f.width, f.height, f.premultiplied.clone())
                .and_then(|p| p.to_png().ok())
                .unwrap_or_default()
        })?;
        done += 1;
        if show && (last_report.elapsed().as_millis() > 200 || done == total) {
            last_report = Instant::now();
            let secs = started.elapsed().as_secs_f64();
            let rate = done as f64 / secs.max(1e-6);
            eprint!(
                "\r  frame {done}/{total} ({:.0}%)  {rate:.1} fps  eta {:.0}s   ",
                100.0 * done as f64 / total as f64,
                (total - done) as f64 / rate.max(1e-6)
            );
            let _ = std::io::stderr().flush();
        }
        Ok(())
    })
    .inspect_err(|_| {
        if show {
            eprintln!();
        }
    })?;
    let render_secs = started.elapsed().as_secs_f64();
    encoder.finish().map_err(Fail::Output)?;
    let total_secs = started.elapsed().as_secs_f64();
    let bytes = if format == Format::PngSequence {
        0
    } else {
        std::fs::metadata(&out).map_or(0, |m| m.len())
    };
    if c.json {
        println!(
            "{}",
            json!({"ok": true, "output": out, "format": format.name(), "frames": total,
                   "width": meta.pixel_width, "height": meta.pixel_height, "fps": meta.fps,
                   "seconds_of_video": total as f64 / meta.fps, "threads": threads,
                   "render_seconds": render_secs, "total_seconds": total_secs,
                   "frames_per_second": total as f64 / render_secs.max(1e-9), "bytes": bytes})
        );
    } else if !c.quiet {
        if show {
            eprintln!();
        }
        eprintln!(
            "wrote {} ({}, {total} frames, {}x{}, {:.1} s on {threads} thread{}, {:.1} fps{})",
            out.display(),
            format.name(),
            meta.pixel_width,
            meta.pixel_height,
            total_secs,
            if threads == 1 { "" } else { "s" },
            total as f64 / render_secs.max(1e-9),
            if bytes > 0 {
                format!(", {:.1} MB", bytes as f64 / 1e6)
            } else {
                String::new()
            }
        );
    }
    Ok(())
}

/// Stretches of 24 consecutive frames `check` scans for pops beyond its
/// sampled frames, in a pure movie: about 230 frames, whatever the machine.
const SWEEP_CHUNKS: u64 = 8;
/// The longest that scan may take. Only a movie with very slow frames gets
/// near it, and then its findings depend on the machine's speed.
const SWEEP_CEILING: std::time::Duration = std::time::Duration::from_secs(8);

/// One sampled moment of `check`.
struct Sample {
    index: u64,
    time: f64,
    millis: f64,
    stats: raster::Stats,
    /// Difference from the next frame: how much moves here.
    motion: f64,
}

fn cmd_check(rest: &[String]) -> Result<(), Fail> {
    let c = parse(rest, &["count", "from", "to", "all"])?;
    let (path, source) = c.source().map_err(usage)?;
    // Small frames: a check is about whether the movie works, not how it looks.
    let opts = c.options(Scale::Width(320)).map_err(usage)?;
    let load_started = Instant::now();
    let mut movie = Movie::load(&source, &opts)?;
    let load_ms = load_started.elapsed().as_secs_f64() * 1e3;
    let meta = movie.meta().clone();
    let (first, end) = range(&c, &meta)?;
    let count = if c.has("all") {
        (end - first) as usize
    } else {
        c.extra_num("count").map_err(usage)?.unwrap_or(16)
    };
    let picks: Vec<u64> = sheet::spread(end - first, count)
        .into_iter()
        .map(|k| k + first)
        .collect();
    let matte = matte(&meta);
    let (w, h) = (meta.pixel_width as usize, meta.pixel_height as usize);
    let mut samples: Vec<Sample> = Vec::new();
    let mut flat = Vec::new();
    let mut next = Vec::new();
    // A pure movie's strings after setup, to compare with after its frames.
    let strings_before = movie.string_snapshot()?;
    // Each sampled frame is rendered with the frames around it: its motion
    // is the change to the next, and the pairs either side say whether that
    // change is a pop. (A movie whose frames depend on the ones before pays
    // for every frame since the last sample: render_ms divides that out, so
    // the figure is the cost of one frame.)
    let last = end - 1;
    let mut scan = scan::Scanner::new(&movie, matte, last);
    for &index in &picks {
        // A pure movie's pops are looked for by the sweep below; its
        // samples need only the next frame. Another movie is looked at
        // around each sample, which costs little beside reaching it.
        if meta.mode == psanim::Mode::Pure && !c.has("all") {
            scan.window(&mut movie, index, index + 1, None)?;
        } else {
            scan.window(
                &mut movie,
                index.saturating_sub(2).max(first),
                index + 3,
                None,
            )?;
        }
        let Some(f) = scan.get(index) else { continue };
        samples.push(Sample {
            index,
            time: f.time,
            millis: f.millis,
            stats: raster::stats(&f.flat, w, h),
            motion: scan.motion(index).unwrap_or(0.0),
        });
    }
    // Pops anywhere else: a pure movie renders in any order, so a fixed
    // set of stretches of consecutive frames spread over it is scanned
    // (every frame with --all). A movie that depends on its history is only
    // scanned around the samples.
    let sweep_started = Instant::now();
    let mut cut_short = false;
    if !c.has("all") {
        cut_short = scan.sweep(&mut movie, first, SWEEP_CHUNKS, SWEEP_CEILING)?;
    }
    let sweep_ms = sweep_started.elapsed().as_secs_f64() * 1e3;
    let possible = (end - first).saturating_sub(5) as usize;
    // Judgements an agent would otherwise have to look at pixels to make.
    let mut warnings: Vec<String> = Vec::new();
    // A run of blank frames at the very start or end is a fade from or to
    // the ground, which is how films open and close: a note. Blank frames
    // in the middle, or nothing but blank frames, are a warning.
    let is_blank = |s: &Sample| s.stats.coverage < 0.002;
    let lead = samples.iter().take_while(|s| is_blank(s)).count();
    let trail = samples.iter().rev().take_while(|s| is_blank(s)).count();
    let mut notes: Vec<String> = Vec::new();
    if lead == samples.len() {
        warnings.push("every sampled frame is blank: the scene draws nothing visible (check colours against the background, and coordinates against the canvas size)".into());
    } else {
        let middle: Vec<u64> = samples[lead..samples.len() - trail]
            .iter()
            .filter(|s| is_blank(s))
            .map(|s| s.index)
            .collect();
        if !middle.is_empty() {
            warnings.push(format!(
                "blank frames in the middle of the movie at {middle:?}: nothing is visible there"
            ));
        }
        let long = |n: usize| n > 1 && samples.len() >= 8 && n * 4 > samples.len();
        if long(lead) {
            warnings.push(format!("the first {lead} sampled frames are blank: the movie is slow to show anything (until about t={:.1}s)", samples[lead].time));
        } else if lead > 0 {
            notes.push("opens from the background".into());
        }
        if long(trail) {
            warnings.push(format!("the last {trail} sampled frames are blank: the movie ends early (after about t={:.1}s)", samples[samples.len() - trail - 1].time));
        } else if trail > 0 {
            notes.push("closes to the background".into());
        }
    }
    if !meta.shadowed.is_empty() {
        warnings.push(format!(
            "the program redefines psanim's own word{} {}: its code now gets its definition, not the engine's. Rename (for example /{}-x)",
            if meta.shadowed.len() == 1 { "" } else { "s" },
            meta.shadowed.iter().map(|w| format!("`{w}`")).collect::<Vec<_>>().join(", "),
            meta.shadowed[0]
        ));
    }
    let moving = samples.iter().filter(|s| s.motion > 1e-5).count();
    if samples.len() > 1 && moving == 0 {
        warnings.push("nothing moves: every sampled frame equals the one after it (does the scene read `now`?)".into());
    }
    let mut motions: Vec<f64> = samples
        .iter()
        .map(|s| s.motion)
        .filter(|m| *m > 0.0)
        .collect();
    motions.sort_by(f64::total_cmp);
    let median = motions.get(motions.len() / 2).copied().unwrap_or(0.0);
    let found = scan.findings();
    if cut_short {
        notes.push(format!("the scan for pops hit its {} s time limit with frames this slow, so which pairs it covered, and what it found, depend on the machine's speed", SWEEP_CEILING.as_secs()));
    }
    // Whole-frame cuts among the samples: the mean difference is enough for
    // a change that fills the frame.
    for s in &samples {
        if s.motion > 0.08 && s.motion > 8.0 * median.max(1e-4) {
            if s.index + 1 >= meta.frames || found.pops.iter().any(|p| p.frame == s.index) {
                continue; // the loop's join is judged below; a pop is reported once
            }
            warnings.push(format!("abrupt change after frame {} (t={:.2}s): difference {:.3} against a typical {median:.4}. A hard cut is fine if intended; otherwise something pops", s.index, s.time, s.motion));
        }
    }
    // Local pops, found tile by tile. A place that changes in steps again
    // and again is a readout or typing, and changes this frequent anywhere
    // are a style (sparkle, particles): notes, not a warning each.
    let covered = found.covered;
    for (area, frames) in &found.repeated {
        notes.push(format!(
            "the area x {:.0}..{:.0}, y {:.0}..{:.0} changes in steps, {} times in the scanned frames (after frames {:?}{}): taken as a readout or typing, not a pop",
            area[0], area[2], area[1], area[3], frames.len(),
            &frames[..frames.len().min(5)],
            if frames.len() > 5 { "..." } else { "" }
        ));
    }
    if meta.mode == psanim::Mode::Persistent {
        // Marks appearing are what a painting does. Only a change that
        // fills much of the frame (a cut) is a warning there.
        let (cuts, marks): (Vec<_>, Vec<_>) = found.pops.iter().partition(|p| p.tiles > 8);
        if !marks.is_empty() {
            notes.push(format!(
                "{} local change{} in the scanned frames (marks appearing on the canvas, after frames {:?}): expected in a persistent movie, not reported one by one",
                marks.len(),
                if marks.len() == 1 { "" } else { "s" },
                marks.iter().map(|p| p.frame).take(6).collect::<Vec<_>>()
            ));
        }
        for p in cuts {
            warnings.push(format!(
                "abrupt change after frame {} (t={:.2}s): {} of {} tiles change by {:.3} between this frame and the next, against {:.3} either side. A hard cut is fine if intended; otherwise something pops",
                p.frame, meta.time_of(p.frame), p.tiles, p.total_tiles, p.change, p.around
            ));
        }
    } else if found.pops.len() > 3 && found.pops.len() * 20 > covered {
        let at: Vec<u64> = found.pops.iter().map(|p| p.frame).take(6).collect();
        notes.push(format!(
            "{} of {covered} scanned frame pairs have a small element appearing or vanishing (after frames {at:?}...): taken as an effect, not reported one by one",
            found.pops.len()
        ));
    } else {
        for p in found.pops.iter().take(5) {
            warnings.push(format!(
                "abrupt change after frame {} (t={:.2}s): {} of {} tiles near x {:.0}..{:.0}, y {:.0}..{:.0} (canvas units) change by {:.3} between this frame and the next, against {:.3} in the pairs either side. Something appears, vanishes or jumps there. A hard cut is fine if intended; otherwise something pops (`psanim sheet --frame {},{}` to see it)",
                p.frame, meta.time_of(p.frame), p.tiles, p.total_tiles,
                p.area[0], p.area[2], p.area[1], p.area[3], p.change, p.around,
                p.frame, p.frame + 1
            ));
        }
        if found.pops.len() > 5 {
            warnings.push(format!(
                "and {} more abrupt changes, after frames {:?}",
                found.pops.len() - 5,
                found
                    .pops
                    .iter()
                    .skip(5)
                    .take(10)
                    .map(|p| p.frame)
                    .collect::<Vec<_>>()
            ));
        }
    }
    // A loop's join, measured directly whatever was sampled: the step from
    // the last frame to the first against the steps on either side of it.
    let mut seam = None;
    if meta.looping && meta.frames >= 4 {
        let mut step = |a: u64, b: u64| -> Result<f64, Fail> {
            let fa = movie.render_frame(a)?;
            raster::flatten_onto(fa.premultiplied, matte, &mut flat);
            let fb = movie.render_frame(b)?;
            raster::flatten_onto(fb.premultiplied, matte, &mut next);
            Ok(raster::difference(&flat, &next))
        };
        let last = meta.frames - 1;
        let join = step(last, 0)?;
        let before = step(last - 1, last)?;
        let after = step(0, 1)?;
        let typical = (before + after) / 2.0;
        seam = Some((join, typical));
        if join > typical * 1.25 + 2e-4 {
            let how = if typical < 1e-5 {
                "while the frames beside it do not change at all".to_string()
            } else {
                format!("{:.1}x the steps beside it ({typical:.4})", join / typical)
            };
            warnings.push(format!(
                "the loop does not close cleanly: the step from the last frame to the first is {join:.4}, {how}. Something does not return to where it started: check that every angle is a whole number of turns over the loop (`k turns`), allowing for each part's symmetry. `psanim sheet --seam 6` shows the join"
            ));
        }
    }
    // A scene that writes into a string made in setup: `restore` does not
    // roll string contents back, so the write outlives the frame.
    if let Some(before) = &strings_before
        && let Some(after) = movie.string_snapshot()?
    {
        let changed = after.changed_since(before);
        if let Some(first_path) = changed.first() {
            let name = first_path
                .split(['/', '['])
                .next()
                .unwrap_or(first_path.as_str());
            let list = changed
                .iter()
                .take(5)
                .map(|p| format!("`{p}`"))
                .collect::<Vec<_>>()
                .join(", ");
            let more = if changed.len() > 5 { ", and more" } else { "" };
            warnings.push(format!(
                "a frame wrote into {} made in setup: {list}{more}. PostScript's `restore` does not roll back the contents of a string, so the change stays for every later frame: a frame is no longer a function of its time, and a parallel render differs from a serial one. Copy the string inside the scene before writing to it (`/{name} {name} dup length string copy def`), or build it there",
                if changed.len() == 1 { "a string" } else { "strings" }
            ));
        }
    }
    let slowest = samples.iter().max_by(|a, b| a.millis.total_cmp(&b.millis));
    let mean_ms = samples.iter().map(|s| s.millis).sum::<f64>() / samples.len().max(1) as f64;
    if c.json {
        println!(
            "{}",
            json!({
                "ok": true, "file": path, "movie": meta.to_json(), "load_ms": load_ms,
                "mean_frame_ms": mean_ms,
                "slowest_frame": slowest.map(|s| json!({"frame": s.index, "ms": s.millis})),
                "loop_seam": seam.map(|(join, typical)| json!({"step": join, "typical_step": typical})),
                "pop_scan": {"pairs_judged": covered, "pairs_possible": possible, "ms": (sweep_ms * 10.0).round() / 10.0},
                "warnings": warnings, "notes": notes, "printed": movie.take_output(),
                "samples": samples.iter().map(|s| json!({
                    "frame": s.index, "time": s.time, "ms": (s.millis * 100.0).round() / 100.0,
                    "motion": (s.motion * 1e5).round() / 1e5, "stats": stats_json(&s.stats)})).collect::<Vec<_>>(),
            })
        );
        return Ok(());
    }
    println!(
        "{}: {}x{} units, {}s at {} fps = {} frames, {}",
        meta.title.as_deref().unwrap_or(&name_of(&path)),
        meta.width,
        meta.height,
        meta.duration,
        meta.fps,
        meta.frames,
        meta.mode.name()
    );
    println!(
        "  loaded in {load_ms:.1} ms; {} frames sampled at {}x{}",
        samples.len(),
        meta.pixel_width,
        meta.pixel_height
    );
    println!(
        "  frame    time    ms   luma  cover  motion   (motion: difference from the next frame)"
    );
    for s in &samples {
        println!(
            "  {:5} {:6.2}s {:5.1}  {:5.3}  {:5.3}  {:6.4}",
            s.index, s.time, s.millis, s.stats.luma, s.stats.coverage, s.motion
        );
    }
    if let Some(s) = slowest {
        println!(
            "  mean {mean_ms:.1} ms/frame; slowest frame {} at {:.1} ms (at preview size)",
            s.index, s.millis
        );
    }
    println!(
        "  scanned {covered} of {possible} frame pairs for pops, tile by tile{}",
        if c.has("all") {
            String::new()
        } else if meta.mode != psanim::Mode::Pure {
            " (around the sampled frames only; `--all` scans every frame)".to_string()
        } else if covered < possible {
            format!(" ({sweep_ms:.0} ms; `--all` scans every frame)")
        } else {
            format!(" ({sweep_ms:.0} ms)")
        }
    );
    if let Some((join, typical)) = seam {
        println!("  loop join: step {join:.4} against {typical:.4} beside it");
    }
    for n in &notes {
        println!("  note: {n}");
    }
    let printed = movie.take_output();
    if !printed.is_empty() {
        println!("  printed by the program:");
        for line in printed.lines().take(40) {
            println!("    {line}");
        }
    }
    if warnings.is_empty() {
        println!("  ok: no problems found");
    }
    for w in &warnings {
        println!("  warning: {w}");
    }
    Ok(())
}

fn cmd_bench(rest: &[String]) -> Result<(), Fail> {
    let c = parse(rest, &["count"])?;
    let (path, source) = c.source().map_err(usage)?;
    let opts = c.options(Scale::Factor(1.0)).map_err(usage)?;
    let started = Instant::now();
    let mut movie = Movie::load(&source, &opts)?;
    let load_ms = started.elapsed().as_secs_f64() * 1e3;
    let meta = movie.meta().clone();
    let count: usize = c.extra_num("count").map_err(usage)?.unwrap_or(48);
    // A pure movie is timed at frames spread over its length. One whose
    // frames depend on the ones before is timed over its first frames in
    // order, since reaching a later frame costs every frame before it.
    let picks: Vec<u64> = if meta.mode == psanim::Mode::Pure {
        sheet::spread(meta.frames, count)
    } else {
        (0..meta.frames.min(count as u64)).collect()
    };
    let mut times = Vec::new();
    for &k in &picks {
        let t = Instant::now();
        movie.render_frame(k)?;
        times.push(t.elapsed().as_secs_f64() * 1e3);
    }
    drop(movie);
    let serial_total: f64 = times.iter().sum();
    let mut sorted = times.clone();
    sorted.sort_by(f64::total_cmp);
    let pct = |p: f64| sorted[((sorted.len() - 1) as f64 * p).round() as usize];
    let threads = effective_threads(&meta, c.threads, picks.len());
    let t = Instant::now();
    render_frames(&source, &opts, &picks, threads, |_| Ok(()))?;
    let parallel_s = t.elapsed().as_secs_f64();
    let result = json!({
        "ok": true, "file": path, "width": meta.pixel_width, "height": meta.pixel_height,
        "mode": meta.mode.name(), "frames_timed": picks.len(), "load_ms": load_ms,
        "frame_ms": {"mean": serial_total / times.len() as f64, "median": pct(0.5), "p95": pct(0.95), "max": pct(1.0)},
        "serial_fps": 1000.0 * times.len() as f64 / serial_total,
        "threads": threads,
        "parallel_fps": picks.len() as f64 / parallel_s,
        "speedup": (picks.len() as f64 / parallel_s) / (1000.0 * times.len() as f64 / serial_total),
    });
    if c.json {
        println!("{result}");
    } else {
        println!(
            "{} at {}x{} ({}), {} frames timed",
            name_of(&path),
            meta.pixel_width,
            meta.pixel_height,
            meta.mode.name(),
            picks.len()
        );
        println!("  load           {load_ms:8.1} ms");
        println!(
            "  frame median   {:8.2} ms   mean {:.2}   p95 {:.2}   max {:.2}",
            pct(0.5),
            serial_total / times.len() as f64,
            pct(0.95),
            pct(1.0)
        );
        if threads > 1 {
            println!(
                "  {threads} thread{}      {:8.1} frames/s  ({:.1}x, load included)",
                if threads == 1 { " " } else { "s" },
                result["parallel_fps"].as_f64().unwrap_or(0.0),
                result["speedup"].as_f64().unwrap_or(0.0)
            );
        }
    }
    Ok(())
}

fn cmd_api(rest: &[String]) -> Result<(), Fail> {
    let c = parse(rest, &["markdown", "libs"])?;
    if c.json {
        println!("{}", psanim::api::to_json());
        return Ok(());
    }
    let libraries = || {
        for (name, _, what) in psanim::LIBRARIES {
            println!("  /{name} use    {what}");
        }
    };
    if c.has("libs") {
        println!("Libraries (load in setup, after `animation`):");
        libraries();
        println!("\n`psanim api pkmotion` lists one library's words.");
        return Ok(());
    }
    let named = c
        .file
        .as_ref()
        .map(|w| w.to_string_lossy().trim_start_matches('/').to_string());
    if let Some(lib) = named
        .as_deref()
        .filter(|w| psanim::api::library_of(w).is_some())
    {
        if c.has("markdown") {
            print!("{}", psanim::api::library_markdown(lib).unwrap_or_default());
            return Ok(());
        }
        let (_, title) = psanim::api::CATEGORIES
            .iter()
            .find(|c| c.0 == lib)
            .expect("a library has a category");
        println!("{title}\n  load with: /{lib} use\n");
        for e in psanim::api::API.iter().filter(|e| e.category == lib) {
            println!("  {}", e.stack);
        }
        if lib == "pkmotion" {
            let of = |kind| {
                psanim::motion::MOTIONS
                    .iter()
                    .filter(|m| m.kind == kind)
                    .map(|m| format!("/{}", m.name))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            println!("\nEntrances: {}", of(psanim::motion::Kind::Entrance));
            println!("Emphases: {}", of(psanim::motion::Kind::Emphasis));
        }
        println!("\n`psanim api NAME` explains one word.");
        return Ok(());
    }
    if c.has("markdown") {
        print!("{}", psanim::api::to_markdown());
        return Ok(());
    }
    if let Some(word) = &named {
        let word = word.as_str();
        // The two names a program defines for the engine to call.
        match word {
            "scene" => {
                println!(
                    "-  scene  -     (you define it; the engine calls it)\n\n  Draws one frame. Called once per frame on an empty operand stack, which it must leave empty, with a clean graphics state (no font, no path) and the interpreter's memory as setup left it. Read the time with `now`. Rename it with /Scene in the animation dictionary.\n"
                );
                return Ok(());
            }
            "step" => {
                println!(
                    "-  step  -      (optional; you define it; the engine calls it)\n\n  Advances a simulation by one tick. Defining it makes the movie stateful: it is called at /StepRate ticks per second (default: the FPS), in order, on an empty operand stack, and what it changes persists. Inside it `dt` is the tick length, `now` the tick's start, `frame` the tick index. Frame 0 is drawn before any tick. See the skill's modes.md.\n"
                );
                return Ok(());
            }
            "overlay" => {
                println!(
                    "-  overlay  -   (optional, persistent movies; you define it)\n\n  Drawn over each frame of a /Persist true movie as shown, then discarded: a brush, a caption, a vignette. It cannot change the painting.\n"
                );
                return Ok(());
            }
            _ => {}
        }
        if let Some(m) = psanim::motion::find(word) {
            let kind = if m.kind == psanim::motion::Kind::Emphasis {
                "an emphasis"
            } else {
                "an entrance"
            };
            println!(
                "/{}  is a motion ({kind}), not a word: {}.\n\n  Use it where a motion goes: {}.\n  Default {} s, easing /{}. In the pkmotion library (`/pkmotion use`); `psanim api pkmotion` lists them all.\n",
                m.name,
                m.what,
                if m.kind == psanim::motion::Kind::Emphasis {
                    format!("`2 /{} {{ ... }} emphasized`", m.name)
                } else {
                    format!(
                        "`2 /{0} {{ ... }} enters`, `[/{0} {1} /{2}] {{ ... }} entering`",
                        m.name, m.dur, m.ease
                    )
                },
                m.dur,
                m.ease
            );
            return Ok(());
        }
        let Some(e) = psanim::api::lookup(word) else {
            let near: Vec<&str> = psanim::api::API
                .iter()
                .map(|e| e.name)
                .filter(|n| n.contains(word))
                .collect();
            return Err(usage(format!(
                "no word '{word}' in the vocabulary{}",
                if near.is_empty() {
                    String::new()
                } else {
                    format!(" (related: {})", near.join(" "))
                }
            )));
        };
        println!("{}\n\n  {}\n", e.stack, e.summary);
        if let Some(lib) = psanim::api::library_of(e.category) {
            println!("  In the {lib} library: load it with `/{lib} use`.\n");
        }
        return Ok(());
    }
    for (key, title) in psanim::api::CATEGORIES {
        if psanim::api::library_of(key).is_some() {
            continue;
        }
        println!("{title}");
        for e in psanim::api::API.iter().filter(|e| e.category == *key) {
            println!("  {}", e.stack);
        }
        println!();
    }
    println!("Easings: {}", psanim::ease::NAMES.join(" "));
    println!("Transitions: {}", psanim::TRANSITIONS.join(" "));
    println!("\nLibraries (`psanim api pkmotion` lists one's words):");
    libraries();
    println!(
        "\n`psanim api NAME` explains one word; `psanim api --markdown` prints the full reference."
    );
    Ok(())
}
