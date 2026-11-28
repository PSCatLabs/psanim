//! The engine: a PostScript program loaded as a movie, and frames out of it.
//!
//! One [`Movie`] owns one interpreter. Loading runs the program's top level
//! once (*setup*) and checkpoints the interpreter's memory with PostScript's
//! own `save`. Every frame after that starts by rolling back to a
//! checkpoint, so a frame is a function of its time and nothing else: frame
//! 300 rendered cold is the same pixels as frame 300 rendered after frames
//! 0 to 299. Stateful movies advance a simulation in fixed ticks between
//! nested checkpoints instead; see [`Mode`].

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use pscat::host::Value;
use pscat::{Interp, Limits, Page, PsError};

use crate::ctx::{Canvas, Ctx, Running, last_step_at, mix64};
use crate::error::{Error, Phase, closest};
use crate::raster::Accumulator;
use crate::signals::Signals;

/// The PostScript prelude, run before every program.
pub const PRELUDE: &str = include_str!("prelude.ps");

/// How the output's pixel size is chosen from the movie's canvas size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    /// Pixels per canvas unit: 1.0 renders a 1280x720 canvas at 1280x720,
    /// 0.5 at 640x360 for a quick preview, 2.0 at 2560x1440.
    Factor(f64),
    /// Scale so the output is this many pixels wide.
    Width(u32),
    /// Scale so the output is this many pixels high.
    Height(u32),
}

/// Averaging several moments into each frame: motion blur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionBlur {
    /// How many moments are rendered and averaged per frame (2 or more;
    /// 8 is smooth for most motion).
    pub samples: u32,
    /// The fraction of the frame interval the shutter is open, 0..=1.
    /// 0.5 is the film-camera "180 degree shutter".
    pub shutter: f64,
}

/// A value for a program's `param`.
#[derive(Clone, Debug, PartialEq)]
pub enum Param {
    Number(f64),
    Text(String),
}

impl Param {
    /// Read a command-line value: a number if it parses as one, else text.
    pub fn parse(text: &str) -> Param {
        text.parse::<f64>()
            .map_or_else(|_| Param::Text(text.to_string()), Param::Number)
    }
}

/// Caps on the work a program may do, separately for setup and per frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Budget {
    /// For the program's top level.
    pub setup: Limits,
    /// For each frame (and each simulation tick).
    pub frame: Limits,
}

impl Budget {
    /// No caps: for programs you wrote.
    pub fn trusted() -> Budget {
        Budget::default()
    }

    /// Caps for programs from strangers: bounded steps and time for setup
    /// and for every frame. A memory cap additionally needs
    /// [`pscat::TrackingAllocator`] installed as the global allocator, so it
    /// is not part of this preset; add it with
    /// `budget.frame.with_max_memory(..)` when the allocator is installed.
    pub fn untrusted() -> Budget {
        Budget {
            setup: Limits::none()
                .with_max_steps(400_000_000)
                .with_max_time(std::time::Duration::from_secs(30)),
            frame: Limits::none()
                .with_max_steps(200_000_000)
                .with_max_time(std::time::Duration::from_secs(20)),
        }
    }
}

/// Everything a host can decide about a render. `Default` renders the movie
/// as declared, at one pixel per canvas unit, with no limits.
#[derive(Clone, Debug)]
pub struct Options {
    pub scale: Scale,
    /// Override the declared frame rate.
    pub fps: Option<f64>,
    /// Override the declared duration, in seconds.
    pub duration: Option<f64>,
    /// Override the declared seed.
    pub seed: Option<i64>,
    /// Values for the program's `param` lookups.
    pub params: BTreeMap<String, Param>,
    /// Signals and events for `signal`, `sinceevent` and `eventcount`.
    pub signals: Signals,
    pub budget: Budget,
    /// Override the declared motion blur. `Some(None)` turns it off.
    pub motion_blur: Option<Option<MotionBlur>>,
    /// Directories the program may read files from (`run`, `file`,
    /// images). `None` leaves reads open; `Some(vec![])` confines the
    /// program to the libraries built into the interpreter. Writes are
    /// never allowed.
    pub read_roots: Option<Vec<PathBuf>>,
    /// The directory relative paths in `loadcomponent` are read from:
    /// normally the movie file's own. `None` reads them from the current
    /// directory.
    pub base_dir: Option<PathBuf>,
    /// A flag another thread raises to stop a render in progress.
    pub interrupt: Option<Arc<AtomicBool>>,
    /// Seconds from some fixed point, for timing the code before
    /// `animation` (see [`Meta::preamble_warning`]). The engine keeps no
    /// clock of its own, so a host that wants the timing passes one; with
    /// `None` the time is not measured.
    pub clock: Option<fn() -> f64>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            scale: Scale::Factor(1.0),
            fps: None,
            duration: None,
            seed: None,
            params: BTreeMap::new(),
            signals: Signals::default(),
            budget: Budget::trusted(),
            motion_blur: None,
            read_roots: None,
            base_dir: None,
            interrupt: None,
            clock: None,
        }
    }
}

/// How a movie's frames depend on each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Each frame is a function of its time alone. Frames can be rendered
    /// in any order and in parallel.
    Pure,
    /// The program defines `step`: a simulation advanced in fixed ticks.
    /// A frame shows the state after all ticks up to its time. Seeking
    /// backward replays from the nearest checkpoint.
    Stateful,
    /// `/Persist true`: the canvas is not erased between frames, so each
    /// frame paints over the last. Frames exist only in order; seeking
    /// replays from the nearest checkpoint.
    Persistent,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Pure => "pure",
            Mode::Stateful => "stateful",
            Mode::Persistent => "persistent",
        }
    }
}

/// What a loaded movie is: the declaration, with the options applied.
#[derive(Clone, Debug, PartialEq)]
pub struct Meta {
    pub title: Option<String>,
    pub description: Option<String>,
    /// Canvas size in units: the coordinate space the program draws in.
    pub width: u32,
    pub height: u32,
    /// Output size in pixels: the canvas size times `scale`.
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub scale: f64,
    pub fps: f64,
    /// Seconds.
    pub duration: f64,
    /// `ceil(duration * fps)`. Frame `k` shows time `k / fps`.
    pub frames: u64,
    pub seed: i64,
    /// The colour each frame starts from; `None` is transparent.
    pub background: Option<[f64; 3]>,
    /// The last frame leads back into the first.
    pub looping: bool,
    /// Moments (seconds) where the movie cuts on purpose: `check` does not
    /// report the abrupt change there.
    pub cuts: Vec<f64>,
    pub mode: Mode,
    /// Simulation ticks per second, for a stateful or persistent movie.
    pub tick_rate: f64,
    pub motion_blur: Option<MotionBlur>,
    /// The parameters the program asked for during setup, with the
    /// defaults it gave, printed as PostScript.
    pub params: BTreeMap<String, String>,
    /// psanim words the program redefined in setup (`/width 300 def`).
    /// Its own code then gets its definition instead of the engine's.
    pub shadowed: Vec<String>,
    /// Seconds the program's code before `animation` took in the probe
    /// pass, which runs it a second time. `None` when no clock was given
    /// in [`Options`] or the program does not call `animation`.
    pub preamble_seconds: Option<f64>,
    /// The code before `animation` printed something.
    pub preamble_printed: bool,
}

/// A preamble slower than this many seconds is worth a warning: it runs
/// twice on every load.
pub const PREAMBLE_WARN_SECONDS: f64 = 0.05;

