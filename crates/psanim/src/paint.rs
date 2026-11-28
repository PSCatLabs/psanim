//! The native half of pkink's stroke scheduler (`paintplan`, `paintkept`,
//! `paintlive`, `painttip`, `paintall` in `lib/pkink.ps`).
//!
//! A plan is a list of strokes, each painted over a stretch of movie time.
//! On a persistent canvas a stroke is kept once, whole, on the step that
//! reaches its end, and until then is drawn in the overlay as far as it has
//! got; a soak is a number of passes, each kept on its own step. These
//! operators only choose: which units are due on this step, which strokes
//! are in progress and how far, where the tip is. The PostScript half runs
//! the strokes' procedures (a native operator cannot call one).

use pscat::host::Value;
use pscat::{Interp, PsError};

use crate::ctx::{Canvas, step_reaching};
use crate::ops::{OpResult, Shared, fail, reg, with_canvas_pixels};

/// The most strokes a plan may hold, and the most units (a whole stroke,
/// or one pass of a soak) over all of them: the per-frame work is bounded
/// by these.
pub(crate) const MAX_STROKES: usize = 10_000;
pub(crate) const MAX_UNITS: usize = 100_000;
pub(crate) const MAX_PASSES: i64 = 1_000;

const KINDS: &[&str] = &["along", "sweep", "soak"];
const KEYS: &[&str] = &[
    "Kind", "At", "For", "Until", "Seed", "Paint", "Path", "Reveal", "Across", "Axis", "Passes",
    "Ease",
];

const ALONG: i64 = 0;
const SWEEP: i64 = 1;
const SOAK: i64 = 2;

fn is_proc(v: &Value) -> bool {
    matches!(v, Value::Other(o) if o.is_procedure())
}

/// One stroke checked and filled in: the dictionary the PostScript half
/// reads, and its times.
struct Checked {
    dict: Vec<(String, Value)>,
    kind: i64,
    t0: f64,
    t1: f64,
    passes: i64,
}

