//! The state the host operators share with the engine: the clock, the time
//! scope stack, open layers, parameters and signals.

use std::collections::BTreeMap;

use pscat::Page;
use pscat::host::Value;

use crate::signals::Signals;

/// One entry of the time scope stack: the local time in seconds, and the
/// length of the window that time runs through (what `progress` divides by).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TimeScope {
    pub t: f64,
    pub dur: f64,
    /// The period that `turns` and `loopnoise` close over. It is the window
    /// except under `delayed`, which only shifts the clock: there the
    /// window shortens but the loop stays the loop it was.
    pub cycle: f64,
}

impl TimeScope {
    /// A scope whose loop is its window.
    pub fn window(t: f64, dur: f64) -> Self {
        TimeScope { t, dur, cycle: dur }
    }

    /// Local time as a fraction of the loop: 1 is one whole period.
    pub fn cycle_fraction(&self) -> f64 {
        if self.cycle > 0.0 && self.cycle.is_finite() {
            self.t / self.cycle
        } else {
            0.0
        }
    }
}

/// What the drawing of a moment lands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Canvas {
    /// Nothing is kept: a pure or stateful movie's frame, or setup.
    Fresh,
    /// A persistent movie's scene: what it paints stays.
    Kept,
    /// A persistent movie's overlay: shown over the kept canvas, never kept.
    Overlay,
}

/// The last step run at movie time `t` for steps at `rate`: the engine's
/// own rule, so the words that report it and the seek agree.
pub(crate) fn last_step_at(t: f64, rate: f64) -> i64 {
    (t * rate + 1e-9).floor() as i64
}

/// The step on which movie time `t` is reached: the first whose time is
/// at or after `t`, a `t` within a millionth of a step after a step's time
/// counting as that step's.
pub(crate) fn step_reaching(t: f64, rate: f64) -> i64 {
    (t * rate - 1e-6).ceil() as i64
}

/// What the engine is running, for error reports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Running {
    Probe,
    Setup,
    Frame,
    Step,
}

pub(crate) struct Ctx {
    pub running: Running,
    /// True while the program is run only to find its `animation`
    /// declaration: `animation` then stops it.
    pub probing: bool,
    /// The dictionary given to `animation`, once it has run.
    pub decl: Option<Value>,

    // Movie-level facts the operators report.
    pub width: f64,
    pub height: f64,
    pub fps: f64,
    pub duration: f64,
    pub seed: i64,
    /// Device pixels per canvas unit.
    pub px_scale: f64,

    // The moment being rendered.
    pub frame: i64,
    pub global_t: f64,
    pub dt: f64,
    pub scopes: Vec<TimeScope>,
    /// The rate the movie steps at: for a persistent movie the declared
    /// FPS (its canvas is painted a step a frame of that rate whatever
    /// rate it is shown at), for a simulation its step rate, otherwise the
    /// frame rate.
    pub step_rate: f64,
    /// While a movie that is not persistent runs its setup: the name of
    /// its simulation procedure and the rate it would step at. Whether it
    /// is a simulation is known only once that procedure is defined, so
    /// `steprate`, `stepat` and `dt` in setup ask then.
    pub setup_sim: Option<(String, f64)>,
    /// The last step run at this moment (-1 before the first): in a
    /// persistent movie's scene the step being painted, in its overlay the
    /// last step the kept canvas holds; in a pure movie the frame.
    pub step: i64,
    /// What the moment draws on: a canvas that is not kept, a persistent
    /// movie's kept canvas (its scene) or its overlay.
    pub canvas: Canvas,

    /// Canvases set aside by `beginlayer`, outermost first.
    pub layers: Vec<Page>,
    /// What each open `faded`/`blended`/`blurred`/`glowing` will do when
    /// its body is done. Kept here, not on the operand stack, so the body
    /// runs with the caller's operands on top.
    pub pending: Vec<LayerEnd>,
    /// Each `faded` / `blended` running its body as a transparency group
    /// (because one was open around it), innermost last.
    pub fades: Vec<FadeGroup>,
    /// The pose each open motion will place its layer under.
    pub poses: Vec<crate::motion::Pose>,
    /// Viewports opened by `within` and `fitted`: the width and height the
    /// program is told, innermost last.
    pub views: Vec<(f64, f64)>,
    /// A canvas-sized page kept for operations that need the canvas out of
    /// the interpreter for a moment.
    pub spare: Option<Page>,
    pub scratch: Vec<u8>,
    /// The clip `.clipmask` just captured, for the effect that asked for
    /// it: `Some(None)` when nothing is clipped away.
    pub clip_taken: Option<Option<ClipMask>>,
    /// Alpha buffers of finished clip masks, kept to be reused.
    pub mask_pool: Vec<Vec<u8>>,