impl Meta {
    /// A warning for a program whose code before `animation` prints or
    /// takes more than [`PREAMBLE_WARN_SECONDS`]: the declaration is read
    /// by a first pass that runs that code, so it runs twice.
    pub fn preamble_warning(&self) -> Option<String> {
        let slow = self.preamble_seconds.filter(|s| *s > PREAMBLE_WARN_SECONDS);
        let what = match (slow, self.preamble_printed) {
            (Some(s), true) => format!("prints and takes {:.0} ms", s * 1e3),
            (Some(s), false) => format!("takes {:.0} ms", s * 1e3),
            (None, true) => "prints".to_string(),
            (None, false) => return None,
        };
        Some(format!(
            "the code before `animation` {what}, and it runs twice: the declaration is found by a first pass that runs everything above it. Move that code after `animation`; above it keep only cheap definitions"
        ))
    }

    /// The time frame `index` shows.
    pub fn time_of(&self, index: u64) -> f64 {
        index as f64 / self.fps
    }

    /// The frame that shows time `t` (the last frame at or before it).
    pub fn frame_at(&self, t: f64) -> u64 {
        ((t * self.fps + 1e-9).floor().max(0.0) as u64).min(self.frames.saturating_sub(1))
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "title": self.title,
            "description": self.description,
            "width": self.width,
            "height": self.height,
            "pixel_width": self.pixel_width,
            "pixel_height": self.pixel_height,
            "scale": self.scale,
            "fps": self.fps,
            "duration": self.duration,
            "frames": self.frames,
            "seed": self.seed,
            "background": self.background,
            "loop": self.looping,
            "cuts": self.cuts,
            "mode": self.mode.name(),
            "tick_rate": self.tick_rate,
            "motion_blur": self.motion_blur.map(|m| serde_json::json!({"samples": m.samples, "shutter": m.shutter})),
            "params": self.params,
            "shadowed": self.shadowed,
            "preamble_ms": self.preamble_seconds.map(|s| s * 1e3),
            "preamble_printed": self.preamble_printed,
        })
    }
}

/// A rendered frame: pixels borrowed from the movie, valid until the next
/// render.
#[derive(Clone, Copy, Debug)]
pub struct Frame<'a> {
    pub index: u64,
    /// Seconds.
    pub time: f64,
    pub width: u32,
    pub height: u32,
    /// Premultiplied RGBA, rows top to bottom.
    pub premultiplied: &'a [u8],
}

impl Frame<'_> {
    /// Straight-alpha RGBA: what a PNG holds.
    pub fn to_rgba8(&self) -> Vec<u8> {
        crate::raster::to_straight(self.premultiplied)
    }

    /// The frame as a [`pscat::Page`], for PNG encoding.
    pub fn to_page(&self) -> Page {
        Page::from_premultiplied_rgba8(self.width, self.height, self.premultiplied.to_vec())
            .expect("a frame is a valid page size")
    }

    /// The frame encoded as PNG.
    pub fn to_png(&self) -> Vec<u8> {
        self.to_page()
            .to_png()
            .expect("encoding to memory cannot fail")
    }
}

/// A point the engine can roll the interpreter back to.
struct Checkpoint {
    /// Ticks applied when it was taken.
    tick: u64,
    /// The PostScript save object.
    save: Value,
    /// The canvas at that moment, for a persistent movie.
    canvas: Option<Vec<u8>>,
}

/// A fingerprint of every string that setup left in userdict, directly or
/// inside the dictionaries and arrays it holds, taken by
/// [`Movie::string_snapshot`].
///
/// PostScript's `restore` does not roll back the contents of a string, so a
/// scene that `put`s into a string made in setup changes it for every later
/// frame, and frames stop being functions of their time. Two snapshots of a
/// pure movie, one after setup and one after frames, tell which strings
/// changed.
#[derive(Clone, Debug, Default)]
pub struct StringSnapshot {
    /// Path (`buf`, `table/name`, `rows[3]`) to a hash of the bytes.
    strings: BTreeMap<String, u64>,
}

impl StringSnapshot {
    /// How many strings were fingerprinted.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Paths of the strings whose bytes differ from `before`'s, in order.
    pub fn changed_since(&self, before: &StringSnapshot) -> Vec<String> {
        self.strings
            .iter()
            .filter(|(path, hash)| before.strings.get(*path).is_some_and(|h| h != *hash))
            .map(|(path, _)| path.clone())
            .collect()
    }
}

/// FNV-1a, 64 bit: deterministic and enough to tell edited bytes.
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in bytes {
        h = (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
    }
    h ^ (bytes.len() as u64)
}

/// A loaded animation program. See the [crate docs](crate).
pub struct Movie {
    interp: Interp,
    ctx: Rc<RefCell<Ctx>>,
    meta: Meta,
    scene: String,
    step: Option<String>,
    checkpoints: Vec<Checkpoint>,
    /// Ticks applied to the interpreter's current state.
    cursor: u64,
    checkpoint_every: u64,
    accum: Accumulator,
    blurred: Vec<u8>,
    frame_limits: Limits,
    /// Interpreter steps the last frame took, when a step limit is set.
    last_steps: u64,
    /// What the program printed and nobody has collected yet.
    output: String,
    /// `output` was cut at [`MAX_OUTPUT`].
    output_truncated: bool,
    /// The persistent canvas, set aside while a frame's overlay is shown.
    kept: Option<Page>,
    /// The overlay procedure of a persistent movie, if it defines one.
    overlay: Option<String>,
    /// Simulation ticks the last render had to apply.
    last_ticks: u64,
    /// UTF-8 text shown in a font that reads bytes, one per font, kept
    /// across runs (PSCat forgets its findings when the next run starts).
    byte_font_text: ByteFontReport,
}

/// UTF-8 text a program showed in a font that reads one character per byte.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ByteFontText {
    font: String,
    characters: Vec<char>,
    more: bool,
    line: Option<usize>,
}

/// The most fonts [`Movie::text_warnings`] names for UTF-8 shown in a byte font.
const MAX_BYTE_FONTS: usize = 5;

/// The most further fonts counted by name past [`MAX_BYTE_FONTS`].
const MAX_BYTE_FONTS_COUNTED: usize = 1024;

/// UTF-8 text shown in byte fonts over a movie's life: the first fonts by
/// name, and how many more there were.
#[derive(Clone, Debug, Default)]
struct ByteFontReport {
    fonts: Vec<ByteFontText>,
    /// Fonts past the first [`MAX_BYTE_FONTS`], by name.
    more: std::collections::BTreeSet<String>,
    /// The most fonts one run had that PSCat itself did not name.
    unnamed: usize,
}

impl ByteFontReport {
    /// How many fonts past the named ones showed UTF-8 text: at least this many.
    fn more_fonts(&self) -> usize {
        self.more.len().max(self.unnamed)
    }
}

/// Add what the interpreter's last run found of UTF-8 text in byte fonts.
/// `lines`: the run was the program's own text, so a line number means
/// something (a frame runs the engine's one-line driver).
fn collect_byte_font_text(interp: &Interp, into: &mut ByteFontReport, lines: bool) {
    let report = interp.utf8_text_findings();
    into.unnamed = into.unnamed.max(report.more_fonts);
    for f in report.fonts {
        if into.fonts.iter().any(|k| k.font == f.font) {
            continue;
        }
        if into.fonts.len() >= MAX_BYTE_FONTS {
            if into.more.len() < MAX_BYTE_FONTS_COUNTED {
                into.more.insert(f.font);
            }
            continue;
        }
        into.fonts.push(ByteFontText {
            font: f.font,
            characters: f.characters,
            more: f.more_characters,
            line: f.line.filter(|_| lines),
        });
    }
}

/// The vocabulary's native operators that a program is likely to shadow
/// by accident: short, ordinary words.
const NATIVE_WORDS: &[&str] = &[
    "now", "progress", "span", "frame", "fps", "dt", "steprate", "stepnow", "stepat", "duration",
    "width", "height", "turns", "ease", "tween", "ramp", "between", "clamp", "remap", "signal",
    "param", "grain", "vignette", "wiggle", "fmod",
];

