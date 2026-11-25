//! The native operators psanim adds to the interpreter.
//!
//! Each is a closure registered with [`pscat::Interp::register_operator`]
//! that shares the engine's [`Ctx`]. Operators whose names start with a dot
//! are the native halves of combinators the prelude (`prelude.ps`) finishes
//! in PostScript: a native operator cannot call a procedure, so `.during`
//! decides and pushes the time scope, and the prelude's `during` runs the
//! body and pops the scope.

use std::cell::RefCell;
use std::rc::Rc;

use pscat::host::{PathSegment, Value};
use pscat::{Interp, Page, PsError};

use crate::ctx::{Ctx, LayerEnd, MAX_LAYERS, MAX_SCOPES, Running, TimeScope, hash01, wiggle};
use crate::ease;
use crate::pathmath;
use crate::raster::{self, Blend};

pub(crate) type Shared = Rc<RefCell<Ctx>>;
pub(crate) type OpResult = Result<(), PsError>;

/// Fail with `err`, leaving `hint` for the engine's error report.
pub(crate) fn fail<T>(ctx: &Shared, err: PsError, hint: impl Into<String>) -> Result<T, PsError> {
    ctx.borrow_mut().hint = Some(hint.into());
    Err(err)
}

pub(crate) fn reg<F>(interp: &mut Interp, ctx: &Shared, name: &str, f: F)
where
    F: Fn(&mut Interp, &Shared) -> OpResult + 'static,
{
    let ctx = ctx.clone();
    interp
        .register_operator(name, move |i| f(i, &ctx))
        .expect("psanim's operator names are valid and few");
}

/// An array of numbers as PostScript writes it, for `info`.
fn v_numbers(v: &Value) -> String {
    v.as_numbers().map_or_else(
        || "[...]".to_string(),
        |n| {
            let parts: Vec<String> = n.iter().map(|x| x.to_string()).collect();
            format!("[{}]", parts.join(" "))
        },
    )
}

/// Pop a procedure, kept opaque so it can be pushed back for the prelude
/// to `exec`.
fn pop_proc(i: &mut Interp, ctx: &Shared, who: &str) -> Result<Value, PsError> {
    match i.pop_value()? {
        Value::Other(o) if o.is_procedure() => Ok(Value::Other(o)),
        // A name is accepted where a procedure is: `/intro` runs `intro`.
        Value::Name(n) => match i.lookup(&n) {
            Some(Value::Other(o)) if o.is_procedure() => Ok(Value::Other(o)),
            _ => fail(
                ctx,
                PsError::Typecheck,
                format!("{who}: /{n} is not a procedure"),
            ),
        },
        _ => fail(
            ctx,
            PsError::Typecheck,
            format!("{who} needs a procedure {{ ... }} as its last operand"),
        ),
    }
}

/// Pop a noise key: a number, or an array of numbers folded to one number
/// that differs for different arrays. Every word that takes a key takes
/// both forms.
fn pop_key(i: &mut Interp, ctx: &Shared, usage: &str) -> Result<f64, PsError> {
    let key = match i.pop_value()? {
        Value::Array(a) => a
            .iter()
            .map(Value::as_f64)
            .collect::<Option<Vec<f64>>>()
            .map(|k| hash01(0, &k) * 1.0e6),
        v => v.as_f64(),
    };
    match key {
        Some(k) => Ok(k),
        None => fail(
            ctx,
            PsError::Typecheck,
            format!("{usage}, where key is a number or an array of numbers"),
        ),
    }
}

fn push_scope(ctx: &Shared, scope: TimeScope) -> OpResult {
    let mut c = ctx.borrow_mut();
    if c.scopes.len() >= MAX_SCOPES {
        c.hint = Some("time scopes nested too deeply (a combinator calling itself?)".into());
        return Err(PsError::Limitcheck);
    }
    c.scopes.push(scope);
    Ok(())
}

/// Linear interpolation of numbers or of equal-length number arrays.
pub(crate) fn lerp_value(a: &Value, b: &Value, u: f64) -> Option<Value> {
    match (a, b) {
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => x
            .iter()
            .zip(y)
            .map(|(p, q)| lerp_value(p, q, u))
            .collect::<Option<Vec<_>>>()
            .map(Value::Array),
        _ => {
            let (x, y) = (a.as_f64()?, b.as_f64()?);
            Some(Value::Real(x + (y - x) * u))
        }
    }
}