    pub params: BTreeMap<String, Value>,
    pub signals: Signals,
    /// Names `param` was asked for and the defaults given, for `info`.
    pub params_seen: BTreeMap<String, String>,
    /// Reels `reelduration` measured while the program's top level ran, with
    /// the length it reported. Checked once the program is fully defined: a
    /// custom transition defined after the measurement was counted as a clip.
    pub measured_reels: Vec<(Vec<Value>, f64)>,

    /// Components being set up or drawn, innermost last. Inside one, the
    /// movie-level facts (`globaltime`, `duration`, `movieseed`, `param`)
    /// are the component's own.
    pub comps: Vec<Comp>,
    /// Gust cells integrated so far this moment, across every particle and
    /// trail: native work the interpreter's budgets cannot see, so it is
    /// capped here (see `particles::charge_gust`).
    pub gust_work: f64,
    /// Cell updates the cellular automaton words did this moment (see
    /// `ca::MAX_WORK`).
    pub cell_work: f64,
    /// States `cellsteps` has computed, kept across frames: the answer is
    /// a function of the operands alone, the cache only saves the work.
    pub cells: crate::ca::Cache,
    /// The particles of each open `particles` call, innermost last.
    pub particles: Vec<crate::particles::ParticleSet>,
    /// Planes opened by `onplane`, innermost last: where the plane is, and
    /// the CTM (as `currentmatrix` gives it) the plane's coordinates are in.
    pub planes: Vec<(crate::plane::Plane, [f64; 6])>,
    /// The next id `storyboard` gives a transition between beats.
    pub story_ids: i64,
    /// For each storyboard transition drawn in this moment: its id, and how
    /// many times its outgoing and its incoming side were drawn, so a
    /// side drawn out of order is an error rather than a wrong picture.
    pub beat_sides: Vec<(i64, u32, u32)>,
    /// The directory relative component paths are read from: the movie
    /// file's own, when the host knows it.
    pub base_dir: Option<std::path::PathBuf>,

    /// A plain-language explanation an operator leaves before it fails,
    /// picked up by the engine for the error report.
    pub hint: Option<String>,
    /// psanim's own kind and message for the error an operator is about
    /// to raise, in place of the PostScript error's name (`groupnesting`
    /// says more than `rangecheck`).
    pub failure: Option<(&'static str, String)>,
    /// The command the error that left `hint` / `failure` names (the
    /// operator, or the missing name of an `undefined`): they describe an
    /// error only if it is the one that escapes (a `stopped` may have
    /// caught theirs).
    pub hint_op: Option<std::rc::Rc<str>>,
}

/// One open component: its own clock, length, seed and parameters.
#[derive(Clone, Debug)]
pub(crate) struct Comp {
    /// The component's own movie time, seconds from its start.
    pub t: f64,
    pub dur: f64,
    pub seed: i64,
    /// The values its `param` lookups get; a name not here gets the default.
    pub params: Vec<(String, Value)>,
    /// How many of the movie's units one of its units is where it is
    /// drawn: blur and glow radii, given in its units, are scaled by this.
    pub scale: f64,
    /// Its box on the canvas, in pixels (x0, y0, x1, y1), when it is placed:
    /// blur, glow, grain and vignette inside it stay inside it.
    pub rect: Option<[usize; 4]>,
}

pub(crate) const MAX_COMPS: usize = 16;

/// The clip in force when a blur or glow began, as coverage: what the
/// spread of the effect is confined to. Captured by painting through the
/// interpreter's own clip, so a rotated, curved or nested clip and its
/// anti-aliased edge are honoured exactly.
#[derive(Clone, Debug)]
pub(crate) struct ClipMask {
    /// One byte of coverage per canvas pixel, 0 (clipped away) to 255.
    pub alpha: Vec<u8>,
    /// The pixels with any coverage (x0, y0, x1, y1); outside it, nothing.
    pub support: [usize; 4],
}

/// How a layer combinator finishes its layer.
#[derive(Clone, Debug)]
pub(crate) struct LayerEnd {
    pub opacity: f64,
    pub blend: crate::raster::Blend,
    /// Blur radius in canvas units, 0 for none.
    pub blur: f64,
    /// Glow radius and strength, strength 0 for none.
    pub glow: (f64, f64),
    /// For a blur or glow, the clip in force when it began, if it clips.
    pub clip: Option<ClipMask>,
    /// The word that began it, for an error: `faded`, `blurred`, ...
    pub who: &'static str,
}

/// A `faded` or `blended` run as a transparency group: who it is, and the
/// group it opened, to check at its end that the body left that group the
/// innermost one (see `crate::groups`).
#[derive(Clone, Debug)]
pub(crate) struct FadeGroup {
    pub who: &'static str,
    /// Groups open, and the innermost, once its own group was open.
    pub opened: Option<(usize, Option<crate::groups::GroupId>)>,
}

pub(crate) const MAX_SCOPES: usize = 256;
pub(crate) const MAX_LAYERS: usize = 16;

impl Ctx {
    pub fn new() -> Ctx {
        Ctx {
            running: Running::Setup,
            probing: false,
            decl: None,
            width: 1280.0,
            height: 720.0,
            fps: 30.0,
            duration: 5.0,
            seed: 1,
            px_scale: 1.0,
            frame: 0,
            global_t: 0.0,
            dt: 1.0 / 30.0,
            scopes: Vec::new(),
            step_rate: 30.0,
            setup_sim: None,
            step: 0,
            canvas: Canvas::Fresh,
            layers: Vec::new(),
            pending: Vec::new(),
            fades: Vec::new(),
            poses: Vec::new(),
            views: Vec::new(),
            spare: None,
            scratch: Vec::new(),
            clip_taken: None,
            mask_pool: Vec::new(),
            params: BTreeMap::new(),
            signals: Signals::default(),
            params_seen: BTreeMap::new(),
            comps: Vec::new(),
            particles: Vec::new(),
            gust_work: 0.0,
            cell_work: 0.0,
            cells: crate::ca::Cache::default(),
            planes: Vec::new(),
            story_ids: 0,
            beat_sides: Vec::new(),
            base_dir: None,
            measured_reels: Vec::new(),
            hint: None,
            failure: None,
            hint_op: None,
        }
    }