/// The most printed text a movie keeps between calls of `take_output`.
const MAX_OUTPUT: usize = 64 * 1024;

/// The line that ends printed text cut at [`MAX_OUTPUT`].
const TRUNCATION_MARK: &str = "[output truncated at 64 KB]\n";

/// What a program printed since the last collection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Printed {
    /// The text. When it was cut at 64 KB it ends with a line saying so.
    pub text: String,
    /// More was printed than `text` holds.
    pub truncated: bool,
}

impl Printed {
    /// Add `more` after this text, keeping the whole within the 64 KB a
    /// movie keeps: what a render of many frames collects, frame by frame.
    pub fn append(&mut self, more: &Printed) {
        push_output(&mut self.text, &mut self.truncated, more.text.as_bytes());
        self.truncated |= more.truncated;
    }
}

/// Add `bytes` to `out`, keeping at most [`MAX_OUTPUT`] bytes and ending
/// with a line that says so when anything was dropped.
fn push_output(out: &mut String, truncated: &mut bool, bytes: &[u8]) {
    if *truncated || bytes.is_empty() {
        return;
    }
    let text = String::from_utf8_lossy(bytes);
    let room = MAX_OUTPUT.saturating_sub(out.len());
    if text.len() <= room {
        out.push_str(&text);
        return;
    }
    let mut cut = room;
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    out.push_str(&text[..cut]);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(TRUNCATION_MARK);
    *truncated = true;
}

fn short_repr(v: &Value) -> String {
    let s = match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Real(r) => format!("{r}"),
        Value::Name(n) => format!("/{n}"),
        Value::String(s) => format!("({})", String::from_utf8_lossy(s)),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(short_repr).collect::<Vec<_>>().join(" ")
        ),
        Value::Dict(d) => format!("<<{} entries>>", d.len()),
        Value::Other(o) if o.is_procedure() => "{...}".to_string(),
        Value::Other(o) => format!("-{}-", o.type_name().trim_end_matches("type")),
        _ => "?".to_string(),
    };
    if s.chars().count() > 40 {
        format!("{}...", s.chars().take(37).collect::<String>())
    } else {
        s
    }
}

/// The names a mistyped psanim word is compared against.
fn vocabulary() -> impl Iterator<Item = &'static str> {
    crate::api::API.iter().map(|e| e.name)
}

fn explain(kind: &str, command: Option<&str>) -> Option<String> {
    Some(match (kind, command) {
        (
            "rangecheck" | "invalidfont" | "typecheck",
            Some(c @ ("pixelface" | "pixelfont" | "pixelvariant")),
        ) => {
            let faces: Vec<String> = crate::vocab::pixel_faces()
                .iter()
                .map(|(n, _)| format!("/{n}"))
                .collect();
            let other = match c {
                "pixelfont" => "5, 7 or 9 for psanim's own fonts, or ",
                "pixelface" => "a font that exists (/Courier-Bold), or ",
                _ => "",
            };
            format!(
                "`{c}` takes {other}one of PSCat's pixel faces: {}. Check the spelling: an unknown name is an error here, not a substitute font.",
                faces.join(" ")
            )
        }
        ("undefined", Some(name))
            if crate::api::lookup(name)
                .and_then(|e| crate::api::library_of(e.category))
                .is_some() =>
        {
            let lib = crate::api::lookup(name)
                .and_then(|e| crate::api::library_of(e.category))
                .unwrap_or_default();
            format!(
                "`{name}` is in the {lib} library, which is not loaded. Add `/{lib} use` in setup, after `animation`."
            )
        }
        ("undefined", Some(name)) => match closest(name, vocabulary()) {
            Some(near) => format!("`{name}` is not defined. Did you mean `{near}`?"),
            None => format!(
                "`{name}` is not defined. Check the spelling, and that the library defining it was loaded in setup: `/pkmotion use` for psanim's, `(lib/artkit.ps) run` for PSCat's."
            ),
        },
        ("stackunderflow", Some(c)) => format!(
            "`{c}` found too few operands. Count what the previous lines leave on the stack; a combinator like `during` or `eachglyph` takes its procedure last."
        ),
        ("typecheck", Some(c)) => format!(
            "`{c}` got an operand of the wrong type. The operand stack above shows what it was handed."
        ),
        ("limitexceeded", _) => {
            "The frame did more work than its budget allows. Precompute in setup what does not change from frame to frame, lower the detail, or raise the budget."
                .to_string()
        }
        ("invalidrestore", _) => {
            "A `restore` in the program rolled back past psanim's own checkpoint. Use `gsave`/`grestore` to scope graphics state; leave `save`/`restore` to the engine."
                .to_string()
        }
        ("invalidfont", _) => {
            "No usable font is selected. Every frame starts from a clean graphics state, so select the font inside the scene: /Helvetica-Bold findfont 48 scalefont setfont."
                .to_string()
        }
        ("nocurrentpoint", Some(c)) => format!(
            "`{c}` needs a current point. Every frame starts with an empty path: `x y moveto` first."
        ),
        ("invalidfileaccess", _) => {
            "The program tried to read a file outside the allowed directories, or to write one. Movies cannot write files.".to_string()
        }
        _ => return None,
    })
}

