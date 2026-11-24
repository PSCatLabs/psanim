//! Easing curves: functions from normalized time to normalized progress.
//!
//! Every curve maps 0 to 0 and 1 to 1 (except [`hold`](by_name), which jumps
//! at the end) and is evaluated on an input clamped to `[0, 1]`. Curves that
//! overshoot (`back-*`, `elastic-*`, `spring`) leave `[0, 1]` in between.
//! All arithmetic is plain `f64` with `libm`-free `std` calls on values whose
//! results are identical across platforms for our purposes; determinism of
//! renders is checked by the test suite on the pixels, not assumed here.

use std::f64::consts::PI;

/// The names [`by_name`] accepts, in documentation order.
pub const NAMES: &[&str] = &[
    "linear",
    "smooth",
    "smoother",
    "quad-in",
    "quad-out",
    "quad-inout",
    "cubic-in",
    "cubic-out",
    "cubic-inout",
    "quart-in",
    "quart-out",
    "quart-inout",
    "quint-in",
    "quint-out",
    "quint-inout",
    "sine-in",
    "sine-out",
    "sine-inout",
    "expo-in",
    "expo-out",
    "expo-inout",
    "circ-in",
    "circ-out",
    "circ-inout",
    "back-in",
    "back-out",
    "back-inout",
    "elastic-in",
    "elastic-out",
    "elastic-inout",
    "bounce-in",
    "bounce-out",
    "bounce-inout",
    "spring",
    "hold",
];

fn pow_in(x: f64, p: i32) -> f64 {
    x.powi(p)
}
fn pow_out(x: f64, p: i32) -> f64 {
    1.0 - (1.0 - x).powi(p)
}
fn pow_inout(x: f64, p: i32) -> f64 {
    if x < 0.5 {
        (2.0 * x).powi(p) / 2.0
    } else {
        1.0 - (2.0 * (1.0 - x)).powi(p) / 2.0
    }
}

fn bounce_out(x: f64) -> f64 {
    const N: f64 = 7.5625;
    const D: f64 = 2.75;
    if x < 1.0 / D {
        N * x * x
    } else if x < 2.0 / D {
        let x = x - 1.5 / D;
        N * x * x + 0.75
    } else if x < 2.5 / D {
        let x = x - 2.25 / D;
        N * x * x + 0.9375
    } else {
        let x = x - 2.625 / D;
        N * x * x + 0.984375
    }
}