    /// The seed randomness draws from: the innermost component's, or the
    /// movie's.
    pub fn seed(&self) -> i64 {
        self.comps.last().map_or(self.seed, |c| c.seed)
    }

    /// The size of the running program's unit in the movie's units: 1,
    /// or inside a component, its placed scale.
    pub fn unit_scale(&self) -> f64 {
        self.comps.last().map_or(1.0, |c| c.scale)
    }

    /// The pixels effects may touch: the innermost component's box, or
    /// the whole canvas.
    pub fn effect_rect(&self, w: usize, h: usize) -> [usize; 4] {
        let mut r = [0, 0, w, h];
        for c in &self.comps {
            if let Some(b) = c.rect {
                r = [
                    r[0].max(b[0]),
                    r[1].max(b[1]),
                    r[2].min(b[2]),
                    r[3].min(b[3]),
                ];
            }
        }
        if r[2] < r[0] {
            r[2] = r[0];
        }
        if r[3] < r[1] {
            r[3] = r[1];
        }
        r
    }

    /// Keep a finished mask's buffer for the next one (a few at most).
    pub fn recycle_mask(&mut self, m: ClipMask) {
        if self.mask_pool.len() <= MAX_LAYERS {
            self.mask_pool.push(m.alpha);
        }
    }

    /// Movie time and length as the running program sees them: inside a
    /// component, its own.
    pub fn movie_clock(&self) -> TimeScope {
        self.comps
            .last()
            .map_or(TimeScope::window(self.global_t, self.duration), |c| {
                TimeScope::window(c.t, c.dur)
            })
    }

    /// The clock steps are counted on, as the running program sees it:
    /// (movie time, step rate, last step reached, canvas, inside a
    /// component). Inside a component the time is the component's own and
    /// the step is reached on it at the movie's step rate (a component has
    /// no kept canvas: its frames are drawn whole), so a delayed, sped-up
    /// or looping instance counts its own steps; its canvas is `Fresh`,
    /// unless it is being drawn into a persistent movie's kept canvas.
    pub fn step_view(&self) -> (f64, f64, i64, Canvas, bool) {
        match self.comps.last() {
            Some(c) => (
                c.t,
                self.step_rate,
                last_step_at(c.t, self.step_rate),
                if self.canvas == Canvas::Kept {
                    Canvas::Kept
                } else {
                    Canvas::Fresh
                },
                true,
            ),
            None => (self.global_t, self.step_rate, self.step, self.canvas, false),
        }
    }

    /// The innermost time scope: local time and window.
    pub fn scope(&self) -> TimeScope {
        self.scopes
            .last()
            .copied()
            .unwrap_or(TimeScope::window(self.global_t, self.duration))
    }