fn check_stroke(k: usize, v: &Value) -> Result<Checked, String> {
    let at = |field: &str, what: &str| format!("paintplan: stroke {k}: /{field} {what}");
    let Value::Dict(entries) = v else {
        return Err(format!(
            "paintplan: stroke {k} is a {}, not a dictionary: << /Kind /along /At 1 /For 2 /Path {{ ... }} /Paint {{ ... }} >>",
            type_word(v)
        ));
    };
    for (key, _) in entries {
        let capital = key.chars().next().is_some_and(|c| c.is_ascii_uppercase());
        if capital && !KEYS.contains(&key.as_str()) {
            let hint = match crate::error::closest(key, KEYS.iter().copied()) {
                Some(near) => format!(" (did you mean /{near}?)"),
                None => String::new(),
            };
            return Err(format!(
                "paintplan: stroke {k}: /{key} is not a key paintplan knows{hint}. Its keys are {}; keys that start with a small letter are yours, and kept",
                KEYS.iter()
                    .map(|k| format!("/{k}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
    }
    let get = |key: &str| v.get(key);
    let kind_name = match get("Kind") {
        None => "along",
        Some(Value::Name(n)) if KINDS.contains(&n.as_str()) => n.as_str(),
        Some(other) => {
            return Err(at(
                "Kind",
                &format!(
                    "is {}: it is /along (a stroke along /Path), /sweep (a front moving /Across) or /soak (/Passes laid one over another)",
                    show(other)
                ),
            ));
        }
    };
    let kind = match kind_name {
        "along" => ALONG,
        "sweep" => SWEEP,
        _ => SOAK,
    };
    let t0 = match get("At") {
        Some(x) if x.as_f64().is_some_and(f64::is_finite) => x.as_f64().unwrap_or(0.0),
        Some(other) => {
            return Err(at(
                "At",
                &format!("is {}: when the stroke starts, in seconds", show(other)),
            ));
        }
        None => return Err(at("At", "is missing: when the stroke starts, in seconds")),
    };
    // The end: /For seconds after /At, or /Until a time (kept exactly, so a
    // plan computed in end times keeps them).
    let t1 = match (get("For"), get("Until")) {
        (Some(_), Some(_)) => {
            return Err(at(
                "Until",
                "and /For are both given: give one, the end time or the length",
            ));
        }
        (Some(x), None) if x.as_f64().is_some_and(|d| d.is_finite() && d > 0.0) => {
            t0 + x.as_f64().unwrap_or(1.0)
        }
        (Some(other), None) => {
            return Err(at(
                "For",
                &format!(
                    "is {}: how long the stroke takes, in seconds, more than 0",
                    show(other)
                ),
            ));
        }
        (None, Some(x)) if x.as_f64().is_some_and(|e| e.is_finite() && e > t0) => {
            x.as_f64().unwrap_or(t0 + 1.0)
        }
        (None, Some(other)) => {
            return Err(at(
                "Until",
                &format!(
                    "is {}: when the stroke ends, in seconds, after its /At ({t0})",
                    show(other)
                ),
            ));
        }
        (None, None) => {
            return Err(at(
                "For",
                "is missing: how long the stroke takes, in seconds (or /Until, when it ends)",
            ));
        }
    };
    let dur = t1 - t0;
    match get("Paint") {
        Some(p) if is_proc(p) => {}
        Some(other) => {
            return Err(at(
                "Paint",
                &format!(
                    "is {}: a procedure, u -> -, that draws the stroke as far as u (0..1)",
                    show(other)
                ),
            ));
        }
        None => {
            return Err(at(
                "Paint",
                "is missing: a procedure, u -> -, that draws the stroke as far as u (0..1)",
            ));
        }
    }
    let seed = match get("Seed") {
        None => k as i64 + 1,
        Some(Value::Int(s)) => *s,
        Some(Value::Real(r)) if r.fract() == 0.0 && r.abs() < 2e9 => *r as i64,
        Some(other) => {
            return Err(at(
                "Seed",
                &format!(
                    "is {}: an integer, given to srand before the stroke is drawn",
                    show(other)
                ),
            ));
        }
    };
    let only = |key: &str, kinds: &str| -> Result<(), String> {
        if get(key).is_some() {
            Err(at(
                key,
                &format!("has no meaning for a /{kind_name}: it is for {kinds}"),
            ))
        } else {
            Ok(())
        }
    };
    let mut reveal = 0.0;
    let mut across = Value::Null;
    let mut axis = "x";
    let mut passes = 1;
    match kind {
        ALONG => {
            match get("Path") {
                Some(p) if is_proc(p) => {}
                Some(other) => {
                    return Err(at(
                        "Path",
                        &format!(
                            "is {}: a procedure that builds the stroke's path",
                            show(other)
                        ),
                    ));
                }
                None => {
                    return Err(at(
                        "Path",
                        "is missing: an /along stroke needs a procedure that builds its path",
                    ));
                }
            }
            if let Some(r) = get("Reveal") {
                reveal = r.as_f64().filter(|w| w.is_finite() && *w > 0.0).ok_or_else(|| {
                    at("Reveal", &format!("is {}: the width of the band the stroke is revealed through, more than 0", show(r)))
                })?;
            }
            only("Across", "a /sweep")?;
            only("Axis", "a /sweep")?;
            only("Passes", "a /soak")?;
        }
        SWEEP => {
            match get("Across").and_then(Value::as_numbers) {
                Some(ab) if ab.len() == 2 && ab.iter().all(|x| x.is_finite()) && ab[0] != ab[1] => {
                    across = Value::Array(vec![Value::Real(ab[0]), Value::Real(ab[1])]);
                }
                _ => {
                    return Err(at(
                        "Across",
                        &format!(
                            "is {}: [a b], where the front starts and where it stops, two different numbers",
                            get("Across").map_or("missing".into(), show)
                        ),
                    ));
                }
            }
            match get("Axis") {
                None => {}
                Some(Value::Name(n)) if n == "x" || n == "y" => {
                    axis = if n == "x" { "x" } else { "y" };
                }
                Some(other) => {
                    return Err(at(
                        "Axis",
                        &format!(
                            "is {}: /x (the front moves across) or /y (it moves up or down)",
                            show(other)
                        ),
                    ));
                }
            }
            only("Path", "an /along")?;
            only("Reveal", "an /along")?;
            only("Passes", "a /soak")?;
        }
        _ => {
            passes = match get("Passes") {
                Some(Value::Int(n)) if (1..=MAX_PASSES).contains(n) => *n,
                other => {
                    return Err(at(
                        "Passes",
                        &format!(
                            "is {}: how many passes the soak lays, an integer from 1 to {MAX_PASSES}",
                            other.map_or("missing".into(), show)
                        ),
                    ));
                }
            };
            only("Path", "an /along")?;
            only("Reveal", "an /along")?;
            only("Across", "a /sweep")?;
            only("Axis", "a /sweep")?;
            only(
                "Ease",
                "an /along or a /sweep (a soak's passes are evenly spaced)",
            )?;
        }
    }
    let ease = match get("Ease") {
        None => Value::Null,
        Some(Value::Name(n)) if crate::ease::ease(n, 0.5).is_some() => Value::Name(n.clone()),
        Some(p) if is_proc(p) => p.clone(),
        Some(other) => {
            return Err(at(
                "Ease",
                &format!(
                    "is {}: an easing name (/sine-inout, /cubic-out ...) or a procedure u -> u",
                    show(other)
                ),
            ));
        }
    };
    let mut dict: Vec<(String, Value)> = entries
        .iter()
        .filter(|(key, _)| !KEYS.contains(&key.as_str()))
        .cloned()
        .collect();
    let mut put = |key: &str, v: Value| dict.push((key.to_string(), v));
    put("Kind", Value::Name(kind_name.into()));
    put("At", Value::Real(t0));
    put("For", Value::Real(dur));
    put("Until", Value::Real(t1));
    put("Seed", Value::Int(seed));
    put("Paint", get("Paint").cloned().unwrap_or(Value::Null));
    put("Path", get("Path").cloned().unwrap_or(Value::Null));
    put("Reveal", Value::Real(reveal));
    put("Across", across);
    put("Axis", Value::Name(axis.into()));
    put("Passes", Value::Int(passes));
    put("Ease", ease);
    Ok(Checked {
        dict,
        kind,
        t0,
        t1,
        passes,
    })
}

fn type_word(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Int(_) | Value::Real(_) => "number",
        Value::Name(_) => "name",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Dict(_) => "dictionary",
        Value::Other(o) if o.is_procedure() => "procedure",
        _ => "object",
    }
}

fn show(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Real(r) => r.to_string(),
        Value::Name(n) => format!("/{n}"),
        Value::Bool(b) => b.to_string(),
        other => format!("a {}", type_word(other)),
    }
}

/// The plan as the PostScript half keeps it, from the list of strokes.
pub(crate) fn compile(items: &[Value]) -> Result<Value, String> {
    if items.is_empty() {
        return Err("paintplan: the plan has no strokes".into());
    }
    if items.len() > MAX_STROKES {
        return Err(format!(
            "paintplan: {} strokes is more than {MAX_STROKES}",
            items.len()
        ));
    }
    let checked = items
        .iter()
        .enumerate()
        .map(|(k, v)| check_stroke(k, v))
        .collect::<Result<Vec<_>, _>>()?;
    // The units in the order they are kept: by time, then stroke, then pass.
    let mut units: Vec<(f64, usize, i64)> = Vec::new();
    for (k, c) in checked.iter().enumerate() {
        for p in 0..c.passes {
            let t = if c.kind == SOAK && p + 1 < c.passes {
                c.t0 + (c.t1 - c.t0) * (p + 1) as f64 / c.passes as f64
            } else {
                c.t1
            };
            units.push((t, k, p));
            if units.len() > MAX_UNITS {
                return Err(format!(
                    "paintplan: the plan lays more than {MAX_UNITS} units (strokes and soak passes)"
                ));
            }
        }
    }
    units.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let reals = |xs: Vec<f64>| Value::Array(xs.into_iter().map(Value::Real).collect());
    let ints = |xs: Vec<i64>| Value::Array(xs.into_iter().map(Value::Int).collect());
    Ok(Value::Dict(vec![
        (
            "Strokes".into(),
            Value::Array(
                checked
                    .iter()
                    .map(|c| Value::Dict(c.dict.clone()))
                    .collect(),
            ),
        ),
        (
            "Kinds".into(),
            ints(checked.iter().map(|c| c.kind).collect()),
        ),
        ("T0".into(), reals(checked.iter().map(|c| c.t0).collect())),
        ("T1".into(), reals(checked.iter().map(|c| c.t1).collect())),
        ("UT".into(), reals(units.iter().map(|u| u.0).collect())),
        (
            "US".into(),
            ints(units.iter().map(|u| u.1 as i64).collect()),
        ),
        ("UP".into(), ints(units.iter().map(|u| u.2).collect())),
        (
            "UU".into(),
            reals(
                units
                    .iter()
                    .map(|u| (u.2 + 1) as f64 / checked[u.1].passes as f64)
                    .collect(),
            ),
        ),
    ]))
}

/// The units due: `[i, j)` of the units sorted by time. On a kept canvas
/// those its step reaches; on a canvas that is not kept, every one reached
/// by now; with `all`, every one.
pub(crate) fn due(
    times: &[f64],
    rate: f64,
    step: i64,
    canvas: Canvas,
    all: bool,
) -> (usize, usize) {
    if all {
        return (0, times.len());
    }
    let reach = |t: f64| step_reaching(t, rate).max(0);
    let j = times.partition_point(|t| reach(*t) <= step);
    let i = match canvas {
        Canvas::Kept => times.partition_point(|t| reach(*t) < step),
        _ => 0,
    };
    (i.min(j), j)
}

/// The strokes in progress at `now`, as (stroke, linear u), in the order
/// they will be kept: started, and not yet on the kept canvas (on a canvas
/// that is not kept, not yet reached). Soaks have none.
pub(crate) fn live(
    kinds: &[i64],
    t0: &[f64],
    t1: &[f64],
    now: f64,
    rate: f64,
    step: i64,
) -> Vec<(usize, f64)> {
    let mut out: Vec<(usize, f64)> = (0..kinds.len().min(t0.len()).min(t1.len()))
        .filter(|&k| kinds[k] != SOAK && now > t0[k] && step_reaching(t1[k], rate).max(0) > step)
        .map(|k| (k, ((now - t0[k]) / (t1[k] - t0[k])).clamp(0.0, 1.0)))
        .collect();
    out.sort_by(|a, b| t1[a.0].total_cmp(&t1[b.0]).then(a.0.cmp(&b.0)));
    out
}

/// The stroke whose tip is moving at `now`, and how far it has got: of
/// those being drawn (started, not ended, not yet kept), the one that
/// started last.
pub(crate) fn tip(
    kinds: &[i64],
    t0: &[f64],
    t1: &[f64],
    now: f64,
    rate: f64,
    step: i64,
) -> Option<(usize, f64)> {
    (0..kinds.len().min(t0.len()).min(t1.len()))
        .filter(|&k| {
            kinds[k] != SOAK
                && now > t0[k]
                && now <= t1[k]
                && step_reaching(t1[k], rate).max(0) > step
        })
        .max_by(|&a, &b| t0[a].total_cmp(&t0[b]).then(a.cmp(&b)))
        .map(|k| (k, ((now - t0[k]) / (t1[k] - t0[k])).clamp(0.0, 1.0)))
}

/// A plan inside a component drawn into a persistent movie's kept canvas:
/// a component has no kept canvas of its own, so the plan would be drawn
/// whole on every step and pile up.
fn in_kept_component(ctx: &Shared, word: &str) -> OpResult {
    fail(
        ctx,
        PsError::Rangecheck,
        format!(
            "{word} inside a component that is drawn into a persistent movie's scene: a component has no kept canvas of its own, so its plan is drawn whole every frame and would pile up on the movie's. Place the component in the movie's `overlay`, or in a movie that is not persistent"
        ),
    )
}

fn pop_reals(i: &mut Interp, ctx: &Shared, word: &str) -> Result<Vec<f64>, PsError> {
    match i.pop_value()?.as_numbers() {
        Some(v) => Ok(v),
        None => fail(
            ctx,
            PsError::Typecheck,
            format!("{word} takes a plan made by paintplan"),
        ),
    }
}

fn pop_kinds(i: &mut Interp, ctx: &Shared, word: &str) -> Result<Vec<i64>, PsError> {
    let v = pop_reals(i, ctx, word)?;
    Ok(v.into_iter().map(|x| x as i64).collect())
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    // strokes .paintplan plan
    reg(interp, ctx, ".paintplan", |i, ctx| {
        let v = i.pop_value()?;
        let Some(items) = v.as_array() else {
            return fail(
                ctx,
                PsError::Typecheck,
                "paintplan takes an array of strokes: [ << /Kind /along /At 1 /For 2 /Path { ... } /Paint { ... } >> ... ] paintplan",
            );
        };
        match compile(items) {
            Ok(plan) => i.push_value(&plan),
            Err(msg) => fail(ctx, PsError::Rangecheck, msg),
        }
    });
    // UT all .paintdue i j
    reg(interp, ctx, ".paintdue", |i, ctx| {
        let all = i.pop_bool()?;
        let times = pop_reals(i, ctx, "paintkept")?;
        let (_, rate, step, canvas, in_comp) = ctx.borrow().step_view();
        if !all && canvas == Canvas::Overlay {
            return fail(
                ctx,
                PsError::Undefined("paintkept".into()),
                "paintkept lays strokes on the kept canvas: call it in `scene`. In `overlay` call paintlive, which shows the strokes in progress",
            );
        }
        if !all && in_comp && canvas == Canvas::Kept {
            return in_kept_component(ctx, "paintkept");
        }
        let (a, b) = due(&times, rate, step, canvas, all);
        i.push_int(a as i64);
        i.push_int(b as i64);
        Ok(())
    });
    // Kinds T0 T1 .paintlive [ s u s u ... ]
    reg(interp, ctx, ".paintlive", |i, ctx| {
        let t1 = pop_reals(i, ctx, "paintlive")?;
        let t0 = pop_reals(i, ctx, "paintlive")?;
        let kinds = pop_kinds(i, ctx, "paintlive")?;
        let (now, rate, step, canvas, in_comp) = ctx.borrow().step_view();
        if in_comp && canvas == Canvas::Kept {
            return in_kept_component(ctx, "paintlive");
        }
        if canvas == Canvas::Kept {
            return fail(
                ctx,
                PsError::Undefined("paintlive".into()),
                "paintlive shows the strokes in progress, which must not be kept: call it in `overlay`. In `scene` call paintkept",
            );
        }
        let out = live(&kinds, &t0, &t1, now, rate, step);
        let flat: Vec<Value> = out
            .into_iter()
            .flat_map(|(k, u)| [Value::Int(k as i64), Value::Real(u)])
            .collect();
        i.push_value(&Value::Array(flat))
    });
    // Kinds T0 T1 .painttip s u | -1
    reg(interp, ctx, ".painttip", |i, ctx| {
        let t1 = pop_reals(i, ctx, "painttip")?;
        let t0 = pop_reals(i, ctx, "painttip")?;
        let kinds = pop_kinds(i, ctx, "painttip")?;
        let (now, rate, step, _, _) = ctx.borrow().step_view();
        let found = tip(&kinds, &t0, &t1, now, rate, step);
        match found {
            Some((k, u)) => {
                i.push_int(k as i64);
                i.push_real(u);
            }
            None => i.push_int(-1),
        }
        Ok(())
    });
    // - .paintsettle -    drop PSCat's 16-bit shadow of the canvas before a
    // unit is laid, so each unit composites onto the 8-bit pixels alone:
    // laid on its own step, or with the whole board in one go, or shown in
    // the overlay, it then gives the same pixels. It swaps the canvas out
    // and back (`with_canvas_pixels` with nothing to do), relying on
    // `swap_canvas` clearing the shadow; PSCatLabs/pscat#1506 asks for that
    // to be documented and for an explicit call, which is the follow-up.
    reg(interp, ctx, ".paintsettle", |i, ctx| {
        with_canvas_pixels(i, ctx, |_, _, _, _| {})
    });
    // k got want .paintbad -    a stroke's procedure left the stack wrong
    reg(interp, ctx, ".paintbad", |i, ctx| {
        let field = i.pop_name()?;
        let n = i.pop_f64()? as i64;
        let k = i.pop_f64()? as i64;
        let what = if n > 0 {
            format!(
                "left {n} more operand{} than it was given",
                if n == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "took {} operand{} more than it was given",
                -n,
                if n == -1 { "" } else { "s" }
            )
        };
        let given = if field == "Paint" {
            " (it is given u, and must take it)"
        } else {
            " (it is given none)"
        };
        fail(
            ctx,
            PsError::Rangecheck,
            format!("paintplan: stroke {k}: /{field} {what}{given}"),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_keeps_each_unit_on_one_step() {
        let times = [0.5, 1.8, 1.8, 2.0, 3.01];
        let mut seen = vec![0; times.len()];
        for step in 0..120 {
            let (a, b) = due(&times, 30.0, step, Canvas::Kept, false);
            for s in &mut seen[a..b] {
                *s += 1;
            }
        }
        assert_eq!(seen, [1, 1, 1, 1, 1]);
        assert_eq!(due(&times, 30.0, 54, Canvas::Kept, false), (1, 3));
        assert_eq!(due(&times, 30.0, 54, Canvas::Fresh, false), (0, 3));
    }

    #[test]
    fn live_strokes_are_in_keeping_order() {
        // Stroke 0 starts first but ends last.
        let live = live(&[0, 0], &[0.0, 0.2], &[2.0, 1.0], 0.5, 30.0, 15);
        assert_eq!(live.iter().map(|l| l.0).collect::<Vec<_>>(), [1, 0]);
        assert!(tip(&[0, 0], &[0.0, 0.2], &[2.0, 1.0], 0.5, 30.0, 15).is_some_and(|t| t.0 == 1));
        // Ended exactly on a step, and kept on it: no tip.
        assert!(tip(&[0], &[0.0], &[1.0], 1.0, 30.0, 30).is_none());
    }
}