/// The easing curve called `name`, or `None` for a name that is not one.
pub fn by_name(name: &str) -> Option<fn(f64) -> f64> {
    const C1: f64 = 1.70158;
    const C2: f64 = C1 * 1.525;
    const C3: f64 = C1 + 1.0;
    Some(match name {
        "linear" => |x| x,
        "smooth" => |x| x * x * (3.0 - 2.0 * x),
        "smoother" => |x| x * x * x * (x * (x * 6.0 - 15.0) + 10.0),
        "quad-in" => |x| pow_in(x, 2),
        "quad-out" => |x| pow_out(x, 2),
        "quad-inout" => |x| pow_inout(x, 2),
        "cubic-in" => |x| pow_in(x, 3),
        "cubic-out" => |x| pow_out(x, 3),
        "cubic-inout" => |x| pow_inout(x, 3),
        "quart-in" => |x| pow_in(x, 4),
        "quart-out" => |x| pow_out(x, 4),
        "quart-inout" => |x| pow_inout(x, 4),
        "quint-in" => |x| pow_in(x, 5),
        "quint-out" => |x| pow_out(x, 5),
        "quint-inout" => |x| pow_inout(x, 5),
        "sine-in" => |x| 1.0 - (x * PI / 2.0).cos(),
        "sine-out" => |x| (x * PI / 2.0).sin(),
        "sine-inout" => |x| -((PI * x).cos() - 1.0) / 2.0,
        "expo-in" => |x| {
            if x <= 0.0 {
                0.0
            } else {
                (2.0f64).powf(10.0 * x - 10.0)
            }
        },
        "expo-out" => |x| {
            if x >= 1.0 {
                1.0
            } else {
                1.0 - (2.0f64).powf(-10.0 * x)
            }
        },
        "expo-inout" => |x| {
            if x <= 0.0 {
                0.0
            } else if x >= 1.0 {
                1.0
            } else if x < 0.5 {
                (2.0f64).powf(20.0 * x - 10.0) / 2.0
            } else {
                (2.0 - (2.0f64).powf(-20.0 * x + 10.0)) / 2.0
            }
        },
        "circ-in" => |x| 1.0 - (1.0 - x * x).max(0.0).sqrt(),
        "circ-out" => |x| (1.0 - (x - 1.0) * (x - 1.0)).max(0.0).sqrt(),
        "circ-inout" => |x| {
            if x < 0.5 {
                (1.0 - (1.0 - 4.0 * x * x).max(0.0).sqrt()) / 2.0
            } else {
                ((1.0 - (-2.0 * x + 2.0).powi(2)).max(0.0).sqrt() + 1.0) / 2.0
            }
        },
        "back-in" => |x| C3 * x * x * x - C1 * x * x,
        "back-out" => |x| 1.0 + C3 * (x - 1.0).powi(3) + C1 * (x - 1.0).powi(2),
        "back-inout" => |x| {
            if x < 0.5 {
                ((2.0 * x).powi(2) * ((C2 + 1.0) * 2.0 * x - C2)) / 2.0
            } else {
                ((2.0 * x - 2.0).powi(2) * ((C2 + 1.0) * (x * 2.0 - 2.0) + C2) + 2.0) / 2.0
            }
        },
        "elastic-in" => |x| {
            if x <= 0.0 {
                0.0
            } else if x >= 1.0 {
                1.0
            } else {
                -(2.0f64).powf(10.0 * x - 10.0) * ((x * 10.0 - 10.75) * (2.0 * PI / 3.0)).sin()
            }
        },
        "elastic-out" => |x| {
            if x <= 0.0 {
                0.0
            } else if x >= 1.0 {
                1.0
            } else {
                (2.0f64).powf(-10.0 * x) * ((x * 10.0 - 0.75) * (2.0 * PI / 3.0)).sin() + 1.0
            }
        },
        "elastic-inout" => |x| {
            const C5: f64 = 2.0 * PI / 4.5;
            if x <= 0.0 {
                0.0
            } else if x >= 1.0 {
                1.0
            } else if x < 0.5 {
                -((2.0f64).powf(20.0 * x - 10.0) * ((20.0 * x - 11.125) * C5).sin()) / 2.0
            } else {
                ((2.0f64).powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * C5).sin()) / 2.0 + 1.0
            }
        },
        "bounce-out" => bounce_out,
        "bounce-in" => |x| 1.0 - bounce_out(1.0 - x),
        "bounce-inout" => |x| {
            if x < 0.5 {
                (1.0 - bounce_out(1.0 - 2.0 * x)) / 2.0
            } else {
                (1.0 + bounce_out(2.0 * x - 1.0)) / 2.0
            }
        },
        // A damped spring released at 0 and settling on 1: one clear
        // overshoot, a small undershoot, then rest. Softer than elastic-out.
        "spring" => |x| {
            if x >= 1.0 {
                1.0
            } else {
                1.0 - (-6.5 * x).exp() * (11.0 * x).cos() * (1.0 - x)
            }
        },
        "hold" => |x| if x >= 1.0 { 1.0 } else { 0.0 },
        _ => return None,
    })
}

/// `x` clamped to `[0, 1]` and passed through the curve called `name`.
pub fn ease(name: &str, x: f64) -> Option<f64> {
    let f = by_name(name)?;
    Some(f(if x.is_nan() { 0.0 } else { x.clamp(0.0, 1.0) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_curve_hits_its_endpoints_and_stays_finite() {
        for name in NAMES {
            let f = by_name(name).unwrap();
            if *name != "hold" {
                assert!(f(0.0).abs() < 1e-9, "{name} at 0 = {}", f(0.0));
            }
            assert!((f(1.0) - 1.0).abs() < 1e-9, "{name} at 1 = {}", f(1.0));
            for i in 0..=1000 {
                let y = f(f64::from(i) / 1000.0);
                assert!(y.is_finite() && (-1.0..=2.0).contains(&y), "{name} {i} {y}");
            }
        }
        assert!(by_name("nope").is_none());
    }

    #[test]
    fn monotone_curves_are_monotone() {
        for name in NAMES {
            if ["back", "elastic", "spring", "bounce"]
                .iter()
                .any(|p| name.starts_with(p))
            {
                continue;
            }
            let f = by_name(name).unwrap();
            let mut last = f(0.0);
            for i in 1..=1000 {
                let y = f(f64::from(i) / 1000.0);
                assert!(y >= last - 1e-12, "{name} decreases at {i}");
                last = y;
            }
        }
    }

    #[test]
    fn inout_curves_are_symmetric() {
        for name in NAMES.iter().filter(|n| n.ends_with("inout")) {
            let f = by_name(name).unwrap();
            for i in 0..=100 {
                let x = f64::from(i) / 100.0;
                assert!((f(x) + f(1.0 - x) - 1.0).abs() < 1e-9, "{name} at {x}");
            }
        }
    }

    #[test]
    fn ease_clamps_its_input() {
        assert_eq!(ease("cubic-out", -3.0), Some(0.0));
        assert_eq!(ease("cubic-out", 7.0), Some(1.0));
        assert_eq!(ease("cubic-out", f64::NAN), Some(0.0));
        assert_eq!(ease("nope", 0.5), None);
    }
}
