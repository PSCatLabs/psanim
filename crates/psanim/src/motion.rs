//! Motions: how a drawing is partly present.
//!
//! A motion maps one number `u` to a [`Pose`]: an offset, a scale, a turn,
//! an opacity, a blur and a reveal. An entrance runs `u` from 0 (absent) to
//! 1 (in place); an exit runs it back; an emphasis runs 0 to 1 and is in
//! place at both ends. The engine draws the body on a layer, finds what was
//! painted, and composites that under the pose, so a motion needs to know
//! nothing about what it moves: "scale about its own centre" is measured,
//! not declared.

use pscat::host::Value;

/// Where a drawing is, relative to its resting place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// Offset in canvas units, y up.
    pub dx: f64,
    pub dy: f64,
    /// Offset in multiples of the drawing's own width and height.
    pub dxrel: f64,
    pub dyrel: f64,
    pub sx: f64,
    pub sy: f64,
    /// Degrees, counter-clockwise.
    pub rot: f64,
    pub alpha: f64,
    /// Blur radius in canvas units.
    pub blur: f64,
    /// The part of the drawing's own box that is shown, as fractions
    /// `[left, bottom, right, top]`.
    pub wipe: [f64; 4],
    /// Clip to the drawing's resting box: what moves in from outside it
    /// appears at its edge.
    pub mask: bool,
    /// The fixed point of scale and turn, as fractions of the drawing's box
    /// (0.5 0.5 is its centre, 0.5 0 the middle of its bottom edge).
    pub pivot: (f64, f64),
}

pub const REST: Pose = Pose {
    dx: 0.0,
    dy: 0.0,
    dxrel: 0.0,
    dyrel: 0.0,
    sx: 1.0,
    sy: 1.0,
    rot: 0.0,
    alpha: 1.0,
    blur: 0.0,
    wipe: [0.0, 0.0, 1.0, 1.0],
    mask: false,
    pivot: (0.5, 0.5),
};

impl Pose {
    /// Nothing would be drawn.
    pub fn absent(&self) -> bool {
        self.alpha <= 0.002
            || self.sx.abs() < 1e-4
            || self.sy.abs() < 1e-4
            || self.wipe[2] - self.wipe[0] <= 1e-4
            || self.wipe[3] - self.wipe[1] <= 1e-4
    }

    /// The drawing is exactly where it rests: no layer is needed.
    pub fn at_rest(&self) -> bool {
        let near = |a: f64, b: f64| (a - b).abs() < 1e-4;
        near(self.dx, 0.0)
            && near(self.dy, 0.0)
            && near(self.dxrel, 0.0)
            && near(self.dyrel, 0.0)
            && near(self.sx, 1.0)
            && near(self.sy, 1.0)
            && near(self.rot, 0.0)
            && self.alpha >= 0.998
            && self.blur < 1e-3
            && self.wipe[0] <= 1e-4
            && self.wipe[1] <= 1e-4
            && self.wipe[2] >= 1.0 - 1e-4
            && self.wipe[3] >= 1.0 - 1e-4
    }

    /// A pose from a dictionary a program's own motion returned. Unknown
    /// keys are an error naming the key, so a misspelling is not silent.
    pub fn from_dict(entries: &[(String, Value)]) -> Result<Pose, String> {
        let mut p = REST;
        for (k, v) in entries {
            let num = || {
                v.as_f64()
                    .ok_or_else(|| format!("motion: /{k} must be a number"))
            };
            let nums = |n: usize| match v {
                Value::Array(a) if a.len() == n => a
                    .iter()
                    .map(Value::as_f64)
                    .collect::<Option<Vec<f64>>>()
                    .ok_or_else(|| format!("motion: /{k} must hold numbers")),
                _ => Err(format!("motion: /{k} must be an array of {n} numbers")),
            };
            match k.as_str() {
                "dx" => p.dx = num()?,
                "dy" => p.dy = num()?,
                "dxrel" => p.dxrel = num()?,
                "dyrel" => p.dyrel = num()?,
                "scale" => {
                    p.sx = num()?;
                    p.sy = p.sx;
                }
                "sx" => p.sx = num()?,
                "sy" => p.sy = num()?,
                "rotate" => p.rot = num()?,
                "alpha" => p.alpha = num()?.clamp(0.0, 1.0),
                "blur" => p.blur = num()?.max(0.0),
                "wipe" => {
                    let a = nums(4)?;
                    p.wipe = [a[0], a[1], a[2], a[3]];
                }
                "mask" => p.mask = matches!(v, Value::Bool(true)),
                "pivot" => {
                    let a = nums(2)?;
                    p.pivot = (a[0], a[1]);
                }
                other => {
                    return Err(format!(
                        "motion: unknown key /{other}. Known: /dx /dy /dxrel /dyrel /scale /sx /sy /rotate /alpha /blur /wipe /mask /pivot"
                    ));
                }
            }
        }
        Ok(p)
    }
}

