//! The native operators behind the expressive libraries (`lib/pk*.ps`) and
//! the parts of the core vocabulary added with them: motions, viewports,
//! library loading, and the text, code, chart and path helpers that are
//! clumsy or slow in PostScript.
//!
//! As in [`crate::ops`], a dot-name is the native half of a word the
//! PostScript side finishes.

use pscat::host::{PathSegment, Value};
use pscat::{Interp, PsError};

use crate::motion::{self, Kind, Pose, Spec};
use crate::ops::{
    OpResult, Shared, begin_layer, ease_named, fail, flat_path, reg, with_canvas_pixels,
};
use crate::raster::{self, Placement};
use crate::{code, pathmath};

/// The libraries `use` can load: name, source, one line on what it is for.
pub const LIBRARIES: &[(&str, &str, &str)] = &[
    (
        "pkstage",
        include_str!("../lib/pkstage.ps"),
        "boxes and layout, themes (colour and type by role), framing the camera on a box, spotlight",
    ),
    (
        "pkmotion",
        include_str!("../lib/pkmotion.ps"),
        "entrances, exits and emphasis for any drawing, measured not anchored; cascade orders; echoes and trails",
    ),
    (
        "pktype",
        include_str!("../lib/pktype.ps"),
        "text in boxes, wrapping and fitting, kinetic type by glyph, word and line, counters, captions",
    ),
    (
        "pkdiagram",
        include_str!("../lib/pkdiagram.ps"),
        "nodes, connectors, arrows, flowing pulses, groups, callouts and hand-drawn marks",
    ),
    (
        "pkcode",
        include_str!("../lib/pkcode.ps"),
        "syntax-highlighted code: typing, line focus, scrolling, animated diffs",
    ),
    (
        "pkchart",
        include_str!("../lib/pkchart.ps"),
        "plot frames, axes, bars, lines, dots, donuts and rankings that animate between data",
    ),
    (
        "pkink",
        include_str!("../lib/pkink.ps"),
        "tapered strokes, strokes that draw themselves in any medium, gradients, radial repetition",
    ),
];

fn spec_from(i: &mut Interp, ctx: &Shared) -> Result<Spec, PsError> {
    let v = i.pop_value()?;
    match Spec::parse(&v) {
        Ok(s) => Ok(s),
        Err(e) => fail(ctx, PsError::Typecheck, e),
    }
}

/// Push `u /name` for the PostScript half of a motion word.
fn push_motion(i: &mut Interp, ctx: &Shared, spec: &Spec, linear: f64) -> OpResult {
    let x = linear.clamp(0.0, 1.0);
    let u = match &spec.ease {
        Some(e) => ease_named(ctx, e, x)?,
        None => x,
    };
    i.push_real(u);
    i.push_name(&spec.name);
    Ok(())
}

fn split_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `value` with `decimals` places, optionally grouped in thousands.
pub fn format_number(value: f64, decimals: i64, grouped: bool) -> String {
    let d = decimals.clamp(0, 9) as usize;
    let s = format!("{:.*}", d, value.abs());
    let (int, frac) = match s.split_once('.') {
        Some((a, b)) => (a.to_string(), Some(b.to_string())),
        None => (s, None),
    };
    let int = if grouped {
        let mut out = String::new();
        for (k, ch) in int.chars().enumerate() {
            if k > 0 && (int.len() - k) % 3 == 0 {
                out.push(',');
            }
            out.push(ch);
        }
        out
    } else {
        int
    };
    let zero = int.chars().all(|c| c == '0' || c == ',')
        && frac.as_deref().is_none_or(|f| f.chars().all(|c| c == '0'));
    let sign = if value < 0.0 && !zero { "-" } else { "" };
    match frac {
        Some(f) => format!("{sign}{int}.{f}"),
        None => format!("{sign}{int}"),
    }
}

/// Round tick positions covering `lo..hi`, about `n` of them.
pub fn nice_ticks(lo: f64, hi: f64, n: f64) -> Vec<f64> {
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    let range = hi - lo;
    if !(range.is_finite() && range > 0.0) {
        return vec![lo];
    }
    let raw = range / n.max(1.0);
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .iter()
        .map(|k| k * mag)
        .find(|s| *s >= raw * 0.999)
        .unwrap_or(10.0 * mag);
    let mut out = Vec::new();
    let mut k = (lo / step - 1e-9).ceil();
    while k * step <= hi + step * 1e-9 && out.len() < 200 {
        // Snap away the float dust so a label prints as 0.3, not 0.30000000000000004.
        let v = k * step;
        out.push((v / step).round() * step);
        k += 1.0;
    }
    out
}

