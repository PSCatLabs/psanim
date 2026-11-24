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

    /// Canvases set aside by `beginlayer`, outermost first.
    pub layers: Vec<Page>,
    /// What each open `faded`/`blended`/`blurred`/`glowing` will do when
    /// its body is done. Kept here, not on the operand stack, so the body
    /// runs with the caller's operands on top.
    pub pending: Vec<LayerEnd>,
    /// A canvas-sized page kept for operations that need the canvas out of
    /// the interpreter for a moment.
    pub spare: Option<Page>,
    pub scratch: Vec<u8>,

    pub params: BTreeMap<String, Value>,
    pub signals: Signals,
    /// Names `param` was asked for and the defaults given, for `info`.
    pub params_seen: BTreeMap<String, String>,

    /// A plain-language explanation an operator leaves before it fails,
    /// picked up by the engine for the error report.
    pub hint: Option<String>,
}

/// How a layer combinator finishes its layer.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LayerEnd {
    pub opacity: f64,
    pub blend: crate::raster::Blend,
    /// Blur radius in canvas units, 0 for none.
    pub blur: f64,
    /// Glow radius and strength, strength 0 for none.
    pub glow: (f64, f64),
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
            layers: Vec::new(),
            pending: Vec::new(),
            spare: None,
            scratch: Vec::new(),
            params: BTreeMap::new(),
            signals: Signals::default(),
            params_seen: BTreeMap::new(),
            hint: None,
        }
    }

    /// The innermost time scope: local time and window.
    pub fn scope(&self) -> TimeScope {
        self.scopes.last().copied().unwrap_or(TimeScope {
            t: self.global_t,
            dur: self.duration,
        })
    }

    /// Start a moment: global time `t`, the scope stack holding just the
    /// movie's own window.
    pub fn begin_moment(&mut self, frame: i64, t: f64, dt: f64) {
        self.frame = frame;
        self.global_t = t;
        self.dt = dt;
        self.scopes.clear();
        self.scopes.push(TimeScope {
            t,
            dur: self.duration,
        });
        self.pending.clear();
        self.hint = None;
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