/// What kind of thing a motion is, which decides its defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// 0 is absent, 1 is in place.
    Entrance,
    /// In place at 0 and at 1.
    Emphasis,
}

pub struct Motion {
    pub name: &'static str,
    pub kind: Kind,
    /// Seconds, when the program does not say.
    pub dur: f64,
    /// Easing, when the program does not say.
    pub ease: &'static str,
    pub what: &'static str,
}

const fn m(
    name: &'static str,
    kind: Kind,
    dur: f64,
    ease: &'static str,
    what: &'static str,
) -> Motion {
    Motion {
        name,
        kind,
        dur,
        ease,
        what,
    }
}

use Kind::{Emphasis, Entrance};

/// The built-in motions. A program adds its own in the `motions`
/// dictionary.
pub const MOTIONS: &[Motion] = &[
    m("fade", Entrance, 0.5, "cubic-out", "opacity only"),
    m(
        "rise",
        Entrance,
        0.6,
        "cubic-out",
        "comes up into place while fading in",
    ),
    m(
        "fall",
        Entrance,
        0.6,
        "cubic-out",
        "comes down into place while fading in",
    ),
    m(
        "slide-left",
        Entrance,
        0.6,
        "cubic-out",
        "arrives from the right, moving left",
    ),
    m(
        "slide-right",
        Entrance,
        0.6,
        "cubic-out",
        "arrives from the left, moving right",
    ),
    m(
        "pop",
        Entrance,
        0.5,
        "back-out",
        "scales up from nothing past its size and settles",
    ),
    m(
        "grow",
        Entrance,
        0.5,
        "cubic-out",
        "scales up from nothing about its centre",
    ),
    m(
        "zoom",
        Entrance,
        0.6,
        "cubic-out",
        "settles from larger than life while fading in",
    ),
    m(
        "drop",
        Entrance,
        0.9,
        "bounce-out",
        "falls from above and bounces",
    ),
    m(
        "focus",
        Entrance,
        0.7,
        "cubic-out",
        "comes into focus from a blur",
    ),
    m(
        "spin",
        Entrance,
        0.7,
        "cubic-out",
        "turns half a turn into place while growing",
    ),
    m(
        "flip",
        Entrance,
        0.5,
        "cubic-out",
        "opens from edge-on about its vertical axis",
    ),
    m(
        "unfold",
        Entrance,
        0.5,
        "cubic-out",
        "grows upward from its bottom edge (bars, columns)",
    ),
    m(
        "unroll",
        Entrance,
        0.5,
        "cubic-out",
        "grows rightward from its left edge (rules, progress bars)",
    ),
    m(
        "wipe-right",
        Entrance,
        0.6,
        "cubic-inout",
        "revealed from its left edge to its right",
    ),
    m(
        "wipe-left",
        Entrance,
        0.6,
        "cubic-inout",
        "revealed from its right edge to its left",
    ),
    m(
        "wipe-up",
        Entrance,
        0.6,
        "cubic-inout",
        "revealed from its bottom edge to its top",
    ),
    m(
        "wipe-down",
        Entrance,
        0.6,
        "cubic-inout",
        "revealed from its top edge to its bottom",
    ),
    m(
        "mask-up",
        Entrance,
        0.7,
        "expo-out",
        "rises into its own box from below it, cut off at the box's edge (the kinetic-type line reveal)",
    ),
    m(
        "mask-down",
        Entrance,
        0.7,
        "expo-out",
        "descends into its own box from above it",
    ),
    m(
        "mask-right",
        Entrance,
        0.7,
        "expo-out",
        "slides into its own box from its left",
    ),
    m("pulse", Emphasis, 0.6, "linear", "swells and returns"),
    m(
        "shake",
        Emphasis,
        0.6,
        "linear",
        "shakes sideways, dying away",
    ),
    m(
        "wobble",
        Emphasis,
        0.8,
        "linear",
        "rocks about its centre, dying away",
    ),
    m(
        "jump",
        Emphasis,
        0.6,
        "linear",
        "hops up and lands, stretching on the way",
    ),
    m(
        "squash",
        Emphasis,
        0.5,
        "linear",
        "flattens against its bottom edge and recovers",
    ),
    m("blink", Emphasis, 0.6, "linear", "dips in opacity twice"),
    m("tada", Emphasis, 0.9, "linear", "swells and rattles"),
    m("dim", Emphasis, 1.0, "linear", "fades down and back"),
];