impl Movie {
    /// Load `source` as a movie: find its `animation` declaration, build an
    /// interpreter of the right size, run the program's top level, and take
    /// the first checkpoint.
    pub fn load(source: &str, options: &Options) -> Result<Movie, Error> {
        // Pass 1, the probe: run just far enough to read the declaration,
        // which decides the canvas the real interpreter is built with.
        let probed = probe(source, options)?;
        let mut decl = Declaration::read(probed.decl.as_ref())?;
        if let Some(fps) = options.fps {
            decl.fps = fps;
        }
        if let Some(d) = options.duration {
            decl.duration = d;
        }
        if let Some(s) = options.seed {
            decl.seed = s;
        }
        if let Some(mb) = options.motion_blur {
            decl.motion_blur = mb;
        }
        if !(decl.fps.is_finite() && decl.fps > 0.0 && decl.fps <= 1000.0) {
            return Err(Error::new(
                Phase::Load,
                "options",
                format!("frame rate {} is not usable", decl.fps),
            )
            .hint("FPS must be above 0 and at most 1000"));
        }
        if !(decl.duration.is_finite() && decl.duration > 0.0) {
            return Err(Error::new(
                Phase::Load,
                "options",
                format!("duration {} is not usable", decl.duration),
            )
            .hint("Duration is in seconds and must be positive"));
        }
        if let Some(mb) = decl.motion_blur
            && !((2..=64).contains(&mb.samples) && mb.shutter > 0.0 && mb.shutter <= 1.0)
        {
            return Err(Error::new(
                Phase::Load,
                "options",
                "motion blur needs 2 to 64 samples and a shutter in (0, 1]",
            ));
        }
        let scale = match options.scale {
            Scale::Factor(f) => f,
            Scale::Width(w) => f64::from(w) / f64::from(decl.width),
            Scale::Height(h) => f64::from(h) / f64::from(decl.height),
        };
        let mut interp = (scale.is_finite() && scale > 0.0)
            .then(|| Interp::with_page_scaled(decl.width, decl.height, scale as f32))
            .flatten()
            .ok_or_else(|| {
                Error::new(
                    Phase::Load,
                    "options",
                    format!(
                        "a {}x{} canvas at scale {scale} is not a usable size",
                        decl.width, decl.height
                    ),
                )
                .hint("the output must be between 1 and 8000 pixels on each side")
            })?;
        let (pixel_width, pixel_height) = interp.canvas_size();

        let ctx = Rc::new(RefCell::new(Ctx::new()));
        {
            let mut c = ctx.borrow_mut();
            c.width = f64::from(decl.width);
            c.height = f64::from(decl.height);
            c.fps = decl.fps;
            c.duration = decl.duration;
            c.seed = decl.seed;
            c.px_scale = scale;
            c.signals = options.signals.clone();
            c.params = host_params(&options.params);
            c.base_dir = options.base_dir.clone();
            c.running = Running::Setup;
            c.begin_moment(0, 0.0, 1.0 / decl.fps);
            // Known before the program runs, so setup can count steps too.
            c.step_rate = if decl.persist {
                decl.declared_fps
            } else {
                decl.fps
            };
            // A persistent movie's setup is before its first step, which is
            // a step of the declared rate: `dt` there is that step's.
            c.dt = 1.0 / c.step_rate;
        }
        configure(&mut interp, &ctx, options)?;
        interp
            .set_limits(options.budget.setup)
            .map_err(|e| Error::new(Phase::Load, "options", e.to_string()))?;
        // The page starts transparent; each frame paints its own ground.
        interp.set_transparent_background();
        run(
            &mut interp,
            &ctx,
            Phase::Setup,
            &format!("{} srand {} noiseseed", decl.seed, decl.seed),
        )?;
        run(&mut interp, &ctx, Phase::Setup, PRELUDE)?;
        run(&mut interp, &ctx, Phase::Setup, source)?;
        let mut byte_font_text = ByteFontReport::default();
        collect_byte_font_text(&interp, &mut byte_font_text, true);
        // `reelduration` guessed which names were transitions; everything is
        // defined now, so a wrong guess is an error, not a longer film.
        crate::ops::check_measured_reels(&interp, &ctx).map_err(|msg| {
            Error::new(Phase::Setup, "reelduration", msg).hint(
                "reelduration runs before the reel's clips and transitions are defined, so it takes only the built-in transitions and names starting t- for transitions",
            )
        })?;

        let is_proc = |interp: &Interp, name: &str| matches!(interp.lookup(name), Some(Value::Other(o)) if o.is_procedure());
        if !is_proc(&interp, &decl.scene) {
            return Err(Error::new(
                Phase::Setup,
                "declaration",
                format!("the program does not define a procedure named `{}`", decl.scene),
            )
            .hint(format!(
                "define the drawing procedure: /{} {{ ... }} def. It is called once per frame; read the time with `now`.",
                decl.scene
            )));
        }
        let step = match &decl.step {
            Some(name) if is_proc(&interp, name) => Some(name.clone()),
            Some(name) if decl.step_declared => {
                return Err(Error::new(
                    Phase::Setup,
                    "declaration",
                    format!("/Step names `{name}`, which is not a procedure"),
                ));
            }
            _ => None,
        };
        let mode = if decl.persist {
            Mode::Persistent
        } else if step.is_some() {
            Mode::Stateful
        } else {
            Mode::Pure
        };
        if mode != Mode::Pure && decl.motion_blur.is_some() {
            return Err(Error::new(
                Phase::Load,
                "options",
                format!("motion blur needs a pure movie; this one is {}", mode.name()),
            )
            .hint("motion blur renders several moments per frame, which a simulation or a persistent canvas cannot rewind for"));
        }
        let tick_rate = if mode == Mode::Persistent {
            decl.declared_fps
        } else {
            decl.step_rate
        };
        ctx.borrow_mut().step_rate = if mode == Mode::Pure {
            decl.fps
        } else {
            tick_rate
        };
        // A persistent movie may define `overlay`: drawn over each frame
        // shown, never kept.
        let overlay = (mode == Mode::Persistent && is_proc(&interp, "overlay"))
            .then(|| "overlay".to_string());
        // Native words the program has redefined: its own code gets its
        // definition, which is rarely what was meant.
        let shadowed: Vec<String> = crate::api::API
            .iter()
            .filter(|e| NATIVE_WORDS.contains(&e.name))
            .filter(|e| {
                !matches!(interp.lookup(e.name), Some(Value::Other(o)) if o.type_name() == "operatortype")
                    && !is_prelude_word(&mut interp, e.name)
            })
            .map(|e| e.name.to_string())
            .collect();

        interp.clear_operands();
        interp
            .set_limits(options.budget.frame)
            .map_err(|e| Error::new(Phase::Load, "options", e.to_string()))?;
        run(
            &mut interp,
            &ctx,
            Phase::Setup,
            "cleardictstack grestoreall save",
        )?;
        let save = interp
            .pop_value()
            .map_err(|_| Error::new(Phase::Setup, "internal", "no checkpoint was taken"))?;

        // Only this pass's printing is kept: the probe's interpreter is
        // gone, so what the code before `animation` prints is shown once.
        let mut setup_output = String::new();
        let mut setup_truncated = false;
        push_output(
            &mut setup_output,
            &mut setup_truncated,
            interp.captured_stdout(),
        );
        interp.capture_stdout();
        let frames = ((decl.duration * decl.fps - 1e-9).ceil().max(1.0)) as u64;
        // Checkpoints about every two seconds, and never more than 64 deep.
        let total_ticks = (decl.duration * tick_rate).ceil().max(1.0);
        let checkpoint_every = ((tick_rate * 2.0).max(total_ticks / 64.0).ceil() as u64).max(1);
        let params = ctx.borrow().params_seen.clone();
        let mut movie = Movie {
            interp,
            ctx,
            meta: Meta {
                title: decl.title,
                description: decl.description,
                width: decl.width,
                height: decl.height,
                pixel_width,
                pixel_height,
                scale,
                fps: decl.fps,
                duration: decl.duration,
                frames,
                seed: decl.seed,
                background: decl.background,
                looping: decl.looping,
                cuts: decl.cuts,
                mode,
                tick_rate,
                motion_blur: decl.motion_blur,
                params,
                shadowed,
                preamble_seconds: probed.seconds,
                preamble_printed: probed.printed,
            },
            scene: decl.scene,
            step,
            checkpoints: vec![Checkpoint {
                tick: 0,
                save,
                canvas: None,
            }],
            cursor: 0,
            checkpoint_every,
            accum: Accumulator::default(),
            blurred: Vec::new(),
            frame_limits: options.budget.frame,
            last_steps: 0,
            output: setup_output,
            output_truncated: setup_truncated,
            byte_font_text,
            kept: None,
            overlay,
            last_ticks: 0,
        };
        if mode == Mode::Persistent {
            movie.reset_canvas()?;
        }
        Ok(movie)
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Interpreter steps the most recent frame took. Counted only when the
    /// budget sets a step limit (otherwise 0); a cheap measure of how heavy
    /// a frame is.
    pub fn last_frame_steps(&self) -> u64 {
        self.last_steps
    }

    /// Simulation ticks (or, for a persistent movie, painted frames) the
    /// most recent render had to apply to reach its moment: 0 for a pure
    /// movie. Rendering frames in order applies a frame's worth each time;
    /// jumping ahead or back applies everything in between.
    pub fn last_frame_ticks(&self) -> u64 {
        self.last_ticks
    }

    /// Record the strings inside `value`, which sits at `path`: those in
    /// arrays and dictionaries, and the literal strings in the bodies of
    /// procedures, which are one string object shared by every call (a
    /// frame that writes into `(    )` in a procedure changes it for good).
    /// A procedure's elements are written `path{i}`.
    fn collect_strings(
        &mut self,
        path: &str,
        value: &Value,
        depth: usize,
        out: &mut BTreeMap<String, u64>,
    ) {
        match value {
            Value::String(bytes) => {
                out.insert(path.to_string(), hash_bytes(bytes));
            }
            Value::Array(items) if depth < 8 => {
                for (i, v) in items.iter().enumerate() {
                    self.collect_strings(&format!("{path}[{i}]"), v, depth + 1, out);
                }
            }
            Value::Dict(entries) if depth < 8 => {
                for (k, v) in entries {
                    self.collect_strings(&format!("{path}/{k}"), v, depth + 1, out);
                }
            }
            Value::Other(_) if depth < 8 => {
                // An operator, a file or the like has no body: only an array
                // object (a procedure) does, and `aload` opens it.
                if self.interp.push_value(value).is_err() {
                    return;
                }
                let body = self.exec(
                    Phase::Setup,
                    "dup type dup /arraytype eq exch /packedarraytype eq or \
                     { [ exch aload pop ] } { pop null } ifelse",
                );
                let items = match (body, self.interp.pop_value()) {
                    (Ok(()), Ok(Value::Array(items))) => items,
                    _ => {
                        self.interp.clear_operands();
                        return;
                    }
                };
                for (i, v) in items.iter().enumerate() {
                    self.collect_strings(&format!("{path}{{{i}}}"), v, depth + 1, out);
                }
            }
            _ => {}
        }
    }

    /// Fingerprint the strings reachable from userdict, after rolling a
    /// pure movie back to its setup checkpoint (so what a frame defined is
    /// gone and what it wrote into a string is not). `None` for a stateful
    /// or persistent movie, which keeps state on purpose.
    ///
    /// Taken right after [`Movie::load`] and again after some frames, the
    /// two differ exactly in the strings a scene wrote into: see
    /// [`StringSnapshot::changed_since`].
    pub fn string_snapshot(&mut self) -> Result<Option<StringSnapshot>, Error> {
        if self.meta.mode != Mode::Pure {
            return Ok(None);
        }
        let phase = Phase::Setup;
        self.rollback(0, phase)?;
        self.interp.clear_operands();
        self.exec(phase, "[ userdict { pop } forall ]")?;
        let keys = self
            .interp
            .pop_value()
            .map_err(|_| Error::new(phase, "internal", "userdict's names could not be read"))?;
        let mut snap = StringSnapshot::default();
        for key in keys.as_array().unwrap_or(&[]) {
            let Some(name) = key.as_text() else { continue };
            if let Some(v) = self.interp.lookup(name) {
                self.collect_strings(name, &v, 0, &mut snap.strings);
            }
        }
        Ok(Some(snap))
    }

    /// Replace the value of signal `name` from now on: what a live host
    /// calls before each frame with the mouse position or an audio level.
    pub fn set_signal(&mut self, name: &str, value: f64) {
        self.ctx.borrow_mut().signals.set(name, value);
    }

    /// Add an event at movie time `t` to the track `name`, for
    /// `sinceevent` and `eventcount`: what a live host calls when the viewer
    /// clicks or presses a key. See [`Signals::push_event`].
    pub fn push_event(&mut self, name: &str, t: f64) {
        self.ctx.borrow_mut().signals.push_event(name, t);
    }

    /// Forget the events of the track `name`.
    pub fn clear_events(&mut self, name: &str) {
        self.ctx.borrow_mut().signals.clear_events(name);
    }

    /// Add a reading of signal `name` taken at movie time `t`, keeping the
    /// earlier ones: what a live dashboard host calls with each new value,
    /// so `signalago` and `signalseries` can read the recent past and the
    /// movie can draw smoothly between readings (`/name 1 signalago` is the
    /// value a second ago, interpolated).
    pub fn record_signal(&mut self, name: &str, t: f64, value: f64) {
        self.ctx.borrow_mut().signals.record(name, t, value);
    }

    /// Render frame `index` (time `index / fps`).
    pub fn render_frame(&mut self, index: u64) -> Result<Frame<'_>, Error> {
        let time = self.meta.time_of(index);
        self.render(index, time)
    }