pub(crate) fn ease_named(ctx: &Shared, name: &str, x: f64) -> Result<f64, PsError> {
    match ease::ease(name, x) {
        Some(v) => Ok(v),
        None => fail(
            ctx,
            PsError::Rangecheck,
            format!(
                "unknown easing /{name}. Known: {}",
                ease::NAMES
                    .iter()
                    .map(|n| format!("/{n}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ),
    }
}

/// One keyframe: time, value, and the easing of the segment arriving at it.
struct Key<'a> {
    t: f64,
    v: &'a Value,
    ease: Option<&'a str>,
}

/// Parse `[t v /ease t v ...]`: a time, a value, and optionally the name of
/// the easing used to arrive at that value.
fn parse_keys<'a>(ctx: &Shared, items: &'a [Value]) -> Result<Vec<Key<'a>>, PsError> {
    let mut keys = Vec::new();
    let mut k = 0;
    while k < items.len() {
        let (Some(t), Some(v)) = (items[k].as_f64(), items.get(k + 1)) else {
            return fail(
                ctx,
                PsError::Typecheck,
                "keyframes: expected [time value /easing? time value ...]",
            );
        };
        k += 2;
        let ease = match items.get(k) {
            Some(Value::Name(n)) => {
                k += 1;
                Some(n.as_str())
            }
            _ => None,
        };
        if keys.last().is_some_and(|p: &Key| p.t > t) {
            return fail(
                ctx,
                PsError::Rangecheck,
                "keyframes: times must not decrease",
            );
        }
        keys.push(Key { t, v, ease });
    }
    if keys.is_empty() {
        return fail(ctx, PsError::Rangecheck, "keyframes: the array is empty");
    }
    Ok(keys)
}

fn eval_keys(ctx: &Shared, items: &[Value], t: f64) -> Result<Value, PsError> {
    let keys = parse_keys(ctx, items)?;
    if t <= keys[0].t {
        return Ok(keys[0].v.clone());
    }
    let Some(j) = keys.iter().position(|k| k.t > t) else {
        return Ok(keys[keys.len() - 1].v.clone());
    };
    let (a, b) = (&keys[j - 1], &keys[j]);
    let u = (t - a.t) / (b.t - a.t);
    let u = match b.ease {
        Some(name) => ease_named(ctx, name, u)?,
        None => u,
    };
    match lerp_value(a.v, b.v, u) {
        Some(v) => Ok(v),
        None => fail(
            ctx,
            PsError::Typecheck,
            "keyframes: values must be numbers, or arrays of numbers of one length",
        ),
    }
}

/// The current path, flattened, with a tolerance of about a device pixel.
pub(crate) fn flat_path(i: &Interp, ctx: &Shared) -> Result<Vec<pathmath::Poly>, PsError> {
    let path = i.current_path()?;
    let px = ctx.borrow().px_scale.max(1e-6);
    Ok(pathmath::flatten(&path, 1.5 / px))
}

/// Take the canvas out of the interpreter, run `f` on its pixels, put it
/// back.
pub(crate) fn with_canvas_pixels(
    i: &mut Interp,
    ctx: &Shared,
    f: impl FnOnce(&mut [u8], usize, usize, &mut Vec<u8>),
) -> OpResult {
    let (w, h) = i.canvas_size();
    let spare = ctx.borrow_mut().spare.take();
    let spare = match spare {
        Some(p) if (p.width(), p.height()) == (w, h) => p,
        _ => Page::transparent(w, h).ok_or(PsError::Limitcheck)?,
    };
    let mut canvas = i.swap_canvas(spare).map_err(|_| PsError::Rangecheck)?;
    let mut scratch = std::mem::take(&mut ctx.borrow_mut().scratch);
    f(
        canvas.premultiplied_rgba8_mut(),
        w as usize,
        h as usize,
        &mut scratch,
    );
    let spare = i.swap_canvas(canvas).map_err(|_| PsError::Rangecheck)?;
    let mut c = ctx.borrow_mut();
    c.scratch = scratch;
    c.spare = Some(spare);
    Ok(())
}

pub(crate) fn begin_layer(i: &mut Interp, ctx: &Shared) -> OpResult {
    if ctx.borrow().layers.len() >= MAX_LAYERS {
        return fail(
            ctx,
            PsError::Limitcheck,
            format!("more than {MAX_LAYERS} nested layers"),
        );
    }
    let (w, h) = i.canvas_size();
    let spare = ctx.borrow_mut().spare.take();
    let layer = match spare {
        Some(mut p) if (p.width(), p.height()) == (w, h) => {
            p.premultiplied_rgba8_mut().fill(0);
            p
        }
        _ => Page::transparent(w, h).ok_or(PsError::Limitcheck)?,
    };
    let below = i.swap_canvas(layer).map_err(|_| PsError::Rangecheck)?;
    ctx.borrow_mut().layers.push(below);
    Ok(())
}

/// Start a layer whose ending is decided now and applied by `.layerend`.
fn begin_pending(i: &mut Interp, ctx: &Shared, end: LayerEnd) -> OpResult {
    begin_layer(i, ctx)?;
    ctx.borrow_mut().pending.push(end);
    Ok(())
}

const PLAIN: LayerEnd = LayerEnd {
    opacity: 1.0,
    blend: Blend::Normal,
    blur: 0.0,
    glow: (0.0, 0.0),
};

fn blend_named(ctx: &Shared, name: &str) -> Result<Blend, PsError> {
    match Blend::from_name(name) {
        Some(b) => Ok(b),
        None => fail(
            ctx,
            PsError::Rangecheck,
            format!("unknown blend mode /{name}. Known: /normal /add /screen /multiply"),
        ),
    }
}

fn end_layer(i: &mut Interp, ctx: &Shared, opacity: f64, blend: Blend) -> OpResult {
    let Some(mut below) = ctx.borrow_mut().layers.pop() else {
        return fail(
            ctx,
            PsError::Rangecheck,
            "endlayer without a matching beginlayer",
        );
    };
    raster::composite(
        below.premultiplied_rgba8_mut(),
        i.canvas_premultiplied_rgba8(),
        opacity,
        blend,
    );
    let layer = i.swap_canvas(below).map_err(|_| PsError::Rangecheck)?;
    // The finished layer is the right size to be the next spare.
    ctx.borrow_mut().spare = Some(layer);
    Ok(())
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    crate::vocab::install(interp, ctx);
    crate::catalog::install(interp, ctx);
    // --- declaration ----------------------------------------------------
    reg(interp, ctx, ".animation", |i, ctx| {
        let decl = i.pop_value()?;
        if !matches!(decl, Value::Dict(_)) {
            return fail(
                ctx,
                PsError::Typecheck,
                "animation takes a dictionary: << /Size [1280 720] /Duration 6 /FPS 30 >> animation",
            );
        }
        let mut c = ctx.borrow_mut();
        if c.running != Running::Probe && c.running != Running::Setup {
            c.hint = Some(
                "animation declares the movie; call it once, at the top of the program".into(),
            );
            return Err(PsError::Rangecheck);
        }
        c.decl = Some(decl);
        let stop = c.probing;
        drop(c);
        i.push_bool(stop);
        Ok(())
    });

    // --- the clock ------------------------------------------------------
    reg(interp, ctx, "now", |i, ctx| {
        i.push_real(ctx.borrow().scope().t);
        Ok(())
    });
    reg(interp, ctx, "progress", |i, ctx| {
        let s = ctx.borrow().scope();
        let u = if s.dur > 0.0 && s.dur.is_finite() {
            (s.t / s.dur).clamp(0.0, 1.0)
        } else {
            0.0
        };
        i.push_real(u);
        Ok(())
    });
    reg(interp, ctx, "span", |i, ctx| {
        i.push_real(ctx.borrow().scope().dur);
        Ok(())
    });
    reg(interp, ctx, "globaltime", |i, ctx| {
        i.push_real(ctx.borrow().movie_clock().t);
        Ok(())
    });
    reg(interp, ctx, "frame", |i, ctx| {
        i.push_int(ctx.borrow().frame);
        Ok(())
    });
    reg(interp, ctx, "fps", |i, ctx| {
        i.push_real(ctx.borrow().fps);
        Ok(())
    });
    reg(interp, ctx, "dt", |i, ctx| {
        i.push_real(ctx.borrow().dt);
        Ok(())
    });
    reg(interp, ctx, "duration", |i, ctx| {
        i.push_real(ctx.borrow().movie_clock().dur);
        Ok(())
    });
    reg(interp, ctx, "width", |i, ctx| {
        let c = ctx.borrow();
        i.push_real(c.views.last().map_or(c.width, |v| v.0));
        Ok(())
    });
    reg(interp, ctx, "height", |i, ctx| {
        let c = ctx.borrow();
        i.push_real(c.views.last().map_or(c.height, |v| v.1));
        Ok(())
    });
    reg(interp, ctx, "movieseed", |i, ctx| {
        i.push_int(ctx.borrow().seed());
        Ok(())
    });

    // --- time scopes (native halves; see prelude.ps) -----------------------
    // Each leaves `proc true` to run the body in the scope it pushed, or
    // `false` when the body is not live now.
    reg(interp, ctx, ".during", |i, ctx| {
        let body = pop_proc(i, ctx, "during")?;
        let (b, a) = (i.pop_f64()?, i.pop_f64()?);
        let s = ctx.borrow().scope();
        if s.t >= a && s.t < b {
            push_scope(
                ctx,
                TimeScope {
                    t: s.t - a,
                    dur: b - a,
                },
            )?;
            i.push_value(&body)?;
            i.push_bool(true);
        } else {
            i.push_bool(false);
        }
        Ok(())
    });
    reg(interp, ctx, ".after", |i, ctx| {
        let body = pop_proc(i, ctx, "after")?;
        let a = i.pop_f64()?;
        let s = ctx.borrow().scope();
        if s.t >= a {
            push_scope(
                ctx,
                TimeScope {
                    t: s.t - a,
                    dur: (s.dur - a).max(0.0),
                },
            )?;
            i.push_value(&body)?;
            i.push_bool(true);
        } else {
            i.push_bool(false);
        }
        Ok(())
    });
    reg(interp, ctx, ".before", |i, ctx| {
        let body = pop_proc(i, ctx, "before")?;
        let b = i.pop_f64()?;
        let s = ctx.borrow().scope();
        if s.t < b {
            push_scope(ctx, TimeScope { t: s.t, dur: b })?;
            i.push_value(&body)?;
            i.push_bool(true);
        } else {
            i.push_bool(false);
        }
        Ok(())
    });
    reg(interp, ctx, ".delayed", |i, ctx| {
        let body = pop_proc(i, ctx, "delayed")?;
        let d = i.pop_f64()?;
        let s = ctx.borrow().scope();
        push_scope(
            ctx,
            TimeScope {
                t: s.t - d,
                dur: s.dur - d,
            },
        )?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".timescaled", |i, ctx| {
        let body = pop_proc(i, ctx, "timescaled")?;
        let k = i.pop_f64()?;
        let s = ctx.borrow().scope();
        push_scope(
            ctx,
            TimeScope {
                t: s.t * k,
                dur: s.dur * k.abs(),
            },
        )?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".looped", |i, ctx| {
        let body = pop_proc(i, ctx, "looped")?;
        let p = i.pop_f64()?;
        if p.is_nan() || p <= 0.0 {
            return fail(
                ctx,
                PsError::Rangecheck,
                "looped: the period must be positive",
            );
        }
        let s = ctx.borrow().scope();
        push_scope(
            ctx,
            TimeScope {
                t: s.t.rem_euclid(p),
                dur: p,
            },
        )?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".pingpong", |i, ctx| {
        let body = pop_proc(i, ctx, "pingpong")?;
        let p = i.pop_f64()?;
        if p.is_nan() || p <= 0.0 {
            return fail(
                ctx,
                PsError::Rangecheck,
                "pingpong: the period must be positive",
            );
        }
        let s = ctx.borrow().scope();
        let phase = s.t.rem_euclid(2.0 * p);
        let t = if phase < p { phase } else { 2.0 * p - phase };
        push_scope(ctx, TimeScope { t, dur: p })?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".attime", |i, ctx| {
        let body = pop_proc(i, ctx, "attime")?;
        let t = i.pop_f64()?;
        let s = ctx.borrow().scope();
        push_scope(ctx, TimeScope { t, dur: s.dur })?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".timewindow", |i, ctx| {
        let body = pop_proc(i, ctx, "timewindow")?;
        let (dur, t) = (i.pop_f64()?, i.pop_f64()?);
        push_scope(ctx, TimeScope { t, dur })?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".globally", |i, ctx| {
        let body = pop_proc(i, ctx, "globally")?;
        let clock = ctx.borrow().movie_clock();
        push_scope(ctx, clock)?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".reversed", |i, ctx| {
        let body = pop_proc(i, ctx, "reversed")?;
        let s = ctx.borrow().scope();
        push_scope(
            ctx,
            TimeScope {
                t: s.dur - s.t,
                dur: s.dur,
            },
        )?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".timepop", |_, ctx| {
        let mut c = ctx.borrow_mut();
        if c.scopes.len() > 1 {
            c.scopes.pop();
        }
        Ok(())
    });
    // [ dur proc dur proc ... ] .sequence -> proc true | false
    reg(interp, ctx, ".sequence", |i, ctx| {
        let Value::Array(items) = i.pop_value()? else {
            return fail(
                ctx,
                PsError::Typecheck,
                "sequence takes an array: [ dur {..} dur {..} ]",
            );
        };
        if items.len() % 2 != 0 {
            return fail(
                ctx,
                PsError::Rangecheck,
                "sequence: the array must be dur/procedure pairs",
            );
        }
        let s = ctx.borrow().scope();
        let mut start = 0.0;
        for pair in items.chunks(2) {
            let Some(dur) = pair[0].as_f64() else {
                return fail(
                    ctx,
                    PsError::Typecheck,
                    "sequence: each pair starts with a duration in seconds",
                );
            };
            if s.t >= start && s.t < start + dur {
                if matches!(pair[1], Value::Null) {
                    break; // a gap: nothing plays
                }
                i.push_value(&pair[1])?;
                let body = pop_proc(i, ctx, "sequence")?;
                push_scope(
                    ctx,
                    TimeScope {
                        t: s.t - start,
                        dur,
                    },
                )?;
                i.push_value(&body)?;
                i.push_bool(true);
                return Ok(());
            }
            start += dur;
        }
        i.push_bool(false);
        Ok(())
    });
    // [ [dur proc] [dur /transition] [dur proc] ... ] .reel
    //   -> 0                                   nothing live
    //    | proc 1                              one clip, its scope pushed
    //    | kind u tB dB procB tA dA procA 2    a transition from A to B
    reg(interp, ctx, ".reel", |i, ctx| {
        let Value::Array(items) = i.pop_value()? else {
            return fail(
                ctx,
                PsError::Typecheck,
                "reel takes an array of [dur proc] and [dur /transition]",
            );
        };
        let t = ctx.borrow().scope().t;
        let plan = match plan_reel(i, &items, true) {
            Ok(p) => p,
            Err(msg) => return fail(ctx, PsError::Rangecheck, msg),
        };
        for (k, clip) in plan.iter().enumerate() {
            // A transition into this clip covers [start, start + lead).
            if let Some((kind, lead)) = &clip.lead
                && k > 0
                && t >= clip.start
                && t < clip.start + lead
            {
                let prev = &plan[k - 1];
                i.push_name(kind);
                i.push_real((t - clip.start) / lead);
                i.push_real(t - clip.start);
                i.push_real(clip.dur);
                i.push_value(clip.body)?;
                i.push_real(t - prev.start);
                i.push_real(prev.dur);
                i.push_value(prev.body)?;
                i.push_int(2);
                return Ok(());
            }
            if t >= clip.start && t < clip.start + clip.dur {
                // Not in this clip's lead-in, and the next clip's lead-in
                // (which overlaps this one's end) is tested on its turn.
                let next_lead = plan
                    .get(k + 1)
                    .is_some_and(|n| n.lead.is_some() && t >= n.start);
                if !next_lead {
                    i.push_value(clip.body)?;
                    let body = pop_proc(i, ctx, "reel")?;
                    push_scope(
                        ctx,
                        TimeScope {
                            t: t - clip.start,
                            dur: clip.dur,
                        },
                    )?;
                    i.push_value(&body)?;
                    i.push_int(1);
                    return Ok(());
                }
            }
        }
        i.push_int(0);
        Ok(())
    });
    reg(interp, ctx, "reelduration", |i, ctx| {
        let Value::Array(items) = i.pop_value()? else {
            return fail(ctx, PsError::Typecheck, "reelduration takes a reel array");
        };
        match plan_reel(i, &items, false) {
            Ok(plan) => {
                i.push_real(plan.last().map_or(0.0, |c| c.start + c.dur));
                Ok(())
            }
            Err(msg) => fail(ctx, PsError::Rangecheck, msg),
        }
    });

    // --- values over time ---------------------------------------------------
    reg(interp, ctx, "ease", |i, ctx| {
        let name = i.pop_name()?;
        let x = i.pop_f64()?;
        let v = ease_named(ctx, &name, x)?;
        i.push_real(v);
        Ok(())
    });
    reg(interp, ctx, "tween", |i, ctx| {
        let name = i.pop_name()?;
        let (b, a) = (i.pop_f64()?, i.pop_f64()?);
        let t = ctx.borrow().scope().t;
        let u = if b > a {
            (t - a) / (b - a)
        } else if t >= b {
            1.0
        } else {
            0.0
        };
        let v = ease_named(ctx, &name, u)?;
        i.push_real(v);
        Ok(())
    });
    reg(interp, ctx, "ramp", |i, ctx| {
        let (b, a) = (i.pop_f64()?, i.pop_f64()?);
        let t = ctx.borrow().scope().t;
        let u = if b > a {
            ((t - a) / (b - a)).clamp(0.0, 1.0)
        } else if t >= b {
            1.0
        } else {
            0.0
        };
        i.push_real(u);
        Ok(())
    });
    reg(interp, ctx, "between", |i, ctx| {
        let (b, a) = (i.pop_value()?, i.pop_value()?);
        let u = i.pop_f64()?;
        match lerp_value(&a, &b, u) {
            Some(v) => {
                i.push_value(&v)?;
                Ok(())
            }
            None => fail(
                ctx,
                PsError::Typecheck,
                "between: u a b, where a and b are numbers or arrays of numbers of one length",
            ),
        }
    });
    reg(interp, ctx, "clamp", |i, _| {
        let (hi, lo, x) = (i.pop_f64()?, i.pop_f64()?, i.pop_f64()?);
        i.push_real(x.max(lo).min(hi));
        Ok(())
    });
    reg(interp, ctx, "fmod", |i, ctx| {
        let (m, x) = (i.pop_f64()?, i.pop_f64()?);
        if m == 0.0 {
            return fail(ctx, PsError::UndefinedResult, "fmod: the modulus is 0");
        }
        // Always in 0..m for a positive m, unlike a remainder: -0.25 1 fmod is 0.75.
        i.push_real(x.rem_euclid(m));
        Ok(())
    });
    reg(interp, ctx, "smoothstep", |i, _| {
        let (x, e1, e0) = (i.pop_f64()?, i.pop_f64()?, i.pop_f64()?);
        let u = if e1 != e0 {
            ((x - e0) / (e1 - e0)).clamp(0.0, 1.0)
        } else if x >= e1 {
            1.0
        } else {
            0.0
        };
        i.push_real(u * u * (3.0 - 2.0 * u));
        Ok(())
    });
    reg(interp, ctx, "remap", |i, _| {
        let (d, c, b, a, x) = (
            i.pop_f64()?,
            i.pop_f64()?,
            i.pop_f64()?,
            i.pop_f64()?,
            i.pop_f64()?,
        );
        let u = if b != a { (x - a) / (b - a) } else { 0.0 };
        i.push_real(c + (d - c) * u);
        Ok(())
    });
    reg(interp, ctx, "keyframes", |i, ctx| {
        let Value::Array(items) = i.pop_value()? else {
            return fail(
                ctx,
                PsError::Typecheck,
                "keyframes takes an array: [ t v /easing t v ... ]",
            );
        };
        let t = ctx.borrow().scope().t;
        let v = eval_keys(ctx, &items, t)?;
        i.push_value(&v)?;
        Ok(())
    });
    reg(interp, ctx, "keyat", |i, ctx| {
        let Value::Array(items) = i.pop_value()? else {
            return fail(ctx, PsError::Typecheck, "keyat: t [ t v /easing t v ... ]");
        };
        let t = i.pop_f64()?;
        let v = eval_keys(ctx, &items, t)?;
        i.push_value(&v)?;
        Ok(())
    });
    reg(interp, ctx, "wiggle", |i, ctx| {
        let (amp, freq) = (i.pop_f64()?, i.pop_f64()?);
        let key = pop_key(i, ctx, "wiggle: key freq amp")?;
        let c = ctx.borrow();
        i.push_real(wiggle(c.seed(), key, c.scope().t * freq) * amp);
        Ok(())
    });
    reg(interp, ctx, "turns", |i, ctx| {
        let k = i.pop_f64()?;
        let s = ctx.borrow().scope();
        let u = if s.dur > 0.0 && s.dur.is_finite() {
            s.t / s.dur
        } else {
            0.0
        };
        i.push_real(k * 360.0 * u);
        Ok(())
    });
    reg(interp, ctx, "loopnoise", |i, ctx| {
        let radius = i.pop_f64()?;
        let key = pop_key(i, ctx, "loopnoise: key radius")?;
        let c = ctx.borrow();
        let s = c.scope();
        let a = if s.dur > 0.0 && s.dur.is_finite() {
            s.t / s.dur
        } else {
            0.0
        } * std::f64::consts::TAU;
        i.push_real(crate::ctx::noise2(
            c.seed(),
            key,
            radius * a.cos(),
            radius * a.sin(),
        ));
        Ok(())
    });
    reg(interp, ctx, "hash01", |i, ctx| {
        let keys = match i.pop_value()? {
            Value::Array(a) => a.iter().map(Value::as_f64).collect::<Option<Vec<f64>>>(),
            v => v.as_f64().map(|x| vec![x]),
        };
        let Some(keys) = keys else {
            return fail(
                ctx,
                PsError::Typecheck,
                "hash01 takes a number or an array of numbers as its key",
            );
        };
        i.push_real(hash01(ctx.borrow().seed(), &keys));
        Ok(())
    });

    // --- paths ----------------------------------------------------------------
    reg(interp, ctx, "pathlength", |i, ctx| {
        let polys = flat_path(i, ctx)?;
        i.push_real(pathmath::total_length(&polys));
        Ok(())
    });
    reg(interp, ctx, "pathpoint", |i, ctx| {
        let u = i.pop_f64()?;
        let polys = flat_path(i, ctx)?;
        let total = pathmath::total_length(&polys);
        let Some((x, y, a)) = pathmath::point_at(&polys, u.clamp(0.0, 1.0) * total) else {
            return fail(
                ctx,
                PsError::NoCurrentPoint,
                "pathpoint: there is no current path to walk along",
            );
        };
        i.push_real(x);
        i.push_real(y);
        i.push_real(a);
        Ok(())
    });
    reg(interp, ctx, "trimpath", |i, ctx| {
        let (u1, u0) = (i.pop_f64()?, i.pop_f64()?);
        if u0 <= 0.0 && u1 >= 1.0 {
            return Ok(()); // the whole path, curves intact
        }
        let polys = flat_path(i, ctx)?;
        i.set_current_path(&pathmath::trim(&polys, u0, u1))
    });
    reg(interp, ctx, "trimeach", |i, ctx| {
        let (u1, u0) = (i.pop_f64()?, i.pop_f64()?);
        if u0 <= 0.0 && u1 >= 1.0 {
            return Ok(());
        }
        let polys = flat_path(i, ctx)?;
        i.set_current_path(&pathmath::trim_each(&polys, u0, u1))
    });
    reg(interp, ctx, "resamplepath", |i, ctx| {
        let n = i.pop_int()?;
        if !(2..=20_000).contains(&n) {
            return fail(
                ctx,
                PsError::Rangecheck,
                "resamplepath: the point count must be between 2 and 20000",
            );
        }
        let polys = flat_path(i, ctx)?;
        if polys.is_empty() {
            return fail(
                ctx,
                PsError::NoCurrentPoint,
                "resamplepath: there is no current path",
            );
        }
        i.push_numbers(&pathmath::resample(&polys, n as usize));
        Ok(())
    });
    reg(interp, ctx, "morphpath", |i, ctx| {
        let u = i.pop_f64()?;
        let (b, a) = (i.pop_numbers()?, i.pop_numbers()?);
        if a.len() != b.len() || a.len() < 4 || a.len() % 2 != 0 {
            return fail(
                ctx,
                PsError::Rangecheck,
                "morphpath: both arrays must hold the same number of x y pairs (use the same count with resamplepath)",
            );
        }
        let mut path = Vec::with_capacity(a.len() / 2 + 1);
        for (k, (p, q)) in a.chunks(2).zip(b.chunks(2)).enumerate() {
            let (x, y) = (p[0] + (q[0] - p[0]) * u, p[1] + (q[1] - p[1]) * u);
            path.push(if k == 0 {
                PathSegment::MoveTo(x, y)
            } else {
                PathSegment::LineTo(x, y)
            });
        }
        path.push(PathSegment::Close);
        i.set_current_path(&path)
    });

    reg(interp, ctx, "polypath", |i, ctx| {
        let pts = i.pop_numbers()?;
        if pts.len() < 4 || pts.len() % 2 != 0 {
            return fail(
                ctx,
                PsError::Rangecheck,
                "polypath takes [x0 y0 x1 y1 ...] with at least two points",
            );
        }
        let path: Vec<PathSegment> = pts
            .chunks(2)
            .enumerate()
            .map(|(k, p)| {
                if k == 0 {
                    PathSegment::MoveTo(p[0], p[1])
                } else {
                    PathSegment::LineTo(p[0], p[1])
                }
            })
            .collect();
        i.set_current_path(&path)
    });

    // --- layers -----------------------------------------------------------------
    reg(interp, ctx, "beginlayer", begin_layer);
    // The native halves of faded / blended / blurred / glowing: take the
    // operands now, so the body runs with the caller's stack on top.
    reg(interp, ctx, ".fadebegin", |i, ctx| {
        let opacity = i.pop_f64()?;
        begin_pending(i, ctx, LayerEnd { opacity, ..PLAIN })
    });
    reg(interp, ctx, ".blendbegin", |i, ctx| {
        let name = i.pop_name()?;
        let opacity = i.pop_f64()?;
        let blend = blend_named(ctx, &name)?;
        begin_pending(
            i,
            ctx,
            LayerEnd {
                opacity,
                blend,
                ..PLAIN
            },
        )
    });
    reg(interp, ctx, ".blurbegin", |i, ctx| {
        let blur = i.pop_f64()? * ctx.borrow().unit_scale();
        begin_pending(i, ctx, LayerEnd { blur, ..PLAIN })
    });
    reg(interp, ctx, ".glowbegin", |i, ctx| {
        let (strength, radius) = (i.pop_f64()?, i.pop_f64()?);
        let radius = radius * ctx.borrow().unit_scale();
        begin_pending(
            i,
            ctx,
            LayerEnd {
                glow: (radius, strength),
                ..PLAIN
            },
        )
    });
    reg(interp, ctx, ".layerend", |i, ctx| {
        let Some(end) = ctx.borrow_mut().pending.pop() else {
            return fail(
                ctx,
                PsError::Rangecheck,
                "a layer combinator ended without having begun",
            );
        };
        let px = ctx.borrow().px_scale;
        if end.blur > 0.0 {
            with_canvas_pixels(i, ctx, |p, w, h, scratch| {
                raster::blur(p, w, h, end.blur * px, scratch)
            })?;
        }
        if end.glow.1 > 0.0 {
            with_canvas_pixels(i, ctx, |p, w, h, scratch| {
                raster::glow(
                    p,
                    w,
                    h,
                    end.glow.0 * px,
                    end.glow.1.clamp(0.0, 2.0),
                    scratch,
                )
            })?;
        }
        end_layer(i, ctx, end.opacity, end.blend)
    });
    reg(interp, ctx, "endlayer", |i, ctx| {
        let opacity = i.pop_f64()?;
        end_layer(i, ctx, opacity, Blend::Normal)
    });
    reg(interp, ctx, "endlayerblend", |i, ctx| {
        let name = i.pop_name()?;
        let opacity = i.pop_f64()?;
        let blend = blend_named(ctx, &name)?;
        end_layer(i, ctx, opacity, blend)
    });
    reg(interp, ctx, "blurlayer", |i, ctx| {
        let radius = i.pop_f64()?;
        let sigma = radius * ctx.borrow().px_scale * ctx.borrow().unit_scale();
        with_canvas_pixels(i, ctx, |px, w, h, scratch| {
            raster::blur(px, w, h, sigma, scratch)
        })
    });
    reg(interp, ctx, "glowlayer", |i, ctx| {
        let (strength, radius) = (i.pop_f64()?, i.pop_f64()?);
        let sigma = radius * ctx.borrow().px_scale * ctx.borrow().unit_scale();
        with_canvas_pixels(i, ctx, |px, w, h, scratch| {
            raster::glow(px, w, h, sigma, strength.clamp(0.0, 2.0), scratch)
        })
    });
    reg(interp, ctx, "vignette", |i, ctx| {
        let amount = i.pop_f64()?.clamp(0.0, 1.0);
        with_canvas_pixels(i, ctx, |px, w, h, _| {
            let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
            let rmax2 = cx * cx + cy * cy;
            for y in 0..h {
                for x in 0..w {
                    let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
                    let r2 = (dx * dx + dy * dy) / rmax2;
                    // Untouched in the middle, darkest in the corners.
                    let k = 1.0 - amount * r2 * r2.sqrt();
                    let p = &mut px[(y * w + x) * 4..(y * w + x) * 4 + 3];
                    for c in p {
                        *c = (f64::from(*c) * k).round() as u8;
                    }
                }
            }
        })
    });
    reg(interp, ctx, "filmgrain", |i, ctx| {
        let amount = i.pop_f64()?.clamp(0.0, 1.0);
        let (seed, frame) = {
            let c = ctx.borrow();
            (c.seed, c.frame)
        };
        with_canvas_pixels(i, ctx, |px, _, _, _| {
            // One cheap xorshift stream per frame: grain that moves.
            let mut s =
                crate::ctx::mix64(seed as u64 ^ (frame as u64).wrapping_mul(0x9E37_79B9)) | 1;
            let amp = amount * 64.0;
            for p in px.chunks_exact_mut(4) {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                let n = ((s >> 40) as f64 / (1u64 << 24) as f64 - 0.5) * amp;
                let a = p[3];
                for c in &mut p[..3] {
                    *c = ((f64::from(*c) + n).clamp(0.0, 255.0) as u8).min(a);
                }
            }
        })
    });

    // --- inputs -------------------------------------------------------------------
    reg(interp, ctx, "param", |i, ctx| {
        let default = i.pop_value()?;
        let name = i.pop_name()?;
        let mut c = ctx.borrow_mut();
        // Inside a component, its own parameters and nothing of the host's.
        if let Some(comp) = c.comps.last() {
            let v = comp
                .params
                .iter()
                .find(|(k, _)| *k == name)
                .map_or(default, |(_, v)| v.clone());
            drop(c);
            i.push_value(&v)?;
            return Ok(());
        }
        let shown = match &default {
            Value::String(s) => format!("({})", String::from_utf8_lossy(s)),
            Value::Name(n) => format!("/{n}"),
            Value::Array(_) => v_numbers(&default),
            v => v
                .as_f64()
                .map_or_else(|| "?".to_string(), |x| x.to_string()),
        };
        c.params_seen.entry(name.clone()).or_insert(shown);
        let v = c.params.get(&name).cloned().unwrap_or(default);
        drop(c);
        i.push_value(&v)?;
        Ok(())
    });
    // --- components (native halves; see prelude.ps) ----------------------
    // t dur seed params unit .comppush -    open a component's clock: its
    //                                  movie time t of dur, its seed, the
    //                                  values its `param` lookups get (a
    //                                  dictionary, or null for none), and
    //                                  the device length of its unit.
    reg(interp, ctx, ".comppush", |i, ctx| {
        let unit = i.pop_f64()?;
        let params = match i.pop_value()? {
            Value::Dict(entries) => entries,
            Value::Null => Vec::new(),
            _ => {
                return fail(
                    ctx,
                    PsError::Typecheck,
                    "a component's /Params must be a dictionary: << /Height 0.8 >>",
                );
            }
        };
        let seed = i.pop_int()?;
        let dur = i.pop_f64()?;
        let t = i.pop_f64()?;
        let mut c = ctx.borrow_mut();
        if c.comps.len() >= crate::ctx::MAX_COMPS {
            c.hint = Some(format!(
                "components are nested more than {} deep: is a component placing itself?",
                crate::ctx::MAX_COMPS
            ));
            return Err(PsError::Limitcheck);
        }
        let scale = if c.px_scale > 0.0 && unit.is_finite() && unit > 0.0 {
            unit / c.px_scale
        } else {
            1.0
        };
        c.comps.push(crate::ctx::Comp {
            t,
            dur,
            seed,
            params,
            scale,
        });
        drop(c);
        push_scope(ctx, TimeScope { t, dur })
    });
    // dur seed .compset -     the component's declaration has been read:
    //                         set its length and seed.
    reg(interp, ctx, ".compset", |i, ctx| {
        let seed = i.pop_int()?;
        let dur = i.pop_f64()?;
        let mut c = ctx.borrow_mut();
        let Some(comp) = c.comps.last_mut() else {
            return Err(PsError::Rangecheck);
        };
        comp.dur = dur;
        comp.seed = seed;
        let t = comp.t;
        if let Some(s) = c.scopes.last_mut() {
            *s = TimeScope { t, dur };
        }
        Ok(())
    });
    reg(interp, ctx, ".comppop", |_, ctx| {
        let mut c = ctx.borrow_mut();
        if c.comps.pop().is_some() && c.scopes.len() > 1 {
            c.scopes.pop();
        }
        Ok(())
    });
    // (why) .comperror -     fail with a rangecheck that explains why.
    reg(interp, ctx, ".comperror", |i, ctx| {
        let why = i.pop_string()?;
        fail(ctx, PsError::Rangecheck, String::from_utf8_lossy(&why))
    });
    reg(interp, ctx, ".compdepth", |i, ctx| {
        let n = ctx.borrow().comps.len();
        i.push_int(n as i64);
        Ok(())
    });
    // (path) .comppath (path)  a relative path read from the movie's own
    //                          directory, when the host said what it is.
    reg(interp, ctx, ".comppath", |i, ctx| {
        let path = match i.pop_value()? {
            Value::String(s) => String::from_utf8_lossy(&s).into_owned(),
            Value::Name(n) => n,
            _ => {
                return fail(
                    ctx,
                    PsError::Typecheck,
                    "loadcomponent takes a file name as a string: /fire (fire.ps) loadcomponent",
                );
            }
        };
        let p = std::path::Path::new(&path);
        let full = match &ctx.borrow().base_dir {
            Some(dir) if p.is_relative() => dir.join(p),
            _ => p.to_path_buf(),
        };
        i.push_string(full.to_string_lossy().as_bytes());
        Ok(())
    });

    reg(interp, ctx, "signal", |i, ctx| {
        let name = i.pop_name()?;
        let c = ctx.borrow();
        i.push_real(c.signals.value(&name, c.global_t).unwrap_or(0.0));
        Ok(())
    });
    reg(interp, ctx, "sinceevent", |i, ctx| {
        let name = i.pop_name()?;
        let c = ctx.borrow();
        // "Long ago" when there has been none, so decay formulas need no case.
        i.push_real(c.signals.last_event(&name, c.global_t).0.unwrap_or(1.0e6));
        Ok(())
    });
    reg(interp, ctx, "eventcount", |i, ctx| {
        let name = i.pop_name()?;
        let c = ctx.borrow();
        i.push_int(c.signals.last_event(&name, c.global_t).1 as i64);
        Ok(())
    });

    // A movie has no pages: `showpage` inside a scene would otherwise copy
    // the canvas aside and arm an erase.
    reg(interp, ctx, "showpage", |_, _| Ok(()));
    reg(interp, ctx, "copypage", |_, _| Ok(()));
    // Wall-clock time would make a render irreproducible; both read the
    // movie's clock instead, in milliseconds as the originals do.
    reg(interp, ctx, "realtime", |i, ctx| {
        i.push_int((ctx.borrow().global_t * 1000.0) as i64);
        Ok(())
    });
    reg(interp, ctx, "usertime", |i, ctx| {
        i.push_int((ctx.borrow().global_t * 1000.0) as i64);
        Ok(())
    });
}

/// One clip of a reel, placed on the timeline.
pub(crate) struct ReelClip<'a> {
    pub start: f64,
    pub dur: f64,
    pub body: &'a Value,
    /// The transition that leads into this clip and how long it lasts; it
    /// overlaps the end of the previous clip.
    pub lead: Option<(String, f64)>,
}

/// Plan a reel against the interpreter's current definitions: a name is a
/// transition if the prelude's `transitions` dictionary has it (so one a
/// program adds is recognised), else it must be a procedure.
///
/// `strict` also requires every clip named to be a procedure now. Playing a
/// reel is strict; measuring one (`reelduration`, usually in the
/// declaration, before the clips are defined) is not.
fn plan_reel<'a>(
    i: &Interp,
    items: &'a [Value],
    strict: bool,
) -> Result<Vec<ReelClip<'a>>, String> {
    let known: Vec<String> = match i.lookup("transitions") {
        Some(Value::Dict(entries)) => entries.into_iter().map(|(k, _)| k).collect(),
        _ => TRANSITIONS.iter().map(|t| t.to_string()).collect(),
    };
    reel_plan(
        items,
        // A name starting with `t-` is taken for a transition even before
        // it is defined, so a reel can be measured in the declaration with
        // its own transitions defined further down the file.
        &|n| known.iter().any(|k| k == n) || n.starts_with("t-"),
        &|n| !strict || matches!(i.lookup(n), Some(Value::Other(o)) if o.is_procedure()),
    )
}

/// Lay a reel's clips out in time: each clip follows the last, pulled
/// earlier by the length of the transition between them.
pub(crate) fn reel_plan<'a>(
    items: &'a [Value],
    is_transition: &dyn Fn(&str) -> bool,
    is_procedure: &dyn Fn(&str) -> bool,
) -> Result<Vec<ReelClip<'a>>, String> {
    let mut plan: Vec<ReelClip> = Vec::new();
    let mut pending: Option<(String, f64)> = None;
    let mut cursor = 0.0;
    for item in items {
        let Some([d, what]) = item
            .as_array()
            .and_then(|a| <&[Value; 2]>::try_from(a).ok())
        else {
            return Err("reel: each entry is [dur {clip}] or [dur /transition]".into());
        };
        let Some(dur) = d.as_f64().filter(|d| *d > 0.0) else {
            return Err("reel: each entry starts with a positive duration in seconds".into());
        };
        // A transition is named by one of the names in the prelude's
        // `transitions` dictionary; anything else names a clip procedure.
        match what {
            Value::Name(n) if is_transition(n) => {
                if plan.is_empty() || pending.is_some() {
                    return Err("reel: a transition must come between two clips".into());
                }
                pending = Some((n.clone(), dur));
            }
            _ => {
                match what {
                    Value::Name(n) if !is_procedure(n) => {
                        return Err(format!(
                            "reel: /{n} is neither a transition (a key of the `transitions` dictionary) nor a procedure"
                        ));
                    }
                    Value::Name(_) => {}
                    Value::Other(o) if o.is_procedure() => {}
                    _ => return Err("reel: the second element of an entry is a clip procedure or a transition's name".into()),
                }
                let lead = pending.take();
                let overlap = lead.as_ref().map_or(0.0, |(_, d)| *d);
                if let Some(prev) = plan.last()
                    && (overlap > prev.dur || overlap > dur)
                {
                    return Err("reel: a transition is longer than a clip beside it".into());
                }
                let start = cursor - overlap;
                plan.push(ReelClip {
                    start,
                    dur,
                    body: what,
                    lead,
                });
                cursor = start + dur;
            }
        }
    }
    if pending.is_some() {
        return Err("reel: the last entry is a transition with no clip after it".into());
    }
    Ok(plan)
}

/// The transitions the prelude defines. A program adds its own by putting
/// a procedure in the `transitions` dictionary under any name.
pub const TRANSITIONS: &[&str] = &[
    "cut",
    "dissolve",
    "dip",
    "wipe-left",
    "wipe-right",
    "wipe-up",
    "wipe-down",
    "iris",
    "push-left",
    "push-right",
    "zoom",
];