pub fn find(name: &str) -> Option<&'static Motion> {
    MOTIONS.iter().find(|m| m.name == name)
}

/// The pose of a built-in motion at `u`. `unit` is one hundredth of the
/// canvas's shorter side in canvas units, so distances suit any size.
pub fn pose(name: &str, u: f64, unit: f64) -> Option<Pose> {
    use std::f64::consts::PI;
    let away = 1.0 - u;
    let c = |x: f64| x.clamp(0.0, 1.0);
    let bump = (PI * c(u)).sin();
    let mut p = REST;
    match name {
        "fade" => p.alpha = c(u),
        "rise" => {
            p.dy = -away * 7.0 * unit;
            p.alpha = c(u * 1.6);
        }
        "fall" => {
            p.dy = away * 7.0 * unit;
            p.alpha = c(u * 1.6);
        }
        "slide-left" => {
            p.dx = away * 14.0 * unit;
            p.alpha = c(u * 1.6);
        }
        "slide-right" => {
            p.dx = -away * 14.0 * unit;
            p.alpha = c(u * 1.6);
        }
        "pop" => {
            p.sx = u.max(0.0);
            p.sy = p.sx;
            p.alpha = c(u * 3.0);
        }
        "grow" => {
            p.sx = c(u);
            p.sy = p.sx;
        }
        "zoom" => {
            p.sx = 1.0 + away * 0.7;
            p.sy = p.sx;
            p.alpha = c(u);
        }
        "drop" => {
            p.dy = away * 45.0 * unit;
            p.alpha = c(u * 6.0);
        }
        "focus" => {
            p.blur = c(away) * 3.0 * unit;
            p.alpha = c(u * 1.3);
        }
        "spin" => {
            p.rot = -away * 180.0;
            p.sx = c(u);
            p.sy = p.sx;
            p.alpha = c(u * 2.0);
        }
        "flip" => {
            p.sx = c(u);
            p.alpha = c(u * 3.0);
        }
        "unfold" => {
            p.sy = c(u);
            p.pivot = (0.5, 0.0);
        }
        "unroll" => {
            p.sx = c(u);
            p.pivot = (0.0, 0.5);
        }
        "wipe-right" => p.wipe = [0.0, 0.0, c(u), 1.0],
        "wipe-left" => p.wipe = [1.0 - c(u), 0.0, 1.0, 1.0],
        "wipe-up" => p.wipe = [0.0, 0.0, 1.0, c(u)],
        "wipe-down" => p.wipe = [0.0, 1.0 - c(u), 1.0, 1.0],
        "mask-up" => {
            p.dyrel = -c(away) * 1.05;
            p.mask = true;
            if u <= 0.0 {
                p.alpha = 0.0;
            }
        }
        "mask-down" => {
            p.dyrel = c(away) * 1.05;
            p.mask = true;
            if u <= 0.0 {
                p.alpha = 0.0;
            }
        }
        "mask-right" => {
            p.dxrel = -c(away) * 1.05;
            p.mask = true;
            if u <= 0.0 {
                p.alpha = 0.0;
            }
        }
        "pulse" => {
            p.sx = 1.0 + 0.14 * bump;
            p.sy = p.sx;
        }
        "shake" => p.dx = (u * 6.0 * PI).sin() * c(away) * 2.2 * unit,
        "wobble" => p.rot = (u * 4.0 * PI).sin() * c(away) * 7.0,
        "jump" => {
            p.dy = 4.0 * c(u) * c(away) * 6.0 * unit;
            p.sy = 1.0 + 0.12 * (2.0 * PI * c(u)).sin().abs() * (1.0 - bump);
            p.sx = 1.0 / p.sy;
            p.pivot = (0.5, 0.0);
        }
        "squash" => {
            p.sy = 1.0 - 0.28 * bump;
            p.sx = 1.0 + 0.2 * bump;
            p.pivot = (0.5, 0.0);
        }
        "blink" => p.alpha = 1.0 - 0.75 * (2.0 * PI * c(u)).sin().abs(),
        "tada" => {
            p.sx = 1.0 + 0.12 * bump;
            p.sy = p.sx;
            p.rot = (u * 6.0 * PI).sin() * 3.5 * bump;
        }
        "dim" => p.alpha = 1.0 - 0.65 * bump,
        _ => return None,
    }
    Some(p)
}