    /// Render the moment `time` seconds in, which need not fall on a frame
    /// boundary. A live host calls this with its own clock.
    pub fn render_at(&mut self, time: f64) -> Result<Frame<'_>, Error> {
        let index = (time * self.meta.fps + 1e-9).floor().max(0.0) as u64;
        self.render(index, time)
    }

    fn render(&mut self, index: u64, time: f64) -> Result<Frame<'_>, Error> {
        let from_accum = match self.draw(index, time) {
            Ok(a) => a,
            Err(mut e) => {
                // What the frame printed before it failed is often the
                // clue: the error carries a copy (the movie keeps it too).
                e.printed = Printed {
                    text: self.output.clone(),
                    truncated: self.output_truncated,
                };
                e.text_warnings = self.text_warnings();
                return Err(e);
            }
        };
        let (width, height) = self.interp.canvas_size();
        Ok(Frame {
            index,
            time,
            width,
            height,
            premultiplied: if from_accum {
                &self.blurred
            } else {
                self.interp.canvas_premultiplied_rgba8()
            },
        })
    }

    /// Draw frame `index` at `time` onto the canvas; true when the frame
    /// is the motion-blur accumulation rather than the canvas.
    fn draw(&mut self, index: u64, time: f64) -> Result<bool, Error> {
        let phase = Phase::Frame { index, time };
        self.last_steps = 0;
        self.last_ticks = 0;
        // The frame shown last time may have been the painting plus an
        // overlay: put the painting itself back before anything else.
        if let Some(kept) = self.kept.take() {
            let _ = self.interp.swap_canvas(kept);
        }
        let mut from_accum = false;
        match self.meta.mode {
            Mode::Pure => match self.meta.motion_blur {
                None => self.draw_pure(index, time, phase)?,
                Some(mb) => {
                    let len = self.interp.canvas_premultiplied_rgba8().len();
                    self.accum.clear(len);
                    for s in 0..mb.samples {
                        let offset = (f64::from(s) + 0.5) / f64::from(mb.samples) * mb.shutter
                            / self.meta.fps;
                        self.draw_pure(index, time + offset, phase)?;
                        self.accum.add(self.interp.canvas_premultiplied_rgba8());
                    }
                    self.accum.resolve(&mut self.blurred);
                    from_accum = true;
                }
            },
            Mode::Stateful => {
                let ticks = last_step_at(time, self.meta.tick_rate).max(0) as u64;
                self.seek(ticks)?;
                // The drawing runs between a save and a restore of its own,
                // so it cannot change the simulation's state.
                self.exec(phase, "save")?;
                let guard = self.pop_save(phase)?;
                self.begin(Running::Frame, index as i64, time, 1.0 / self.meta.fps);
                self.ctx.borrow_mut().step = ticks as i64 - 1;
                let drawn = self
                    .exec(phase, &self.draw_program(true))
                    .and_then(|()| self.check_stack(phase, &self.scene.clone()));
                self.interp.clear_operands();
                self.push_save(phase, &guard)?;
                self.exec(phase, "cleardictstack restore")?;
                drawn?;
                self.check_layers(phase)?;
            }
            Mode::Persistent => {
                // Frame j of the declared rate is painted by tick j.
                let ticks = last_step_at(time, self.meta.tick_rate).max(0) as u64 + 1;
                self.seek(ticks)?;
                if let Some(overlay) = self.overlay.clone() {
                    // Draw the overlay on a copy; the painting is set aside.
                    let (w, h) = self.interp.canvas_size();
                    let copy = Page::from_premultiplied_rgba8(
                        w,
                        h,
                        self.interp.canvas_premultiplied_rgba8().to_vec(),
                    )
                    .ok_or_else(|| {
                        Error::new(phase, "internal", "the canvas could not be copied")
                    })?;
                    self.kept = self.interp.swap_canvas(copy).ok();
                    self.exec(phase, "cleardictstack save")?;
                    let guard = self.pop_save(phase)?;
                    self.begin(Running::Frame, index as i64, time, 1.0 / self.meta.fps);
                    {
                        let mut c = self.ctx.borrow_mut();
                        c.step = ticks as i64 - 1;
                        c.canvas = Canvas::Overlay;
                    }
                    let program = format!(
                        "clear cleardictstack grestoreall initgraphics {} srand {overlay}",
                        self.meta.seed
                    );
                    let drawn = self
                        .exec(phase, &program)
                        .and_then(|()| self.check_stack(phase, &overlay));
                    self.interp.clear_operands();
                    self.push_save(phase, &guard)?;
                    self.exec(phase, "cleardictstack restore")?;
                    drawn?;
                    self.check_layers(phase)?;
                }
            }
        }
        Ok(from_accum)
    }

    fn begin(&mut self, running: Running, frame: i64, t: f64, dt: f64) {
        let mut c = self.ctx.borrow_mut();
        c.running = running;
        c.begin_moment(frame, t, dt);
        // Layers left by a frame that failed are dropped; the frame about
        // to be drawn erases whatever canvas is current.
        c.layers.clear();
    }

    fn exec(&mut self, phase: Phase, program: &str) -> Result<(), Error> {
        let r = run(&mut self.interp, &self.ctx, phase, program);
        collect_byte_font_text(&self.interp, &mut self.byte_font_text, false);
        let printed = self.interp.captured_stdout();
        if !printed.is_empty() {
            push_output(&mut self.output, &mut self.output_truncated, printed);
            self.interp.capture_stdout();
        }
        if self.frame_limits.max_steps.is_some() {
            self.last_steps += self.interp.steps_used();
        }
        r
    }

    /// A scene or step is called on an empty operand stack and must leave
    /// it empty. Anything left is a bug in the program more often than not
    /// (an operand a later operator was meant to take), and it would
    /// otherwise pass unseen.
    fn check_stack(&mut self, phase: Phase, who: &str) -> Result<(), Error> {
        let left = self.interp.operand_count();
        if left == 0 {
            return Ok(());
        }
        let mut stack = Vec::new();
        while stack.len() < 6 {
            match self.interp.pop_value() {
                Ok(v) => stack.push(short_repr(&v)),
                Err(_) => break,
            }
        }
        stack.reverse();
        self.interp.clear_operands();
        let mut e = Error::new(
            phase,
            "stackleft",
            format!("`{who}` left {left} operand{} on the stack", if left == 1 { "" } else { "s" }),
        )
        .hint(format!(
            "`{who}` is called on an empty stack and must leave it empty. Something pushed a value nothing used: look for the values shown above. (The time is not passed as an operand; read it with `now`.)"
        ));
        e.stack = stack;
        Err(e)
    }

    /// What the program has printed (`print`, `=`, `==`, `pstack`) since
    /// the last call, as text. Printing is how a program is debugged:
    /// `psanim frame` shows it.
    ///
    /// Output is kept to 64 KB between calls; past that it is dropped and
    /// the text ends with `[output truncated at 64 KB]`. [`take_printed`]
    /// reports that as a flag too.
    ///
    /// [`take_printed`]: Movie::take_printed
    pub fn take_output(&mut self) -> String {
        self.take_printed().text
    }

    /// Like [`take_output`](Movie::take_output), saying whether the text
    /// was cut at 64 KB.
    /// What went wrong with text so far, one line each, for `check` and
    /// `frame` to report: a character no font had (PSCat drew its `.notdef`
    /// box), a font PSCat stood in for, and UTF-8 text shown in a font that
    /// reads bytes (stray accents or blanks). Each is said once for the
    /// movie's life, however many frames drew it; empty when all is well.
    pub fn text_warnings(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .interp
            .font_warnings()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.strip_prefix("pscat: warning: ").unwrap_or(l).to_string())
            .collect();
        for f in &self.byte_font_text.fonts {
            let chars: Vec<String> = f.characters.iter().map(char::to_string).collect();
            out.push(format!(
                "UTF-8 text ({}{}) shown in /{}, a font that reads one byte per character, so it draws stray accents or blanks{}. Select the font with `face`, or make it read UTF-8: /{} findfont unicodefont 24 scalefont setfont",
                chars.join(" "),
                if f.more { " ..." } else { "" },
                f.font,
                f.line.map(|l| format!(" (line {l})")).unwrap_or_default(),
                f.font
            ));
        }
        let more = self.byte_font_text.more_fonts();
        if more > 0 {
            out.push(format!(
                "UTF-8 text was also shown in at least {more} more font{} that read{} one byte per character (not named here)",
                if more == 1 { "" } else { "s" },
                if more == 1 { "s" } else { "" }
            ));
        }
        out
    }

    pub fn take_printed(&mut self) -> Printed {
        Printed {
            text: std::mem::take(&mut self.output),
            truncated: std::mem::take(&mut self.output_truncated),
        }
    }

    fn pop_save(&mut self, phase: Phase) -> Result<Value, Error> {
        self.interp
            .pop_value()
            .map_err(|_| Error::new(phase, "internal", "the checkpoint was lost"))
    }

    fn push_save(&mut self, phase: Phase, save: &Value) -> Result<(), Error> {
        self.interp
            .push_value(save)
            .map_err(|_| Error::new(phase, "internal", "the checkpoint could not be restored"))
    }

    fn check_layers(&mut self, phase: Phase) -> Result<(), Error> {
        let open = self.ctx.borrow().layers.len();
        if open == 0 {
            return Ok(());
        }
        // Put the frame's own canvas back so the next frame starts sound.
        let base = self.ctx.borrow_mut().layers.drain(..).next();
        if let Some(base) = base {
            let _ = self.interp.swap_canvas(base);
        }
        Err(Error::new(
            phase,
            "unclosedlayer",
            format!("{open} layer(s) opened with beginlayer were never closed"),
        )
        .hint(
            "every beginlayer needs an `alpha endlayer`; `alpha { ... } faded` pairs them for you",
        ))
    }

    /// The PostScript that draws one frame: a clean graphics state, the
    /// seed, optionally a fresh canvas, then the scene, on an empty operand
    /// stack. The scene reads the time with `now`.
    fn draw_program(&self, fresh_canvas: bool) -> String {
        let mut p = String::from("clear cleardictstack grestoreall initgraphics ");
        if fresh_canvas {
            p.push_str(&self.ground());
        }
        p.push_str(&format!("{} srand {}", self.meta.seed, self.scene));
        p
    }

    /// Erase to the declared background.
    fn ground(&self) -> String {
        match self.meta.background {
            Some([r, g, b]) => format!(
                "erasepage {r:?} {g:?} {b:?} setrgbcolor 0 0 {} {} rectfill 0 setgray ",
                self.meta.width, self.meta.height
            ),
            None => "erasepage ".to_string(),
        }
    }

    fn reset_canvas(&mut self) -> Result<(), Error> {
        let ground = format!("grestoreall initgraphics {}", self.ground());
        self.exec(Phase::Setup, &ground)
    }

    fn draw_pure(&mut self, index: u64, time: f64, phase: Phase) -> Result<(), Error> {
        self.begin(Running::Frame, index as i64, time, 1.0 / self.meta.fps);
        self.rollback(0, phase)?;
        let program = self.draw_program(true);
        self.exec(phase, &program)?;
        self.check_stack(phase, &self.scene.clone())?;
        self.check_layers(phase)
    }

    /// Roll the interpreter back to checkpoint `k`, discarding the ones
    /// after it, and take that checkpoint again (a save object is spent by
    /// its restore).
    fn rollback(&mut self, k: usize, phase: Phase) -> Result<(), Error> {
        self.checkpoints.truncate(k + 1);
        self.interp.clear_operands();
        let save = self.checkpoints[k].save.clone();
        self.push_save(phase, &save)?;
        self.exec(phase, "cleardictstack restore save")?;
        self.checkpoints[k].save = self.pop_save(phase)?;
        self.cursor = self.checkpoints[k].tick;
        Ok(())
    }

    /// Bring the simulation to exactly `target` ticks applied.
    fn seek(&mut self, target: u64) -> Result<(), Error> {
        if target < self.cursor {
            let k = self
                .checkpoints
                .iter()
                .rposition(|c| c.tick <= target)
                .unwrap_or(0);
            let phase = Phase::Step {
                tick: self.checkpoints[k].tick,
                time: 0.0,
            };
            self.rollback(k, phase)?;
            if self.meta.mode == Mode::Persistent {
                match self.checkpoints[k].canvas.clone() {
                    Some(pixels) => {
                        let (w, h) = self.interp.canvas_size();
                        let page =
                            Page::from_premultiplied_rgba8(w, h, pixels).ok_or_else(|| {
                                Error::new(
                                    phase,
                                    "internal",
                                    "a canvas checkpoint has the wrong size",
                                )
                            })?;
                        let _ = self.interp.swap_canvas(page);
                    }
                    None => self.reset_canvas()?,
                }
            }
        }
        while self.cursor < target {
            self.tick()?;
            self.last_ticks += 1;
        }
        Ok(())
    }

    /// Apply one tick: advance the simulation, and for a persistent movie
    /// paint that tick's frame over the canvas.
    fn tick(&mut self) -> Result<(), Error> {
        let tick = self.cursor;
        let dt = 1.0 / self.meta.tick_rate;
        let time = tick as f64 * dt;
        let phase = Phase::Step { tick, time };
        // Each tick gets its own seed: `rand` inside `step` is repeatable
        // on replay, and differs from tick to tick.
        let seed =
            (mix64(self.meta.seed as u64 ^ tick.wrapping_mul(0x9E37_79B9_7F4A_7C15)) >> 33) as i64;
        if let Some(step) = self.step.clone() {
            self.begin(Running::Step, tick as i64, time, dt);
            self.exec(
                phase,
                &format!("clear cleardictstack grestoreall initgraphics {seed} srand {step}"),
            )?;
            self.check_stack(phase, &step)?;
        }
        if self.meta.mode == Mode::Persistent {
            let phase = Phase::Frame { index: tick, time };
            self.exec(phase, "cleardictstack save")?;
            let guard = self.pop_save(phase)?;
            self.begin(Running::Frame, tick as i64, time, dt);
            self.ctx.borrow_mut().canvas = Canvas::Kept;
            let drawn = self
                .exec(phase, &self.draw_program(false))
                .and_then(|()| self.check_stack(phase, &self.scene.clone()));
            self.interp.clear_operands();
            self.push_save(phase, &guard)?;
            self.exec(phase, "cleardictstack restore")?;
            drawn?;
            self.check_layers(phase)?;
        }
        self.cursor += 1;
        if self.cursor.is_multiple_of(self.checkpoint_every) {
            self.interp.clear_operands();
            self.exec(phase, "cleardictstack grestoreall save")?;
            let save = self.pop_save(phase)?;
            let canvas = (self.meta.mode == Mode::Persistent)
                .then(|| self.interp.canvas_premultiplied_rgba8().to_vec());
            self.checkpoints.push(Checkpoint {
                tick: self.cursor,
                save,
                canvas,
            });
        }
        Ok(())
    }
}