/// The outline of a stroke along `polys` whose width follows `widths`,
/// spread evenly along each subpath's length.
fn taper(polys: &[pathmath::Poly], widths: &[f64]) -> Vec<PathSegment> {
    let width_at = |u: f64| -> f64 {
        if widths.len() == 1 {
            return widths[0];
        }
        let x = u.clamp(0.0, 1.0) * (widths.len() - 1) as f64;
        let k = (x.floor() as usize).min(widths.len() - 2);
        widths[k] + (widths[k + 1] - widths[k]) * (x - k as f64)
    };
    let mut out = Vec::new();
    for poly in polys {
        let mut pts = poly.points.clone();
        if poly.closed && pts.len() > 1 {
            pts.push(pts[0]);
        }
        pts.dedup_by(|a, b| (a.0 - b.0).hypot(a.1 - b.1) < 1e-9);
        if pts.len() < 2 {
            continue;
        }
        // A long straight run has no points along it for the width to
        // change at: cut every edge into pieces short against the whole.
        let total: f64 = pts
            .windows(2)
            .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
            .sum();
        let step = total / (widths.len() as f64 * 12.0).max(24.0);
        let mut fine = vec![pts[0]];
        for w in pts.windows(2) {
            let d = (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
            let n = (d / step).ceil().max(1.0) as usize;
            for k in 1..=n {
                let u = k as f64 / n as f64;
                fine.push((
                    w[0].0 + (w[1].0 - w[0].0) * u,
                    w[0].1 + (w[1].1 - w[0].1) * u,
                ));
            }
        }
        let pts = fine;
        let mut acc = vec![0.0];
        for w in pts.windows(2) {
            let d = (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
            acc.push(acc.last().unwrap() + d);
        }
        let total = *acc.last().unwrap();
        let n = pts.len();
        let mut left = Vec::with_capacity(n);
        let mut right = Vec::with_capacity(n);
        for k in 0..n {
            let a = pts[k.saturating_sub(1)];
            let b = pts[(k + 1).min(n - 1)];
            let (tx, ty) = (b.0 - a.0, b.1 - a.1);
            let len = tx.hypot(ty).max(1e-12);
            let (nx, ny) = (-ty / len, tx / len);
            let half = width_at(acc[k] / total).max(0.0) / 2.0;
            left.push((pts[k].0 + nx * half, pts[k].1 + ny * half));
            right.push((pts[k].0 - nx * half, pts[k].1 - ny * half));
        }
        out.push(PathSegment::MoveTo(left[0].0, left[0].1));
        out.extend(left[1..].iter().map(|p| PathSegment::LineTo(p.0, p.1)));
        out.extend(right.iter().rev().map(|p| PathSegment::LineTo(p.0, p.1)));
        out.push(PathSegment::Close);
    }
    out
}

/// A smooth track through `[t v t v ...]`: cubic Hermite with tangents
/// from the neighbouring keys, at rest at both ends.
fn smooth_keys(items: &[Value], t: f64) -> Result<Value, String> {
    let usage = "smoothkeys: expected [time value time value ...], values all numbers or all arrays of one length";
    if items.len() < 2 || !items.len().is_multiple_of(2) {
        return Err(usage.into());
    }
    let mut times = Vec::new();
    let mut vals: Vec<Vec<f64>> = Vec::new();
    let mut scalar = false;
    for pair in items.chunks(2) {
        times.push(pair[0].as_f64().ok_or(usage)?);
        match &pair[1] {
            Value::Array(_) => vals.push(pair[1].as_numbers().ok_or(usage)?),
            v => {
                scalar = true;
                vals.push(vec![v.as_f64().ok_or(usage)?]);
            }
        }
    }
    if vals.iter().any(|v| v.len() != vals[0].len()) || times.windows(2).any(|w| w[1] <= w[0]) {
        return Err(format!("{usage}; times must increase"));
    }
    let n = times.len();
    let wrap = |v: Vec<f64>| {
        if scalar {
            Value::Real(v[0])
        } else {
            Value::Array(v.into_iter().map(Value::Real).collect())
        }
    };
    if t <= times[0] || n == 1 {
        return Ok(wrap(vals[0].clone()));
    }
    if t >= times[n - 1] {
        return Ok(wrap(vals[n - 1].clone()));
    }
    let j = times.iter().position(|x| *x > t).unwrap();
    let (a, b) = (j - 1, j);
    let h = times[b] - times[a];
    let u = (t - times[a]) / h;
    let tangent = |k: usize, c: usize| -> f64 {
        if k == 0 || k == n - 1 {
            0.0
        } else {
            (vals[k + 1][c] - vals[k - 1][c]) / (times[k + 1] - times[k - 1])
        }
    };
    let (u2, u3) = (u * u, u * u * u);
    let out = (0..vals[0].len())
        .map(|c| {
            (2.0 * u3 - 3.0 * u2 + 1.0) * vals[a][c]
                + (u3 - 2.0 * u2 + u) * h * tangent(a, c)
                + (-2.0 * u3 + 3.0 * u2) * vals[b][c]
                + (u3 - u2) * h * tangent(b, c)
        })
        .collect();
    Ok(wrap(out))
}

/// Straighten `poly` by `u` (0 unchanged, 1 a straight line), keeping
/// every edge's length: each turn is scaled by `1 - u`. The point `anchor`
/// (0..1 of the way along) stays where it is, with its direction.
fn straighten(poly: &pathmath::Poly, u: f64, anchor: f64) -> Vec<(f64, f64)> {
    let mut pts = poly.points.clone();
    if poly.closed && pts.len() > 1 {
        pts.push(pts[0]);
    }
    if pts.len() < 2 {
        return pts;
    }
    let lens: Vec<f64> = pts
        .windows(2)
        .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
        .collect();
    let heads: Vec<f64> = pts
        .windows(2)
        .map(|w| (w[1].1 - w[0].1).atan2(w[1].0 - w[0].0))
        .collect();
    let total: f64 = lens.iter().sum();
    // The edge the anchor lies on, and how far along it.
    let target = anchor.clamp(0.0, 1.0) * total;
    let (mut k, mut run) = (0, 0.0);
    while k + 1 < lens.len() && run + lens[k] < target {
        run += lens[k];
        k += 1;
    }
    let f = if lens[k] > 0.0 {
        (target - run) / lens[k]
    } else {
        0.0
    };
    let pin = (
        pts[k].0 + (pts[k + 1].0 - pts[k].0) * f,
        pts[k].1 + (pts[k + 1].1 - pts[k].1) * f,
    );
    // New headings: the anchor edge keeps its own; turns away from it shrink.
    let keep = 1.0 - u;
    let turn = |a: f64, b: f64| {
        let mut d = b - a;
        while d > std::f64::consts::PI {
            d -= 2.0 * std::f64::consts::PI;
        }
        while d < -std::f64::consts::PI {
            d += 2.0 * std::f64::consts::PI;
        }
        d
    };
    // The direction at the anchor itself: between the edge's own and its
    // neighbour's, by how near the anchor is to that end of the edge.
    let tangent = if f >= 0.5 && k + 1 < heads.len() {
        heads[k] + turn(heads[k], heads[k + 1]) * (f - 0.5)
    } else if f < 0.5 && k > 0 {
        heads[k] - turn(heads[k - 1], heads[k]) * (0.5 - f)
    } else {
        heads[k]
    };
    let mut new_heads = heads.clone();
    new_heads[k] = tangent + (heads[k] - tangent) * keep;
    for j in k + 1..heads.len() {
        new_heads[j] = new_heads[j - 1] + turn(heads[j - 1], heads[j]) * keep;
    }
    for j in (0..k).rev() {
        new_heads[j] = new_heads[j + 1] - turn(heads[j], heads[j + 1]) * keep;
    }
    let mut out = vec![(0.0, 0.0); pts.len()];
    out[k] = (
        pin.0 - new_heads[k].cos() * lens[k] * f,
        pin.1 - new_heads[k].sin() * lens[k] * f,
    );
    for j in k..lens.len() {
        out[j + 1] = (
            out[j].0 + new_heads[j].cos() * lens[j],
            out[j].1 + new_heads[j].sin() * lens[j],
        );
    }
    for j in (0..k).rev() {
        out[j] = (
            out[j + 1].0 - new_heads[j].cos() * lens[j],
            out[j + 1].1 - new_heads[j].sin() * lens[j],
        );
    }
    out
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    // --- libraries ----------------------------------------------------------
    reg(interp, ctx, ".libsource", |i, ctx| {
        let name = match i.pop_value()? {
            Value::Name(n) => n,
            Value::String(s) => split_text(&s),
            _ => {
                return fail(
                    ctx,
                    PsError::Typecheck,
                    "use takes a library name: /pkmotion use",
                );
            }
        };
        match LIBRARIES.iter().find(|l| l.0 == name) {
            Some(lib) => {
                i.push_string(lib.1.as_bytes());
                Ok(())
            }
            None => fail(
                ctx,
                PsError::Undefined(name.clone()),
                format!(
                    "there is no library /{name}. Libraries: {}. PSCat's own load with (lib/artkit.ps) run.",
                    LIBRARIES
                        .iter()
                        .map(|l| format!("/{}", l.0))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            ),
        }
    });

    reg(interp, ctx, ".nobeat", |i, ctx| {
        let name = i.pop_name().unwrap_or_default();
        fail(
            ctx,
            PsError::Undefined(name.clone()),
            format!(
                "the storyboard has no beat /{name}. Declare beats with [ /name seconds ... ] storyboard, before `animation`."
            ),
        )
    });

    // --- viewports ----------------------------------------------------------
    reg(interp, ctx, ".viewpush", |i, ctx| {
        let (h, w) = (i.pop_f64()?, i.pop_f64()?);
        let mut c = ctx.borrow_mut();
        if c.views.len() >= 64 {
            c.hint = Some("viewports nested too deeply".into());
            return Err(PsError::Limitcheck);
        }
        c.views.push((w, h));
        Ok(())
    });
    reg(interp, ctx, ".viewpop", |_, ctx| {
        ctx.borrow_mut().views.pop();
        Ok(())
    });

    // --- motions ------------------------------------------------------------
    // spec .enterspec -> u /name      eased progress through the first
    // `dur` seconds of the current time scope.
    reg(interp, ctx, ".enterspec", |i, ctx| {
        let spec = spec_from(i, ctx)?;
        let t = ctx.borrow().scope().t;
        push_motion(i, ctx, &spec, t / spec.dur)
    });
    // spec .leavespec -> u /name      the same curve run backward through
    // the last `dur` seconds of the scope.
    reg(interp, ctx, ".leavespec", |i, ctx| {
        let spec = spec_from(i, ctx)?;
        let s = ctx.borrow().scope();
        push_motion(i, ctx, &spec, (s.dur - s.t) / spec.dur)
    });
    // t spec .emphspec -> u /name     progress through `dur` seconds from t.
    reg(interp, ctx, ".emphspec", |i, ctx| {
        let spec = spec_from(i, ctx)?;
        let t0 = i.pop_f64()?;
        let t = ctx.borrow().scope().t;
        push_motion(i, ctx, &spec, (t - t0) / spec.dur)
    });
    // u spec .motionspec -> u /name   u is used as given.
    reg(interp, ctx, ".motionspec", |i, ctx| {
        let spec = spec_from(i, ctx)?;
        i.push_name(&spec.name);
        Ok(())
    });
    // u /name|dict .motionbegin -> code    0: draw nothing; 1: draw as it
    // is; 2: a layer is open and `.motionend` will place it.
    reg(interp, ctx, ".motionbegin", |i, ctx| {
        let what = i.pop_value()?;
        let u = i.pop_f64()?;
        let unit = {
            let c = ctx.borrow();
            c.width.min(c.height) / 100.0
        };
        let pose = match &what {
            Value::Name(n) => match motion::pose(n, u, unit) {
                Some(p) => p,
                None => {
                    return fail(
                        ctx,
                        PsError::Rangecheck,
                        format!(
                            "unknown motion /{n}. Known: {}. Define your own with `motions /{n} {{ ... }} put`.",
                            motion::MOTIONS
                                .iter()
                                .map(|m| format!("/{}", m.name))
                                .collect::<Vec<_>>()
                                .join(" ")
                        ),
                    );
                }
            },
            Value::Dict(entries) => match Pose::from_dict(entries) {
                Ok(p) => p,
                Err(e) => return fail(ctx, PsError::Rangecheck, e),
            },
            _ => {
                return fail(
                    ctx,
                    PsError::Typecheck,
                    "a motion in the `motions` dictionary takes u and returns a dictionary: { 1 exch sub -40 mul /dy exch << 3 1 roll >> }",
                );
            }
        };
        let code = if pose.absent() {
            0
        } else if pose.at_rest() {
            1
        } else {
            begin_layer(i, ctx)?;
            ctx.borrow_mut().poses.push(pose);
            2
        };
        i.push_int(code);
        Ok(())
    });
    reg(interp, ctx, ".motionend", |i, ctx| {
        let Some(pose) = ctx.borrow_mut().poses.pop() else {
            return fail(
                ctx,
                PsError::Rangecheck,
                "a motion ended without having begun",
            );
        };
        let px = ctx.borrow().px_scale;
        let (w, h) = i.canvas_size();
        let (w, h) = (w as usize, h as usize);
        // What was painted, before any blur spreads it: the motion's box.
        let rest = raster::painted_box(i.canvas_premultiplied_rgba8(), w, h);
        if pose.blur > 0.0 && rest.is_some() {
            with_canvas_pixels(i, ctx, |p, w, h, scratch| {
                raster::blur(p, w, h, pose.blur * px, scratch)
            })?;
        }
        let Some(mut below) = ctx.borrow_mut().layers.pop() else {
            return fail(ctx, PsError::Rangecheck, "a motion's layer is missing");
        };
        if let Some((x0, y0, x1, y1)) = rest {
            let (x0, y0, x1, y1) = (x0 as f64, y0 as f64, x1 as f64, y1 as f64);
            let (bw, bh) = (x1 - x0, y1 - y0);
            // Pixels run y down; poses are y up.
            let (pvx, pvy) = (x0 + pose.pivot.0 * bw, y1 - pose.pivot.1 * bh);
            let (ox, oy) = (
                pose.dx * px + pose.dxrel * bw,
                -(pose.dy * px) - pose.dyrel * bh,
            );
            let (sin, cos) = pose.rot.to_radians().sin_cos();
            let m = [cos * pose.sx, -sin * pose.sx, sin * pose.sy, cos * pose.sy];
            let reach = if pose.blur > 0.0 {
                (pose.blur * px * 3.0).ceil() + 1.0
            } else {
                0.0
            };
            let wiped = pose.wipe != [0.0, 0.0, 1.0, 1.0];
            let src = if wiped {
                (
                    x0 + pose.wipe[0] * bw,
                    y1 - pose.wipe[3] * bh,
                    x0 + pose.wipe[2] * bw,
                    y1 - pose.wipe[1] * bh,
                )
            } else {
                (x0 - reach, y0 - reach, x1 + reach, y1 + reach)
            };
            let clip = if pose.mask {
                (x0, y0, x1, y1)
            } else {
                (0.0, 0.0, w as f64, h as f64)
            };
            raster::composite_placed(
                below.premultiplied_rgba8_mut(),
                i.canvas_premultiplied_rgba8(),
                w,
                h,
                &Placement {
                    m: [
                        m[0],
                        m[1],
                        m[2],
                        m[3],
                        pvx + ox - (m[0] * pvx + m[2] * pvy),
                        pvy + oy - (m[1] * pvx + m[3] * pvy),
                    ],
                    opacity: pose.alpha,
                    src,
                    clip,
                },
            );
        }
        let layer = i.swap_canvas(below).map_err(|_| PsError::Rangecheck)?;
        ctx.borrow_mut().spare = Some(layer);
        Ok(())
    });
    // -  motionnames  array     the built-in motions, for a catalogue.
    reg(interp, ctx, "motionnames", |i, _| {
        i.push_value(&Value::Array(
            motion::MOTIONS
                .iter()
                .map(|m| Value::Name(m.name.into()))
                .collect(),
        ))?;
        Ok(())
    });
    // /name .motionkind -> bool   true for an emphasis.
    reg(interp, ctx, ".motionemphasis", |i, _| {
        let n = i.pop_name()?;
        i.push_bool(motion::find(&n).is_some_and(|m| m.kind == Kind::Emphasis));
        Ok(())
    });

    // --- values -------------------------------------------------------------
    reg(interp, ctx, "smoothkeys", |i, ctx| {
        let Value::Array(items) = i.pop_value()? else {
            return fail(ctx, PsError::Typecheck, "smoothkeys takes [t v t v ...]");
        };
        let t = ctx.borrow().scope().t;
        match smooth_keys(&items, t) {
            Ok(v) => {
                i.push_value(&v)?;
                Ok(())
            }
            Err(e) => fail(ctx, PsError::Rangecheck, e),
        }
    });

    // array x valueat -> v    the array read at a fractional index.
    reg(interp, ctx, "valueat", |i, ctx| {
        let x = i.pop_f64()?;
        let Value::Array(a) = i.pop_value()? else {
            return fail(
                ctx,
                PsError::Typecheck,
                "valueat takes an array and an index: table 22.4 valueat",
            );
        };
        if a.is_empty() {
            return fail(ctx, PsError::Rangecheck, "valueat: the array is empty");
        }
        let x = x.clamp(0.0, (a.len() - 1) as f64);
        let k = (x.floor() as usize).min(a.len().saturating_sub(2));
        let v = if a.len() == 1 {
            Some(a[0].clone())
        } else {
            crate::ops::lerp_value(&a[k], &a[k + 1], x - k as f64)
        };
        match v {
            Some(v) => {
                i.push_value(&v)?;
                Ok(())
            }
            None => fail(
                ctx,
                PsError::Typecheck,
                "valueat: elements must be numbers, or arrays of numbers of one length",
            ),
        }
    });

    // --- text ---------------------------------------------------------------
    reg(interp, ctx, "strcat", |i, _| {
        let b = i.pop_string()?;
        let mut a = i.pop_string()?;
        a.extend_from_slice(&b);
        i.push_string(&a);
        Ok(())
    });
    reg(interp, ctx, "fmtnum", |i, _| {
        let d = i.pop_int()?;
        let v = i.pop_f64()?;
        i.push_string(format_number(v, d, true).as_bytes());
        Ok(())
    });
    reg(interp, ctx, "fmtfixed", |i, _| {
        let d = i.pop_int()?;
        let v = i.pop_f64()?;
        i.push_string(format_number(v, d, false).as_bytes());
        Ok(())
    });
    reg(interp, ctx, "splitwords", |i, _| {
        let s = i.pop_string()?;
        i.push_value(&Value::Array(
            s.split(|b| *b == b' ' || *b == b'\t' || *b == b'\n')
                .filter(|w| !w.is_empty())
                .map(|w| Value::String(w.to_vec()))
                .collect(),
        ))?;
        Ok(())
    });
    reg(interp, ctx, "splitlines", |i, _| {
        let s = i.pop_string()?;
        let mut lines: Vec<&[u8]> = s.split(|b| *b == b'\n').collect();
        if lines.last().is_some_and(|l| l.is_empty()) && lines.len() > 1 {
            lines.pop();
        }
        i.push_value(&Value::Array(
            lines
                .into_iter()
                .map(|l| Value::String(l.to_vec()))
                .collect(),
        ))?;
        Ok(())
    });

    // --- code ---------------------------------------------------------------
    // (source) /lang codetokens -> [ [ (text) /kind ... ] per line ]
    reg(interp, ctx, "codetokens", |i, ctx| {
        let lang = i.pop_name()?;
        let src = split_text(&i.pop_string()?);
        let Some(lang) = code::Lang::from_name(&lang) else {
            return fail(
                ctx,
                PsError::Rangecheck,
                format!("unknown language /{lang}. Known: {}", code::LANG_NAMES),
            );
        };
        let lines = code::tokenize(&src, lang)
            .into_iter()
            .map(|line| {
                Value::Array(
                    line.into_iter()
                        .flat_map(|(text, kind)| {
                            [Value::String(text.into_bytes()), Value::Name(kind.into())]
                        })
                        .collect(),
                )
            })
            .collect();
        i.push_value(&Value::Array(lines))?;
        Ok(())
    });
    // (old) (new) difflines -> [ [ /same|/del|/add (line) oldindex newindex ] ... ]
    reg(interp, ctx, "difflines", |i, _| {
        let new = split_text(&i.pop_string()?);
        let old = split_text(&i.pop_string()?);
        let rows = code::diff_lines(&old, &new)
            .into_iter()
            .map(|r| {
                Value::Array(vec![
                    Value::Name(r.kind.into()),
                    Value::String(r.text.into_bytes()),
                    Value::Int(r.old),
                    Value::Int(r.new),
                ])
            })
            .collect();
        i.push_value(&Value::Array(rows))?;
        Ok(())
    });

    // --- charts -------------------------------------------------------------
    reg(interp, ctx, "niceticks", |i, _| {
        let (n, hi, lo) = (i.pop_f64()?, i.pop_f64()?, i.pop_f64()?);
        i.push_numbers(&nice_ticks(lo, hi, n));
        Ok(())
    });
    // [v ...] ranks -> [rank ...]   0 for the largest; ties in input order.
    reg(interp, ctx, "ranks", |i, _| {
        let v = i.pop_numbers()?;
        let mut order: Vec<usize> = (0..v.len()).collect();
        order.sort_by(|a, b| {
            v[*b]
                .partial_cmp(&v[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut rank = vec![0.0; v.len()];
        for (r, k) in order.into_iter().enumerate() {
            rank[k] = r as f64;
        }
        i.push_numbers(&rank);
        Ok(())
    });

    reg(interp, ctx, "zeropad", |i, _| {
        let width = i.pop_int()?.clamp(0, 32) as usize;
        let v = i.pop_f64()?.round() as i64;
        i.push_string(format!("{v:0width$}").as_bytes());
        Ok(())
    });
    // seconds | (H:MM[:SS[.fff]]) parseclock -> seconds
    reg(interp, ctx, "parseclock", |i, ctx| {
        match i.pop_value()? {
            Value::String(s) => match parse_clock(&String::from_utf8_lossy(&s)) {
                Some(secs) => i.push_real(secs),
                None => {
                    return fail(
                        ctx,
                        PsError::Rangecheck,
                        format!(
                            "parseclock: ({}) is not a time of day. Write (HH:MM), (HH:MM:SS) or (HH:MM:SS.fff), or give seconds since midnight",
                            String::from_utf8_lossy(&s)
                        ),
                    );
                }
            },
            v => match v.as_f64() {
                Some(x) => i.push_real(x),
                None => return Err(PsError::Typecheck),
            },
        }
        Ok(())
    });
    // seconds clockparts -> hours minutes seconds fraction
    reg(interp, ctx, "clockparts", |i, _| {
        let t = i.pop_f64()?.rem_euclid(86400.0);
        let whole = t.floor();
        let n = whole as i64;
        i.push_int(n / 3600);
        i.push_int(n / 60 % 60);
        i.push_int(n % 60);
        i.push_real(t - whole);
        Ok(())
    });
    // seconds fmtday -> (HH:MM:SS), the time of day on the 24 hour clock
    reg(interp, ctx, "fmtday", |i, _| {
        let n = i.pop_f64()?.rem_euclid(86400.0).floor() as i64;
        let text = format!("{:02}:{:02}:{:02}", n / 3600, n / 60 % 60, n % 60);
        i.push_string(text.as_bytes());
        Ok(())
    });
    reg(interp, ctx, "fmtclock", |i, _| {
        let secs = i.pop_f64()?.max(0.0).floor() as i64;
        let text = if secs >= 3600 {
            format!("{}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
        } else {
            format!("{:02}:{:02}", secs / 60, secs % 60)
        };
        i.push_string(text.as_bytes());
        Ok(())
    });

    // --- paths --------------------------------------------------------------
    // u anchor straightenpath: unbend the current path, keeping its length.
    reg(interp, ctx, "straightenpath", |i, ctx| {
        let (anchor, u) = (i.pop_f64()?, i.pop_f64()?);
        let polys = flat_path(i, ctx)?;
        let mut out = Vec::new();
        for poly in &polys {
            let pts = straighten(poly, u, anchor);
            for (k, p) in pts.iter().enumerate() {
                out.push(if k == 0 {
                    PathSegment::MoveTo(p.0, p.1)
                } else {
                    PathSegment::LineTo(p.0, p.1)
                });
            }
        }
        i.set_current_path(&out)
    });
    reg(interp, ctx, "taperpath", |i, ctx| {
        let widths = i.pop_numbers()?;
        if widths.is_empty() {
            return fail(
                ctx,
                PsError::Rangecheck,
                "taperpath takes the widths along the stroke: [0 8 0] taperpath fill",
            );
        }
        let polys = flat_path(i, ctx)?;
        i.set_current_path(&taper(&polys, &widths))
    });
}

/// A time of day as seconds since midnight: `H:MM`, `H:MM:SS` or
/// `H:MM:SS.fff`. Hours 0..=24 (24:00:00 is allowed), minutes and seconds
/// below 60.
fn parse_clock(text: &str) -> Option<f64> {
    let mut parts = text.trim().split(':');
    let h: u32 = parts.next()?.trim().parse().ok()?;
    let m: u32 = parts.next()?.trim().parse().ok()?;
    let s: f64 = match parts.next() {
        Some(s) => s.trim().parse().ok()?,
        None => 0.0,
    };
    if parts.next().is_some() || h > 24 || m > 59 || !(0.0..60.0).contains(&s) {
        return None;
    }
    Some(f64::from(h) * 3600.0 + f64::from(m) * 60.0 + s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_of_day_parse() {
        assert_eq!(parse_clock("12:34:56"), Some(45296.0));
        assert_eq!(parse_clock("7:05"), Some(25500.0));
        assert_eq!(parse_clock("23:59:59.5"), Some(86399.5));
        assert_eq!(parse_clock("25:00"), None);
        assert_eq!(parse_clock("12:60"), None);
        assert_eq!(parse_clock("noon"), None);
        assert_eq!(parse_clock("1:2:3:4"), None);
    }

    #[test]
    fn straightening_keeps_length_and_the_anchor() {
        // A quarter circle of radius 10, from (10, 0) up to (0, 10).
        let pts: Vec<(f64, f64)> = (0..=90)
            .map(|d| {
                let a = f64::from(d).to_radians();
                (10.0 * a.cos(), 10.0 * a.sin())
            })
            .collect();
        let poly = pathmath::Poly {
            points: pts.clone(),
            closed: false,
        };
        let len = |p: &[(f64, f64)]| -> f64 {
            p.windows(2)
                .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
                .sum()
        };
        assert_eq!(straighten(&poly, 0.0, 0.0).len(), pts.len());
        for (a, b) in straighten(&poly, 0.0, 0.3).iter().zip(&pts) {
            assert!((a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9);
        }
        let flat = straighten(&poly, 1.0, 0.0);
        assert!((len(&flat) - len(&pts)).abs() < 1e-9);
        // Anchored at its start, heading up: a vertical line from (10, 0).
        let end = flat[flat.len() - 1];
        assert!(
            (end.0 - 10.0).abs() < 0.2 && (end.1 - len(&pts)).abs() < 0.1,
            "{end:?}"
        );
        let half = straighten(&poly, 0.5, 0.5);
        assert!((len(&half) - len(&pts)).abs() < 1e-9);
        let mid = half[45];
        assert!((mid.0 - pts[45].0).abs() < 1e-6 && (mid.1 - pts[45].1).abs() < 1e-6);
    }

    #[test]
    fn numbers_format() {
        assert_eq!(format_number(1234567.891, 1, true), "1,234,567.9");
        assert_eq!(format_number(1234567.891, 0, false), "1234568");
        assert_eq!(format_number(-0.004, 2, true), "0.00");
        assert_eq!(format_number(-12.5, 1, true), "-12.5");
        assert_eq!(format_number(999.0, 0, true), "999");
        assert_eq!(format_number(1000.0, 0, true), "1,000");
    }

    #[test]
    fn ticks_are_round() {
        assert_eq!(
            nice_ticks(0.0, 100.0, 5.0),
            vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
        );
        assert_eq!(nice_ticks(0.0, 1.0, 4.0), vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        let t = nice_ticks(3.0, 47.0, 5.0);
        assert_eq!(t, vec![10.0, 20.0, 30.0, 40.0]);
        assert_eq!(nice_ticks(5.0, 5.0, 4.0), vec![5.0]);
    }

    #[test]
    fn smooth_keys_pass_through_their_keys_and_rest_at_the_ends() {
        let keys = [
            Value::Int(0),
            Value::Int(0),
            Value::Int(1),
            Value::Int(10),
            Value::Int(3),
            Value::Int(4),
        ];
        let at = |t: f64| smooth_keys(&keys, t).unwrap().as_f64().unwrap();
        assert_eq!(at(-1.0), 0.0);
        assert!((at(1.0) - 10.0).abs() < 1e-9);
        assert_eq!(at(9.0), 4.0);
        // At rest at the start: barely moved after a small step.
        assert!(at(0.01) < 0.01);
        // Velocity is continuous through the middle key.
        let (a, b) = (at(1.0) - at(0.999), at(1.001) - at(1.0));
        assert!((a - b).abs() < 1e-3, "{a} {b}");
    }

    #[test]
    fn a_taper_is_one_closed_outline_per_subpath() {
        let polys = vec![pathmath::Poly {
            points: vec![(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)],
            closed: false,
        }];
        let out = taper(&polys, &[0.0, 4.0, 0.0]);
        // One closed outline, widest in the middle, pointed at the ends.
        assert!(matches!(out[0], PathSegment::MoveTo(x, y) if x == 0.0 && y == 0.0));
        assert!(matches!(out.last(), Some(PathSegment::Close)));
        assert_eq!(
            out.iter()
                .filter(|s| matches!(s, PathSegment::MoveTo(..)))
                .count(),
            1
        );
        let top = out
            .iter()
            .filter_map(|s| match s {
                PathSegment::LineTo(x, y) => Some((*x, *y)),
                _ => None,
            })
            .fold((0.0, 0.0), |a, p| if p.1 > a.1 { p } else { a });
        assert!(
            (top.0 - 10.0).abs() < 1.0 && (top.1 - 2.0).abs() < 0.2,
            "{top:?}"
        );
        // A straight line of two points still tapers.
        let line = vec![pathmath::Poly {
            points: vec![(0.0, 0.0), (20.0, 0.0)],
            closed: false,
        }];
        assert!(taper(&line, &[0.0, 4.0, 0.0]).len() > 20);
    }
}