/// A motion as a program names it: `/pop`, `[/pop 0.8]` or
/// `[/pop 0.8 /elastic-out]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub name: String,
    pub dur: f64,
    /// `None`: the motion's own default.
    pub ease: Option<String>,
    pub kind: Kind,
}

impl Spec {
    pub fn parse(v: &Value) -> Result<Spec, String> {
        let usage = "a motion is /name, [/name seconds] or [/name seconds /easing]";
        let (name, dur, ease) = match v {
            Value::Name(n) => (n.clone(), None, None),
            Value::Array(a) => {
                let Some(Value::Name(n)) = a.first() else {
                    return Err(usage.into());
                };
                let dur = match a.get(1) {
                    None => None,
                    Some(d) => Some(d.as_f64().ok_or(usage)?),
                };
                let ease = match a.get(2) {
                    None => None,
                    Some(Value::Name(e)) => Some(e.clone()),
                    Some(_) => return Err(usage.into()),
                };
                if a.len() > 3 {
                    return Err(usage.into());
                }
                (n.clone(), dur, ease)
            }
            _ => return Err(usage.into()),
        };
        let known = find(&name);
        Ok(Spec {
            dur: dur.or(known.map(|m| m.dur)).unwrap_or(0.5).max(1e-6),
            ease: ease.or(known.map(|m| m.ease.to_string())),
            kind: known.map_or(Entrance, |m| m.kind),
            name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entrances_start_absent_and_end_at_rest_emphases_rest_at_both_ends() {
        for m in MOTIONS {
            let start = pose(m.name, 0.0, 7.2).unwrap();
            let end = pose(m.name, 1.0, 7.2).unwrap();
            assert!(end.at_rest(), "{} is not at rest at u = 1: {end:?}", m.name);
            match m.kind {
                Entrance => {
                    assert!(start.absent(), "{} is visible at u = 0", m.name);
                    assert!(crate::ease::ease(m.ease, 0.5).is_some(), "{}", m.name);
                }
                Emphasis => assert!(start.at_rest(), "{} moves at u = 0", m.name),
            }
            let mid = pose(m.name, 0.4, 7.2).unwrap();
            assert!(!mid.at_rest(), "{} does nothing at u = 0.4", m.name);
        }
        assert!(pose("no-such", 0.5, 1.0).is_none());
    }

    #[test]
    fn specs_parse_with_defaults() {
        let s = Spec::parse(&Value::Name("pop".into())).unwrap();
        assert_eq!((s.dur, s.ease.as_deref()), (0.5, Some("back-out")));
        let s = Spec::parse(&Value::Array(vec![
            Value::Name("rise".into()),
            Value::Real(1.5),
            Value::Name("linear".into()),
        ]))
        .unwrap();
        assert_eq!((s.dur, s.ease.as_deref()), (1.5, Some("linear")));
        assert!(Spec::parse(&Value::Int(3)).is_err());
        // A program's own motion: unknown here, default timing.
        let s = Spec::parse(&Value::Name("mine".into())).unwrap();
        assert_eq!((s.dur, s.ease, s.kind), (0.5, None, Entrance));
    }

    #[test]
    fn dict_poses_reject_unknown_keys() {
        let p = Pose::from_dict(&[("scale".into(), Value::Real(2.0))]).unwrap();
        assert_eq!((p.sx, p.sy), (2.0, 2.0));
        assert!(Pose::from_dict(&[("scael".into(), Value::Real(2.0))]).is_err());
    }
}