/// Register the operators and apply the host's policy to a new interpreter.
fn configure(interp: &mut Interp, ctx: &Rc<RefCell<Ctx>>, options: &Options) -> Result<(), Error> {
    crate::ops::install(interp, ctx);
    interp.quiet_stderr();
    interp.capture_stdout();
    // UTF-8 shown in a byte font draws stray accents; say so (it changes
    // nothing drawn and no step count).
    interp.set_utf8_text_check(true);
    interp.set_interrupt(options.interrupt.clone());
    if let Some(roots) = &options.read_roots {
        interp
            .set_read_roots(roots)
            .map_err(|e| Error::new(Phase::Load, "options", format!("read root: {e}")))?;
    }
    Ok(())
}

fn host_params(params: &BTreeMap<String, Param>) -> BTreeMap<String, Value> {
    params
        .iter()
        .map(|(k, v)| {
            let v = match v {
                Param::Number(n) => Value::Real(*n),
                Param::Text(s) => Value::String(s.clone().into_bytes()),
            };
            (k.clone(), v)
        })
        .collect()
}

/// Run `program`, turning a PostScript error into an [`Error`] that says
/// where in the movie it happened.
/// The word is still the prelude's own procedure (saved by the prelude as
/// `.prelude-<name>`), not one the program put in its place.
fn is_prelude_word(interp: &mut Interp, name: &str) -> bool {
    let saved = format!(".prelude-{name}");
    if interp.lookup(&saved).is_none() {
        return false;
    }
    // systemdict on top, so the program's own `load` or `eq` are not used.
    let probe = format!("systemdict begin /{name} load /{saved} load eq end");
    let same =
        interp.run_str(&probe).is_ok() && matches!(interp.pop_value(), Ok(Value::Bool(true)));
    interp.clear_operands();
    same
}