    /// Start a moment: global time `t`, the scope stack holding just the
    /// movie's own window.
    pub fn begin_moment(&mut self, frame: i64, t: f64, dt: f64) {
        self.frame = frame;
        self.global_t = t;
        self.dt = dt;
        self.step = frame;
        self.canvas = Canvas::Fresh;
        self.scopes.clear();
        self.scopes.push(TimeScope::window(t, self.duration));
        if let Some(m) = self.clip_taken.take().flatten() {
            self.recycle_mask(m);
        }
        let ends: Vec<LayerEnd> = self.pending.drain(..).collect();
        for m in ends.into_iter().filter_map(|e| e.clip) {
            self.recycle_mask(m);
        }
        self.poses.clear();
        self.fades.clear();
        self.views.clear();
        self.comps.clear();
        self.particles.clear();
        self.gust_work = 0.0;
        self.cell_work = 0.0;
        self.planes.clear();
        self.beat_sides.clear();
        self.hint = None;
        self.failure = None;
        self.hint_op = None;
    }
}

/// A 64-bit mix (splitmix64's finalizer): the one source of the engine's
/// stateless randomness.
pub(crate) fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A uniform value in `[0, 1)` determined by `seed` and `keys` alone.
pub(crate) fn hash01(seed: i64, keys: &[f64]) -> f64 {
    let mut h = mix64(seed as u64);
    for k in keys {
        // -0.0 and 0.0 are the same key.
        let k = if *k == 0.0 { 0.0 } else { *k };
        h = mix64(h ^ k.to_bits());
    }
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Smooth one-dimensional value noise in `[-1, 1]`: two octaves.
pub(crate) fn wiggle(seed: i64, key: f64, x: f64) -> f64 {
    let octave = |x: f64, salt: f64| {
        let i = x.floor();
        let f = x - i;
        let a = hash01(seed, &[key, salt, i]) * 2.0 - 1.0;
        let b = hash01(seed, &[key, salt, i + 1.0]) * 2.0 - 1.0;
        let s = f * f * (3.0 - 2.0 * f);
        a + (b - a) * s
    };
    (octave(x, 0.0) + 0.5 * octave(x * 2.03 + 17.0, 1.0)) / 1.5
}

/// Smooth two-dimensional value noise in `[-1, 1]`: two octaves. Sampled
/// on a circle it gives a wandering value that returns to its start.
pub(crate) fn noise2(seed: i64, key: f64, x: f64, y: f64) -> f64 {
    let octave = |x: f64, y: f64, salt: f64| {
        let (ix, iy) = (x.floor(), y.floor());
        let (fx, fy) = (x - ix, y - iy);
        let at = |dx: f64, dy: f64| hash01(seed, &[key, salt, ix + dx, iy + dy]) * 2.0 - 1.0;
        let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
        let top = at(0.0, 0.0) + (at(1.0, 0.0) - at(0.0, 0.0)) * sx;
        let bottom = at(0.0, 1.0) + (at(1.0, 1.0) - at(0.0, 1.0)) * sx;
        top + (bottom - top) * sy
    };
    (octave(x, y, 0.0) + 0.5 * octave(x * 2.03 + 17.0, y * 2.03 - 5.0, 1.0)) / 1.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash01_is_uniform_enough_and_stable() {
        let n = 20_000;
        let mut sum = 0.0;
        let mut buckets = [0u32; 10];
        for i in 0..n {
            let v = hash01(7, &[f64::from(i)]);
            assert!((0.0..1.0).contains(&v));
            sum += v;
            buckets[(v * 10.0) as usize] += 1;
        }
        assert!((sum / f64::from(n) - 0.5).abs() < 0.01);
        assert!(
            buckets.iter().all(|b| (1800..2200).contains(b)),
            "{buckets:?}"
        );
        // Pinned: renders depend on these exact values staying put.
        assert_eq!(hash01(1, &[0.0]), hash01(1, &[-0.0]));
        assert_ne!(hash01(1, &[1.0]), hash01(2, &[1.0]));
        assert_ne!(hash01(1, &[1.0, 2.0]), hash01(1, &[2.0, 1.0]));
        assert_eq!(format!("{:.12}", hash01(1, &[42.0])), "0.032946017316");
    }

    #[test]
    fn wiggle_is_bounded_and_continuous() {
        let mut last = wiggle(3, 0.0, 0.0);
        for i in 1..5000 {
            let v = wiggle(3, 0.0, f64::from(i) * 0.01);
            assert!((-1.0..=1.0).contains(&v));
            assert!((v - last).abs() < 0.08, "jump at {i}: {last} -> {v}");
            last = v;
        }
        assert_ne!(wiggle(3, 0.0, 0.37), wiggle(3, 1.0, 0.37));
    }
}