fn run(
    interp: &mut Interp,
    ctx: &Rc<RefCell<Ctx>>,
    phase: Phase,
    program: &str,
) -> Result<(), Error> {
    let Err(err) = interp.run_str(program) else {
        return Ok(());
    };
    Err(describe(interp, ctx, phase, &err))
}

fn describe(interp: &mut Interp, ctx: &Rc<RefCell<Ctx>>, phase: Phase, err: &PsError) -> Error {
    let d = interp.error_details(err);
    let mut e = Error::new(phase, d.name, "");
    e.command = d.command.clone();
    e.limit = d.limit.map(|l| l.name());
    // Only setup runs program text directly; a frame's "line" would be a
    // line of the engine's own one-line driver.
    if phase == Phase::Setup {
        e.line = d.line;
        e.file = d.file;
    }
    e.message = match (&d.command, d.limit, &d.detail) {
        (_, Some(limit), _) => format!("the {} budget ran out ({limit})", phase.name()),
        (_, _, Some(detail)) => format!("syntax error: {detail}"),
        (Some(c), _, _) if d.name == "undefined" => format!("undefined name `{c}`"),
        (Some(c), _, _) => format!("{} in `{c}`", d.name),
        (None, _, _) => d.name.to_string(),
    };
    // The program's own procedures; the prelude's plumbing (a combinator's
    // `{ exec .timepop }`) would only be noise between them.
    e.backtrace = d
        .backtrace
        .iter()
        .filter(|f| {
            !f.excerpt.split_whitespace().any(|w| {
                let w = w.trim_start_matches(['{', '>', '-']);
                w.starts_with('.') && w[1..].starts_with(|c: char| c.is_ascii_alphabetic())
            }) && f
                .procedure
                .as_deref()
                .is_none_or(|n| crate::api::lookup(n).is_none())
        })
        .map(|f| crate::error::TraceLine {
            procedure: f.procedure.clone(),
            excerpt: f.excerpt.clone(),
        })
        .collect();
    // What the failing operator was looking at.
    let mut stack = Vec::new();
    while stack.len() < 6 {
        match interp.pop_value() {
            Ok(v) => stack.push(short_repr(&v)),
            Err(_) => break,
        }
    }
    stack.reverse();
    e.stack = stack;
    interp.clear_operands();
    e.hint = ctx
        .borrow_mut()
        .hint
        .take()
        .or_else(|| explain(d.name, d.command.as_deref()));
    e
}

/// What the probe pass found out.
struct Probed {
    /// The dictionary `animation` was given, if it was called.
    decl: Option<Value>,
    /// Seconds the program's code before `animation` took, when a clock
    /// was given and `animation` was called.
    seconds: Option<f64>,
    /// That code printed something. (The text is dropped with the
    /// probe's interpreter: the real pass prints it again, once.)
    printed: bool,
}

/// Run the program in a throwaway interpreter until `animation` is called,
/// and return the dictionary it was given. A program that never calls it
/// runs to its end here and gets the defaults.
fn probe(source: &str, options: &Options) -> Result<Probed, Error> {
    let mut interp = Interp::with_page(64, 64).expect("a small page");
    let ctx = Rc::new(RefCell::new(Ctx::new()));
    {
        let mut c = ctx.borrow_mut();
        c.probing = true;
        c.running = Running::Probe;
        c.params = host_params(&options.params);
        c.signals = options.signals.clone();
        c.base_dir = options.base_dir.clone();
        c.begin_moment(0, 0.0, 1.0 / 30.0);
    }
    configure(&mut interp, &ctx, options)?;
    interp
        .set_limits(options.budget.setup)
        .map_err(|e| Error::new(Phase::Load, "options", e.to_string()))?;
    run(&mut interp, &ctx, Phase::Setup, PRELUDE)?;
    let started = options.clock.map(|clock| clock());
    run(&mut interp, &ctx, Phase::Setup, source)?;
    let seconds = options
        .clock
        .zip(started)
        .map(|(clock, started)| clock() - started);
    let decl = ctx.borrow_mut().decl.take();
    let called = decl.is_some();
    Ok(Probed {
        seconds: seconds.filter(|_| called),
        printed: called && !interp.captured_stdout().is_empty(),
        decl,
    })
}

/// The `animation` dictionary, read and checked.
struct Declaration {
    title: Option<String>,
    description: Option<String>,
    width: u32,
    height: u32,
    fps: f64,
    /// The rate the program declared, before any override: a persistent
    /// movie paints at this rate whatever the output rate is.
    declared_fps: f64,
    duration: f64,
    seed: i64,
    background: Option<[f64; 3]>,
    looping: bool,
    cuts: Vec<f64>,
    scene: String,
    step: Option<String>,
    step_declared: bool,
    step_rate: f64,
    persist: bool,
    motion_blur: Option<MotionBlur>,
}

const DECLARATION_KEYS: &[&str] = &[
    "Size",
    "Duration",
    "FPS",
    "Seed",
    "Background",
    "Title",
    "Description",
    "Loop",
    "Cuts",
    "Scene",
    "Step",
    "StepRate",
    "Persist",
    "MotionBlur",
    "Shutter",
];

impl Declaration {
    fn read(decl: Option<&Value>) -> Result<Declaration, Error> {
        let bad = |key: &str, want: &str| {
            Error::new(
                Phase::Load,
                "declaration",
                format!("/{key} in the animation dictionary must be {want}"),
            )
        };
        let mut d = Declaration {
            title: None,
            description: None,
            width: 1280,
            height: 720,
            fps: 30.0,
            declared_fps: 30.0,
            duration: 5.0,
            seed: 1,
            background: Some([0.0, 0.0, 0.0]),
            looping: false,
            cuts: Vec::new(),
            scene: "scene".into(),
            step: Some("step".into()),
            step_declared: false,
            step_rate: 0.0,
            persist: false,
            motion_blur: None,
        };
        let entries: &[(String, Value)] = match decl {
            Some(Value::Dict(e)) => e,
            _ => &[],
        };
        let mut samples = None;
        let mut shutter = 0.5;
        for (key, v) in entries {
            match key.as_str() {
                "Size" => {
                    let size = v.as_numbers().filter(|s| s.len() == 2);
                    let ok = size.as_ref().is_some_and(|s| {
                        s.iter()
                            .all(|x| x.fract() == 0.0 && (1.0..=8000.0).contains(x))
                    });
                    let Some(s) = size.filter(|_| ok) else {
                        return Err(bad("Size", "[width height], whole numbers from 1 to 8000"));
                    };
                    (d.width, d.height) = (s[0] as u32, s[1] as u32);
                }
                "Duration" => {
                    d.duration = v
                        .as_f64()
                        .ok_or_else(|| bad("Duration", "a number of seconds"))?
                }
                "FPS" => d.fps = v.as_f64().ok_or_else(|| bad("FPS", "a number"))?,
                "Seed" => d.seed = v.as_i64().ok_or_else(|| bad("Seed", "an integer"))?,
                "Background" => {
                    d.background = match v {
                        Value::Null => None,
                        Value::Name(n) if n == "transparent" => None,
                        _ => {
                            let c = v.as_numbers().filter(|c| c.len() == 3).ok_or_else(|| {
                                bad(
                                    "Background",
                                    "[r g b] with components 0..1, or /transparent",
                                )
                            })?;
                            Some([
                                c[0].clamp(0.0, 1.0),
                                c[1].clamp(0.0, 1.0),
                                c[2].clamp(0.0, 1.0),
                            ])
                        }
                    }
                }
                "Title" => {
                    d.title = Some(
                        v.as_text()
                            .ok_or_else(|| bad("Title", "a string"))?
                            .to_string(),
                    )
                }
                "Description" => {
                    d.description = Some(
                        v.as_text()
                            .ok_or_else(|| bad("Description", "a string"))?
                            .to_string(),
                    );
                }
                "Loop" => d.looping = v.as_bool().ok_or_else(|| bad("Loop", "true or false"))?,
                "Cuts" => {
                    d.cuts = v
                        .as_numbers()
                        .filter(|c| c.iter().all(|t| t.is_finite() && *t >= 0.0))
                        .ok_or_else(|| {
                            bad("Cuts", "an array of times in seconds, like [4 12.5]")
                        })?;
                }
                "Persist" => {
                    d.persist = v.as_bool().ok_or_else(|| bad("Persist", "true or false"))?
                }
                "Scene" => match v {
                    Value::Name(n) => d.scene = n.clone(),
                    _ => {
                        return Err(bad(
                            "Scene",
                            "the name of the drawing procedure, like /scene",
                        ));
                    }
                },
                "Step" => match v {
                    Value::Name(n) => {
                        d.step = Some(n.clone());
                        d.step_declared = true;
                    }
                    Value::Null => d.step = None,
                    _ => {
                        return Err(bad(
                            "Step",
                            "the name of the simulation procedure, like /step",
                        ));
                    }
                },
                "StepRate" => {
                    d.step_rate = v
                        .as_f64()
                        .ok_or_else(|| bad("StepRate", "ticks per second"))?
                }
                "MotionBlur" => {
                    samples = Some(v.as_i64().filter(|n| (0..=64).contains(n)).ok_or_else(|| {
                        bad(
                            "MotionBlur",
                            "the number of samples per frame, 2 to 64 (0 for none)",
                        )
                    })? as u32);
                }
                "Shutter" => {
                    shutter = v
                        .as_f64()
                        .ok_or_else(|| bad("Shutter", "a fraction of the frame, 0..1"))?
                }
                other => {
                    let mut e = Error::new(
                        Phase::Load,
                        "declaration",
                        format!("the animation dictionary has an unknown key /{other}"),
                    );
                    e.hint = Some(match closest(other, DECLARATION_KEYS.iter().copied()) {
                        Some(near) => format!("did you mean /{near}?"),
                        None => format!(
                            "the keys are {}",
                            DECLARATION_KEYS
                                .iter()
                                .map(|k| format!("/{k}"))
                                .collect::<Vec<_>>()
                                .join(" ")
                        ),
                    });
                    return Err(e);
                }
            }
        }
        d.declared_fps = d.fps;
        if d.step_rate == 0.0 {
            d.step_rate = d.fps;
        }
        if !(d.step_rate.is_finite() && d.step_rate > 0.0 && d.step_rate <= 10_000.0) {
            return Err(bad("StepRate", "above 0 and at most 10000"));
        }
        if let Some(n) = samples.filter(|n| *n >= 2) {
            d.motion_blur = Some(MotionBlur {
                samples: n,
                shutter,
            });
        }
        Ok(d)
    }
}
