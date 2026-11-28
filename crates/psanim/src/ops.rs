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

use crate::ctx::{
    ClipMask, Ctx, FadeGroup, LayerEnd, MAX_LAYERS, MAX_SCOPES, Running, TimeScope, hash01, wiggle,
};
use crate::ease;
use crate::groups::{groups_open, innermost_group};
use crate::pathmath;
use crate::plane::{self, Plane};
use crate::raster::{self, Blend};

pub(crate) type Shared = Rc<RefCell<Ctx>>;
pub(crate) type OpResult = Result<(), PsError>;

/// Fail with `err`, leaving `hint` for the engine's error report.
pub(crate) fn fail<T>(ctx: &Shared, err: PsError, hint: impl Into<String>) -> Result<T, PsError> {
    ctx.borrow_mut().hint = Some(hint.into());
    Err(err)
}

/// Fail as psanim's `groupnesting`: a layer and a transparency group were
/// nested in a way that would draw the wrong picture.
pub(crate) fn group_fail<T>(
    ctx: &Shared,
    message: impl Into<String>,
    hint: impl Into<String>,
) -> Result<T, PsError> {
    ctx.borrow_mut().failure = Some(("groupnesting", message.into()));
    fail(ctx, PsError::Rangecheck, hint)
}

/// Refuse to open a layer, or to work on the canvas's pixels, while a
/// transparency group is open: the program is painting on the group's
/// layer, and a canvas swap would exchange the page below it, so the
/// effect would be lost and the group would composite onto the wrong page.
/// (Whether one is open is `crate::groups`'s to say.)
pub(crate) fn refuse_in_group(i: &Interp, ctx: &Shared, who: &str, example: &str) -> OpResult {
    if groups_open(i) == 0 {
        return Ok(());
    }
    group_fail(
        ctx,
        format!("{who} cannot run inside begingroup ... endgroup"),
        format!(
            "psanim draws this on a layer of its own, and inside a transparency group the program paints on the group's layer, which psanim cannot reach: the effect would be lost. Write it with the group inside the effect, or closed before it: {example}. For opacity alone, `faded` works inside a group (it becomes a group itself)."
        ),
    )
}

/// Fail when a group opened inside `who`'s body is still open as it ends.
/// A psanim layer only ever opens with no group open, so any group open
/// now was opened inside it.
pub(crate) fn refuse_unclosed_inside(i: &Interp, ctx: &Shared, who: &str) -> OpResult {
    if groups_open(i) == 0 {
        return Ok(());
    }
    unclosed_inside(ctx, who)
}

fn unclosed_inside(ctx: &Shared, who: &str) -> OpResult {
    group_fail(
        ctx,
        format!("a transparency group opened inside {who} was not closed inside it"),
        "end each begingroup with endgroup inside the same body: a group left open escapes the effect and composites onto the page below it",
    )
}

/// `faded` or `blended` while a transparency group is open: run the body as
/// a group of its own, which composites where the program is painting
/// (the open group's layer). Leaves the group's dictionary and `true` for
/// the prelude's `.groupfaded`.
fn begin_as_group(i: &mut Interp, ctx: &Shared, end: &LayerEnd) -> OpResult {
    let mut entries = vec![("Alpha".to_string(), Value::Real(end.opacity))];
    match end.blend {
        Blend::Normal => {}
        Blend::Multiply => entries.push(("Blend".to_string(), Value::Name("Multiply".into()))),
        Blend::Add | Blend::Screen => {
            return group_fail(
                ctx,
                "`blended` with /add or /screen cannot run inside begingroup ... endgroup",
                "a transparency group blends only /normal and /multiply, and psanim's own blend modes need a layer of their own, which a group's marks never reach. Put the group inside it instead (`1 /add { << >> begingroup ... endgroup } blended`), or close the group first.",
            );
        }
    }
    i.push_value(&Value::Dict(entries))?;
    i.push_bool(true);
    ctx.borrow_mut().fades.push(FadeGroup {
        who: end.who,
        opened: None,
    });
    Ok(())
}

pub(crate) fn reg<F>(interp: &mut Interp, ctx: &Shared, name: &str, f: F)
where
    F: Fn(&mut Interp, &Shared) -> OpResult + 'static,
{
    let ctx = ctx.clone();
    let op: Rc<str> = name.into();
    interp
        .register_operator(name, move |i| {
            // A hint or failure describes the error this call raises, and
            // nothing later: one a `stopped` caught must not be read as the
            // diagnosis of the error that finally escapes.
            {
                let mut c = ctx.borrow_mut();
                if c.hint_op.is_some() || c.hint.is_some() || c.failure.is_some() {
                    c.hint = None;
                    c.failure = None;
                    c.hint_op = None;
                }
            }
            let r = f(i, &ctx);
            if let Err(e) = &r {
                let mut c = ctx.borrow_mut();
                if c.hint.is_some() || c.failure.is_some() {
                    // The command the interpreter will name for this error:
                    // the operator, or for `undefined` the missing name.
                    c.hint_op = Some(match e {
                        PsError::Undefined(n) => n.as_str().into(),
                        _ => op.clone(),
                    });
                }
            }
            r
        })
        .expect("psanim's operator names are valid and few");
}

/// An array of numbers as PostScript writes it, for `info`.
fn v_numbers(v: &Value) -> String {
    // Numbers, names and strings print as written; anything deeper as "...".
    fn one(v: &Value) -> Option<String> {
        match v {
            Value::Name(n) => Some(format!("/{n}")),
            Value::String(s) => Some(format!("({})", String::from_utf8_lossy(s))),
            v => v.as_f64().map(|x| x.to_string()),
        }
    }
    match v {
        Value::Array(a) if a.len() <= 12 => a
            .iter()
            .map(one)
            .collect::<Option<Vec<_>>>()
            .map_or_else(|| "[...]".to_string(), |p| format!("[{}]", p.join(" "))),
        _ => "[...]".to_string(),
    }
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

pub(crate) fn eval_keys(ctx: &Shared, items: &[Value], t: f64) -> Result<Value, PsError> {
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

/// A plane's dictionary as `onplane`, `projectpath` and `planepoint` take
/// it.
fn parse_plane(ctx: &Shared, v: &Value, who: &str) -> Result<Plane, PsError> {
    if !matches!(v, Value::Dict(_)) {
        return fail(
            ctx,
            PsError::Typecheck,
            format!(
                "{who} takes a plane dictionary, << /Tilt 60 >> for one; << >> is the flat plane"
            ),
        );
    }
    let view_w = {
        let c = ctx.borrow();
        c.views.last().map_or(c.width, |v| v.0)
    };
    let num = |key: &str, default: f64| -> Result<f64, PsError> {
        match v.get(key) {
            None => Ok(default),
            Some(x) => match x.as_f64() {
                Some(n) if n.is_finite() => Ok(n),
                _ => fail(
                    ctx,
                    PsError::Typecheck,
                    format!("{who}: /{key} must be a number"),
                ),
            },
        }
    };
    let pair = |key: &str| -> Result<Option<(f64, f64)>, PsError> {
        match v.get(key) {
            None => Ok(None),
            Some(x) => match x.as_numbers().as_deref() {
                Some([a, b]) if a.is_finite() && b.is_finite() => Ok(Some((*a, *b))),
                _ => fail(
                    ctx,
                    PsError::Typecheck,
                    format!("{who}: /{key} must be [a b], two numbers"),
                ),
            },
        }
    };
    let distance = num("Distance", view_w)?;
    if distance <= 0.0 {
        return fail(
            ctx,
            PsError::Rangecheck,
            format!("{who}: /Distance (how far the eye is from the screen) must be more than 0"),
        );
    }
    Ok(Plane {
        tilt: num("Tilt", 0.0)?,
        turn: num("Turn", 0.0)?,
        depth: num("Depth", 0.0)?,
        eye: pair("Eye")?.unwrap_or((0.0, 0.0)),
        distance,
        fade: pair("Fade")?,
    })
}

/// Device pixels per unit of a matrix's user space: the square root of its
/// determinant's size.
fn matrix_scale(m: &[f64; 6]) -> f64 {
    (m[0] * m[3] - m[1] * m[2]).abs().sqrt().max(1e-9)
}

/// Take the canvas out of the interpreter, run `f` on its pixels, put it
/// back.
/// Clear every pixel outside `r` (x0, y0, x1, y1): what a blur or glow
/// spread past a component's box.
fn clear_outside(px: &mut [u8], w: usize, h: usize, r: [usize; 4]) {
    if r == [0, 0, w, h] {
        return;
    }
    for y in 0..h {
        let row = &mut px[y * w * 4..(y + 1) * w * 4];
        if y < r[1] || y >= r[3] {
            row.fill(0);
        } else {
            row[..r[0].min(w) * 4].fill(0);
            row[r[2].min(w) * 4..].fill(0);
        }
    }
}

/// Put back the pixels outside `r` from `orig`: a blur or glow applied in
/// place to a canvas that also holds other drawing (blurlayer, glowlayer
/// inside a component) must leave that drawing as it was.
fn keep_outside(px: &mut [u8], orig: &[u8], w: usize, h: usize, r: [usize; 4]) {
    if r == [0, 0, w, h] {
        return;
    }
    for y in 0..h {
        let row = y * w * 4..(y + 1) * w * 4;
        if y < r[1] || y >= r[3] {
            px[row.clone()].copy_from_slice(&orig[row]);
        } else {
            let (a, b) = (y * w * 4, r[0].min(w) * 4);
            px[a..a + b].copy_from_slice(&orig[a..a + b]);
            let (c, d) = (y * w * 4 + r[2].min(w) * 4, (y + 1) * w * 4);
            px[c..d].copy_from_slice(&orig[c..d]);
        }
    }
}

/// The most points `alignpath` compares (its work grows as the square).
const ALIGN_MAX_POINTS: usize = 2048;

pub(crate) fn with_canvas_pixels(
    i: &mut Interp,
    ctx: &Shared,
    f: impl FnOnce(&mut [u8], usize, usize, &mut Vec<u8>),
) -> OpResult {
    refuse_in_group(
        i,
        ctx,
        "`blurlayer`, `glowlayer`, `vignette` or `filmgrain`",
        "`<< /Alpha 0.5 >> begingroup ... endgroup 4 blurlayer`",
    )?;
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

/// Open a layer for `who` (a phrase naming the word, for an error) and
/// `example` (how to write it with the group inside).
pub(crate) fn begin_layer(i: &mut Interp, ctx: &Shared, who: &str, example: &str) -> OpResult {
    refuse_in_group(i, ctx, who, example)?;
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
fn begin_pending(
    i: &mut Interp,
    ctx: &Shared,
    end: LayerEnd,
    who: &str,
    example: &str,
) -> OpResult {
    begin_layer(i, ctx, who, example)?;
    ctx.borrow_mut().pending.push(end);
    Ok(())
}

const PLAIN: LayerEnd = LayerEnd {
    opacity: 1.0,
    blend: Blend::Normal,
    blur: 0.0,
    glow: (0.0, 0.0),
    clip: None,
    who: "`faded`",
};

/// The clip `.clipmask` captured for the effect about to begin, or none.
fn take_clip(ctx: &Shared) -> Option<ClipMask> {
    ctx.borrow_mut().clip_taken.take().flatten()
}

/// `.maskbegin`, with the clip as the current path in device space (see
/// `.clipmask` in the prelude): the rectangle to paint through the clip,
/// on a fresh layer, and true; or false when the clip is empty and there
/// is nothing to paint.
fn mask_begin(i: &mut Interp, ctx: &Shared) -> OpResult {
    if groups_open(i) > 0 {
        // The effect asking for the mask refuses to run here, naming
        // itself; painting the mask would mark the group's layer.
        ctx.borrow_mut().clip_taken = Some(None);
        i.push_bool(false);
        return Ok(());
    }
    let (w, h) = i.canvas_size();
    let path = i.current_path()?;
    let mut b = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    let mut grow = |x: f64, y: f64| {
        b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
    };
    for s in &path {
        match *s {
            PathSegment::MoveTo(x, y) | PathSegment::LineTo(x, y) => grow(x, y),
            PathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => {
                grow(x1, y1);
                grow(x2, y2);
                grow(x3, y3);
            }
            PathSegment::Close => {}
        }
    }
    // The newest clip's box, a pixel wider all round, on the canvas: the
    // older clips it was intersected with lie inside it, and painting
    // through the clip itself gives the exact coverage.
    let x0 = (b[0].floor() - 1.0).max(0.0);
    let y0 = (b[1].floor() - 1.0).max(0.0);
    let x1 = (b[2].ceil() + 1.0).min(f64::from(w));
    let y1 = (b[3].ceil() + 1.0).min(f64::from(h));
    if !(x1 > x0 && y1 > y0) {
        // Everything is clipped away: the effect may show nothing.
        let mut c = ctx.borrow_mut();
        let mut alpha = c.mask_pool.pop().unwrap_or_default();
        alpha.clear();
        alpha.resize((w * h) as usize, 0);
        c.clip_taken = Some(Some(ClipMask {
            alpha,
            support: [0, 0, 0, 0],
        }));
        drop(c);
        i.push_bool(false);
        return Ok(());
    }
    begin_layer(
        i,
        ctx,
        "`blurred`, `glowing`, `blurlayer`, `glowlayer`, `vignette` or `filmgrain`",
        "`4 { << /Alpha 0.5 >> begingroup ... endgroup } blurred`",
    )?;
    i.push_numbers(&[x0, y0, x1 - x0, y1 - y0]);
    i.push_bool(true);
    Ok(())
}

/// `.maskend`: close the layer `.maskbegin` opened and keep its coverage
/// as the clip; none at all when it covers the whole canvas.
fn mask_end(i: &mut Interp, ctx: &Shared) -> OpResult {
    let Some(below) = ctx.borrow_mut().layers.pop() else {
        return fail(ctx, PsError::Rangecheck, ".maskend without .maskbegin");
    };
    let page = i.swap_canvas(below).map_err(|_| PsError::Rangecheck)?;
    let (w, h) = (page.width() as usize, page.height() as usize);
    let px = page.premultiplied_rgba8();
    let mut c = ctx.borrow_mut();
    let clip = if px.chunks_exact(4).all(|p| p[3] == 255) {
        None
    } else {
        let mut alpha = c.mask_pool.pop().unwrap_or_default();
        alpha.clear();
        alpha.extend(px.chunks_exact(4).map(|p| p[3]));
        let mut s = [w, h, 0, 0];
        for y in 0..h {
            let row = &alpha[y * w..(y + 1) * w];
            if let (Some(a), Some(b)) = (
                row.iter().position(|v| *v != 0),
                row.iter().rposition(|v| *v != 0),
            ) {
                s = [s[0].min(a), s[1].min(y), s[2].max(b + 1), s[3].max(y + 1)];
            }
        }
        if s[2] <= s[0] {
            s = [0, 0, 0, 0];
        }
        Some(ClipMask { alpha, support: s })
    };
    c.clip_taken = Some(clip);
    c.spare = Some(page);
    Ok(())
}

/// Confine an effect's spread to the clip it began under: zero what lies
/// outside the clip (a layer, whose own drawing was clipped already), or
/// with `orig`, put back what was there (the canvas, or a layer drawn on
/// before the clip was set).
fn apply_clip(px: &mut [u8], w: usize, h: usize, clip: &ClipMask, orig: Option<&[u8]>) {
    match orig {
        None => raster::mask_multiply(px, w, h, &clip.alpha, clip.support),
        Some(o) => raster::mask_mix(px, o, &clip.alpha),
    }
}

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

fn end_layer(i: &mut Interp, ctx: &Shared, opacity: f64, blend: Blend, who: &str) -> OpResult {
    if ctx.borrow().layers.is_empty() {
        return fail(
            ctx,
            PsError::Rangecheck,
            "endlayer without a matching beginlayer",
        );
    }
    refuse_unclosed_inside(i, ctx, who)?;
    let Some(mut below) = ctx.borrow_mut().layers.pop() else {
        return fail(ctx, PsError::Rangecheck, "a layer is missing");
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

/// `blurlayer` (no strength) or `glowlayer`: spread what is on the canvas,
/// within the component's box and the clip in force.
fn spread_layer(i: &mut Interp, ctx: &Shared, radius: f64, strength: Option<f64>) -> OpResult {
    let clip = take_clip(ctx);
    let sigma = radius * ctx.borrow().px_scale * ctx.borrow().unit_scale();
    let (cw, ch) = i.canvas_size();
    let rect = ctx.borrow().effect_rect(cw as usize, ch as usize);
    with_canvas_pixels(i, ctx, |px, w, h, scratch| {
        // These work in place on a canvas that may hold other drawing (the
        // movie's, around a component): what lies outside the component's
        // box or the clip keeps its earlier pixels.
        let orig = (clip.is_some() || rect != [0, 0, w, h]).then(|| px.to_vec());
        match strength {
            None => raster::blur(px, w, h, sigma, scratch),
            Some(k) => raster::glow(px, w, h, sigma, k.clamp(0.0, 2.0), scratch),
        }
        if let Some(o) = &orig {
            keep_outside(px, o, w, h, rect);
        }
        if let (Some(c), Some(o)) = (&clip, &orig) {
            apply_clip(px, w, h, c, Some(o));
        }
    })?;
    if let Some(c) = clip {
        ctx.borrow_mut().recycle_mask(c);
    }
    Ok(())
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    crate::vocab::install(interp, ctx);
    crate::catalog::install(interp, ctx);
    crate::terminal::install(interp, ctx);
    crate::particles::install(interp, ctx);
    crate::playlist::install(interp, ctx);
    crate::ca::install(interp, ctx);
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
            push_scope(ctx, TimeScope::window(s.t - a, b - a))?;
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
            push_scope(ctx, TimeScope::window(s.t - a, (s.dur - a).max(0.0)))?;
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
            push_scope(ctx, TimeScope::window(s.t, b))?;
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
            // A pure shift of the clock: the window shortens (so `progress`
            // and `span` are as they always were) but the period that
            // `turns` and `loopnoise` close over does not.
            TimeScope {
                t: s.t - d,
                dur: s.dur - d,
                cycle: s.cycle,
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
                cycle: s.cycle * k.abs(),
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
        push_scope(ctx, TimeScope::window(s.t.rem_euclid(p), p))?;
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
        push_scope(ctx, TimeScope::window(t, p))?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".attime", |i, ctx| {
        let body = pop_proc(i, ctx, "attime")?;
        let t = i.pop_f64()?;
        let s = ctx.borrow().scope();
        push_scope(
            ctx,
            TimeScope {
                t,
                dur: s.dur,
                cycle: s.cycle,
            },
        )?;
        i.push_value(&body)?;
        Ok(())
    });
    reg(interp, ctx, ".timewindow", |i, ctx| {
        let body = pop_proc(i, ctx, "timewindow")?;
        let (dur, t) = (i.pop_f64()?, i.pop_f64()?);
        push_scope(ctx, TimeScope::window(t, dur))?;
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
                cycle: s.cycle,
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
                push_scope(ctx, TimeScope::window(s.t - start, dur))?;
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
                    push_scope(ctx, TimeScope::window(t - clip.start, clip.dur))?;
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
                let total = plan.last().map_or(0.0, |c| c.start + c.dur);
                // Remember what was reported, so the load can check it once
                // the program has defined its clips and transitions. Only the
                // program's top level counts: a frame may measure freely.
                let mut c = ctx.borrow_mut();
                if c.running == Running::Setup && c.measured_reels.len() < 64 {
                    c.measured_reels.push((items.to_vec(), total));
                }
                drop(c);
                i.push_real(total);
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
        i.push_real(k * 360.0 * s.cycle_fraction());
        Ok(())
    });
    reg(interp, ctx, "loopnoise", |i, ctx| {
        let radius = i.pop_f64()?;
        let key = pop_key(i, ctx, "loopnoise: key radius")?;
        let c = ctx.borrow();
        let s = c.scope();
        let a = s.cycle_fraction() * std::f64::consts::TAU;
        i.push_real(crate::ctx::noise2(
            c.seed(),
            key,
            radius * a.cos(),
            radius * a.sin(),
        ));
        Ok(())
    });
    reg(interp, ctx, "sincelast", |i, ctx| {
        let Ok(times) = i.pop_numbers() else {
            return fail(
                ctx,
                PsError::Typecheck,
                "sincelast takes an array of times: [1.2 2.5 4] sincelast",
            );
        };
        let t = ctx.borrow().scope().t;
        // "Long ago" when none has passed, as `sinceevent` says it.
        let last = times
            .iter()
            .copied()
            .filter(|x| *x <= t)
            .fold(f64::NAN, f64::max);
        i.push_real(if last.is_nan() { 1.0e6 } else { t - last });
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

    reg(interp, ctx, "alignpath", |i, ctx| {
        let (b, a) = (i.pop_numbers()?, i.pop_numbers()?);
        if a.len() != b.len() || a.len() < 4 || a.len() % 2 != 0 {
            return fail(
                ctx,
                PsError::Rangecheck,
                "alignpath: both arrays must hold the same number of x y pairs (use the same count with resamplepath)",
            );
        }
        // Quadratic native work outside the interpreter's budgets: cap it.
        if a.len() / 2 > ALIGN_MAX_POINTS {
            return fail(
                ctx,
                PsError::Limitcheck,
                format!(
                    "alignpath: at most {ALIGN_MAX_POINTS} points per outline (resample to fewer)"
                ),
            );
        }
        i.push_numbers(&pathmath::align_outline(&a, &b));
        Ok(())
    });
    // --- perspective ------------------------------------------------------
    reg(interp, ctx, ".planebegin", |i, ctx| {
        let m = i.pop_numbers()?;
        let v = i.pop_value()?;
        let p = parse_plane(ctx, &v, "onplane")?;
        let Ok(m) = <[f64; 6]>::try_from(m.as_slice()) else {
            return Err(PsError::Rangecheck);
        };
        let mut c = ctx.borrow_mut();
        if c.planes.len() >= MAX_SCOPES {
            drop(c);
            return fail(
                ctx,
                PsError::Limitcheck,
                "onplane: too many planes open at once",
            );
        }
        c.planes.push((p, m));
        Ok(())
    });
    reg(interp, ctx, ".planeend", |_i, ctx| {
        ctx.borrow_mut().planes.pop();
        Ok(())
    });
    reg(interp, ctx, ".planedepth", |i, ctx| {
        i.push_int(ctx.borrow().planes.len() as i64);
        Ok(())
    });
    reg(interp, ctx, ".planectm", |i, ctx| {
        let m = ctx.borrow().planes.last().map(|p| p.1);
        i.push_numbers(&m.unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
        Ok(())
    });
    // The innermost plane, with the path in its coordinates (the prelude has
    // set its matrix): replace the path by its projection, every subpath
    // closed as fill sees it, and leave the opacity the fade gives it.
    reg(interp, ctx, ".planeproject", |i, ctx| {
        let Some((p, m)) = ctx.borrow().planes.last().copied() else {
            i.push_real(1.0);
            return Ok(());
        };
        let path = i.current_path()?;
        let out = plane::project_path(&p, &path, 1.0 / matrix_scale(&m), true);
        i.set_current_path(&out.path)?;
        i.push_real(out.opacity(&p));
        Ok(())
    });
    reg(interp, ctx, ".projectpath", |i, ctx| {
        let m = i.pop_numbers()?;
        let v = i.pop_value()?;
        let p = parse_plane(ctx, &v, "projectpath")?;
        let Ok(m) = <[f64; 6]>::try_from(m.as_slice()) else {
            return Err(PsError::Rangecheck);
        };
        let path = i.current_path()?;
        let out = plane::project_path(&p, &path, 1.0 / matrix_scale(&m), false);
        i.set_current_path(&out.path)
    });
    reg(interp, ctx, "planepoint", |i, ctx| {
        let v = i.pop_value()?;
        let y = i.pop_f64()?;
        let x = i.pop_f64()?;
        let p = parse_plane(ctx, &v, "planepoint")?;
        let Some((sx, sy)) = p.project(x, y) else {
            return fail(
                ctx,
                PsError::UndefinedResult,
                "planepoint: the point is behind the eye (or too close to it) and has no place on the screen",
            );
        };
        i.push_real(sx);
        i.push_real(sy);
        i.push_real(p.opacity(p.place(x, y)[2]));
        Ok(())
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
    reg(interp, ctx, "beginlayer", |i, ctx| {
        begin_layer(
            i,
            ctx,
            "`beginlayer`",
            "`beginlayer << /Alpha 0.5 >> begingroup ... endgroup 1 endlayer`",
        )
    });
    // The native halves of faded / blended / blurred / glowing: take the
    // operands now, so the body runs with the caller's stack on top. Each
    // leaves a boolean: true when the body is to run as a transparency
    // group (faded and blended inside one), with the group's dictionary
    // under it.
    reg(interp, ctx, ".fadebegin", |i, ctx| {
        let opacity = i.pop_f64()?;
        let end = LayerEnd { opacity, ..PLAIN };
        if groups_open(i) > 0 {
            return begin_as_group(i, ctx, &end);
        }
        begin_pending(i, ctx, end, "`faded`", "")?;
        i.push_bool(false);
        Ok(())
    });
    reg(interp, ctx, ".blendbegin", |i, ctx| {
        let name = i.pop_name()?;
        let opacity = i.pop_f64()?;
        let blend = blend_named(ctx, &name)?;
        let end = LayerEnd {
            opacity,
            blend,
            who: "`blended`",
            ..PLAIN
        };
        if groups_open(i) > 0 {
            return begin_as_group(i, ctx, &end);
        }
        begin_pending(i, ctx, end, "`blended`", "")?;
        i.push_bool(false);
        Ok(())
    });
    reg(interp, ctx, ".blurbegin", |i, ctx| {
        let blur = i.pop_f64()? * ctx.borrow().unit_scale();
        let clip = take_clip(ctx);
        begin_pending(
            i,
            ctx,
            LayerEnd {
                blur,
                clip,
                who: "`blurred`",
                ..PLAIN
            },
            "`blurred`",
            "`4 { << /Alpha 0.5 >> begingroup ... endgroup } blurred`",
        )?;
        i.push_bool(false);
        Ok(())
    });
    reg(interp, ctx, ".glowbegin", |i, ctx| {
        let (strength, radius) = (i.pop_f64()?, i.pop_f64()?);
        let radius = radius * ctx.borrow().unit_scale();
        let clip = take_clip(ctx);
        begin_pending(
            i,
            ctx,
            LayerEnd {
                glow: (radius, strength),
                clip,
                who: "`glowing`",
                ..PLAIN
            },
            "`glowing`",
            "`8 1 { << /Alpha 0.5 >> begingroup ... endgroup } glowing`",
        )?;
        i.push_bool(false);
        Ok(())
    });
    reg(interp, ctx, ".maskbegin", mask_begin);
    reg(interp, ctx, ".maskend", mask_end);
    reg(interp, ctx, ".layerend", |i, ctx| {
        let Some(end) = ctx.borrow_mut().pending.pop() else {
            return fail(
                ctx,
                PsError::Rangecheck,
                "a layer combinator ended without having begun",
            );
        };
        let who = end.who;
        refuse_unclosed_inside(i, ctx, who)?;
        let px = ctx.borrow().px_scale;
        let (cw, ch) = i.canvas_size();
        let rect = ctx.borrow().effect_rect(cw as usize, ch as usize);
        let spread = end.blur > 0.0 || end.glow.1 > 0.0;
        if end.blur > 0.0 {
            with_canvas_pixels(i, ctx, |p, w, h, scratch| {
                raster::blur(p, w, h, end.blur * px, scratch);
                clear_outside(p, w, h, rect);
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
                );
                clear_outside(p, w, h, rect);
            })?;
        }
        if let Some(clip) = end.clip {
            if spread {
                with_canvas_pixels(i, ctx, |p, w, h, _| apply_clip(p, w, h, &clip, None))?;
            }
            ctx.borrow_mut().recycle_mask(clip);
        }
        end_layer(i, ctx, end.opacity, end.blend, who)
    });
    reg(interp, ctx, "endlayer", |i, ctx| {
        let opacity = i.pop_f64()?;
        end_layer(
            i,
            ctx,
            opacity,
            Blend::Normal,
            "`beginlayer` ... `endlayer`",
        )
    });
    reg(interp, ctx, "endlayerblend", |i, ctx| {
        let name = i.pop_name()?;
        let opacity = i.pop_f64()?;
        let blend = blend_named(ctx, &name)?;
        end_layer(i, ctx, opacity, blend, "`beginlayer` ... `endlayerblend`")
    });
    // `.groupfaded` (the prelude) runs a `faded` or `blended` body as a
    // transparency group: these note the group it opened, and check at the
    // end that the body left that group the innermost one, neither closing
    // it (an `endgroup` in the body) nor leaving one of its own open.
    reg(interp, ctx, ".groupopened", |i, ctx| {
        let now = (groups_open(i), innermost_group(i));
        let mut c = ctx.borrow_mut();
        match c.fades.last_mut() {
            Some(f) if f.opened.is_none() => f.opened = Some(now),
            // `.groupfaded` called on its own (the tests do): a fade.
            _ => c.fades.push(FadeGroup {
                who: "`faded`",
                opened: Some(now),
            }),
        }
        Ok(())
    });
    reg(interp, ctx, ".groupbodydone", |i, ctx| {
        let Some(f) = ctx.borrow().fades.last().cloned() else {
            return fail(
                ctx,
                PsError::Rangecheck,
                ".groupbodydone outside .groupfaded",
            );
        };
        let (count, id) = f.opened.unwrap_or((0, None));
        let (now, now_id) = (groups_open(i), innermost_group(i));
        if now > count {
            return unclosed_inside(ctx, f.who);
        }
        if now < count || now_id != id {
            return group_fail(
                ctx,
                format!(
                    "the body of {} closed a transparency group it did not open",
                    f.who
                ),
                "an endgroup belongs in the same procedure as its begingroup: inside a group, `faded` runs its body as a group of its own, and an endgroup in the body closes that one instead",
            );
        }
        Ok(())
    });
    // The end of a frame's scene (or overlay), inside its run: a group
    // still open there is a misplaced `endgroup`. (PSCat would close it at
    // the end of the run, with its marks; the check must come before.)
    reg(interp, ctx, ".framedone", |i, ctx| {
        let open = groups_open(i);
        if open == 0 {
            return Ok(());
        }
        ctx.borrow_mut().failure = Some((
            "unclosedgroup",
            format!("{open} transparency group(s) opened with begingroup were never closed"),
        ));
        fail(
            ctx,
            PsError::Rangecheck,
            "every begingroup needs an endgroup in the same frame, in the same procedure",
        )
    });
    reg(interp, ctx, ".groupfadedone", |_, ctx| {
        ctx.borrow_mut().fades.pop();
        Ok(())
    });
    // `.groupfaded` found that the body ran `grestore` past the graphics
    // state it began with: a group's start is a floor it cannot pass.
    reg(interp, ctx, ".groupgrestored", |_, ctx| {
        let who = ctx.borrow().fades.last().map_or("`faded`", |f| f.who);
        group_fail(
            ctx,
            format!("the body of {who} ran grestore past the graphics state it began with"),
            "inside a transparency group, `faded` and `blended` run as a group of their own, and the start of a group is a floor `grestore` cannot pass. Pair every grestore in the body with a gsave in the body.",
        )
    });
    // The native halves of blurlayer and glowlayer, after `.clipmask`.
    reg(interp, ctx, ".blurlayer", |i, ctx| {
        refuse_in_group(
            i,
            ctx,
            "`blurlayer`",
            "`<< >> begingroup ... endgroup 4 blurlayer`",
        )?;
        let radius = i.pop_f64()?;
        spread_layer(i, ctx, radius, None)
    });
    reg(interp, ctx, ".glowlayer", |i, ctx| {
        refuse_in_group(
            i,
            ctx,
            "`glowlayer`",
            "`<< >> begingroup ... endgroup 8 1 glowlayer`",
        )?;
        let (strength, radius) = (i.pop_f64()?, i.pop_f64()?);
        spread_layer(i, ctx, radius, Some(strength))
    });
    // The native halves of vignette and filmgrain, after `.clipmask`: they
    // work within the component's box and hand back what the clip in force
    // (a rotated box's own outline) leaves outside it.
    reg(interp, ctx, ".vignette", |i, ctx| {
        refuse_in_group(
            i,
            ctx,
            "`vignette`",
            "`<< >> begingroup ... endgroup 0.5 vignette`",
        )?;
        let amount = i.pop_f64()?.clamp(0.0, 1.0);
        let clip = take_clip(ctx);
        let (cw, ch) = i.canvas_size();
        let r = ctx.borrow().effect_rect(cw as usize, ch as usize);
        with_canvas_pixels(i, ctx, |px, w, h, _| {
            let orig = clip.as_ref().map(|_| px.to_vec());
            // Centred on the component's box when inside one.
            let (hw, hh) = ((r[2] - r[0]) as f64 / 2.0, (r[3] - r[1]) as f64 / 2.0);
            let (cx, cy) = (r[0] as f64 + hw, r[1] as f64 + hh);
            let rmax2 = (hw * hw + hh * hh).max(1.0);
            for y in r[1]..r[3] {
                for x in r[0]..r[2] {
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
            if let (Some(c), Some(o)) = (&clip, &orig) {
                apply_clip(px, w, h, c, Some(o));
            }
        })?;
        if let Some(c) = clip {
            ctx.borrow_mut().recycle_mask(c);
        }
        Ok(())
    });
    reg(interp, ctx, ".filmgrain", |i, ctx| {
        refuse_in_group(
            i,
            ctx,
            "`filmgrain`",
            "`<< >> begingroup ... endgroup 0.2 filmgrain`",
        )?;
        let amount = i.pop_f64()?.clamp(0.0, 1.0);
        let clip = take_clip(ctx);
        let (seed, frame) = {
            let c = ctx.borrow();
            (c.seed(), c.frame)
        };
        let (cw, ch) = i.canvas_size();
        let r = ctx.borrow().effect_rect(cw as usize, ch as usize);
        with_canvas_pixels(i, ctx, |px, w, h, _| {
            let orig = clip.as_ref().map(|_| px.to_vec());
            // One cheap xorshift stream per frame: grain that moves.
            let mut s =
                crate::ctx::mix64(seed as u64 ^ (frame as u64).wrapping_mul(0x9E37_79B9)) | 1;
            let amp = amount * 64.0;
            let inside = |k: usize| {
                let (x, y) = (k % w, k / w);
                x >= r[0] && x < r[2] && y >= r[1] && y < r[3]
            };
            for (k, p) in px.chunks_exact_mut(4).enumerate() {
                if !inside(k) {
                    continue;
                }
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                let n = ((s >> 40) as f64 / (1u64 << 24) as f64 - 0.5) * amp;
                let a = p[3];
                for c in &mut p[..3] {
                    *c = ((f64::from(*c) + n).clamp(0.0, 255.0) as u8).min(a);
                }
            }
            if let (Some(c), Some(o)) = (&clip, &orig) {
                apply_clip(px, w, h, c, Some(o));
            }
        })?;
        if let Some(c) = clip {
            ctx.borrow_mut().recycle_mask(c);
        }
        Ok(())
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
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
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
    // t dur seed params unit rect .comppush -    open a component's clock:
    //                                  its movie time t of dur, its seed,
    //                                  the values its `param` lookups get
    //                                  (a dictionary, or null for none),
    //                                  the device length of its unit, and
    //                                  its box in device pixels [x0 y0 x1
    //                                  y1] (or null).
    reg(interp, ctx, ".comppush", |i, ctx| {
        let rect = match i.pop_value()? {
            Value::Null => None,
            v => v.as_numbers().filter(|n| n.len() == 4).map(|n| {
                let px = |x: f64| x.max(0.0).round() as usize;
                [px(n[0]), px(n[1]), px(n[2]), px(n[3])]
            }),
        };
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
            rect,
        });
        drop(c);
        push_scope(ctx, TimeScope::window(t, dur))
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
            *s = TimeScope::window(t, dur);
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
    // /name dt signalago -> v     the signal dt seconds before now.
    reg(interp, ctx, "signalago", |i, ctx| {
        let dt = i.pop_f64()?;
        let name = i.pop_name()?;
        let c = ctx.borrow();
        i.push_real(c.signals.value(&name, c.global_t - dt).unwrap_or(0.0));
        Ok(())
    });
    // /name span n signalseries -> [v ...]   n readings over the last span
    // seconds, oldest first, the last one now.
    reg(interp, ctx, "signalseries", |i, ctx| {
        let n = i.pop_int()?;
        let span = i.pop_f64()?;
        let name = i.pop_name()?;
        if !(2..=100_000).contains(&n) {
            return fail(
                ctx,
                PsError::Rangecheck,
                "signalseries wants 2 to 100000 readings: /cpu 3600 240 signalseries",
            );
        }
        let c = ctx.borrow();
        let t = c.global_t;
        let v = c
            .signals
            .series(&name, t - span, t, n as usize)
            .unwrap_or_else(|| vec![0.0; n as usize]);
        drop(c);
        i.push_numbers(&v);
        Ok(())
    });
    // /name t0 t1 n signalbetween -> [v ...]   n readings from signal time
    // t0 to t1: a fixed grid, for a window that does not slide.
    reg(interp, ctx, "signalbetween", |i, ctx| {
        let n = i.pop_int()?;
        let t1 = i.pop_f64()?;
        let t0 = i.pop_f64()?;
        let name = i.pop_name()?;
        if !(2..=100_000).contains(&n) {
            return fail(
                ctx,
                PsError::Rangecheck,
                "signalbetween wants 2 to 100000 readings: /cpu 0 60 240 signalbetween",
            );
        }
        let v = ctx
            .borrow()
            .signals
            .series(&name, t0, t1, n as usize)
            .unwrap_or_else(|| vec![0.0; n as usize]);
        i.push_numbers(&v);
        Ok(())
    });
    // The movie time signals are read at: the movie's own, inside a component too.
    reg(interp, ctx, "signaltime", |i, ctx| {
        i.push_real(ctx.borrow().global_t);
        Ok(())
    });
    reg(interp, ctx, "hassignal", |i, ctx| {
        let name = i.pop_name()?;
        let known = ctx.borrow().signals.tracks.contains_key(&name);
        i.push_bool(known);
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
/// `strict` is for playing, when everything is defined: every clip named
/// must be a procedure and a transition must really be in the dictionary.
/// Measuring a reel (`reelduration`, usually in the declaration, before the
/// clips are defined) is not strict, and must guess: a name starting `t-` is
/// taken for a transition even if it is not defined yet. Load checks the
/// guess afterwards (`check_measured_reels`).
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
        &|n| known.iter().any(|k| k == n) || (!strict && n.starts_with("t-")),
        &|n| !strict || matches!(i.lookup(n), Some(Value::Other(o)) if o.is_procedure()),
    )
}

/// Once the program's top level has run, compare each reel `reelduration`
/// measured with the reel as it now plays. The measurement had to guess
/// which names were transitions; a custom transition with another name than
/// `t-...`, defined after the measurement, was counted as a clip and made
/// the film longer. That is an error, naming the entry, rather than a film
/// that is quietly the wrong length.
pub(crate) fn check_measured_reels(i: &Interp, ctx: &Shared) -> Result<(), String> {
    let measured = std::mem::take(&mut ctx.borrow_mut().measured_reels);
    for (items, reported) in measured {
        // A reel that does not plan now is reported when it plays, with the
        // reason; only a reel that plans differently is the guess's fault.
        let Ok(plan) = plan_reel(i, &items, true) else {
            continue;
        };
        let now = plan.last().map_or(0.0, |c| c.start + c.dur);
        if (now - reported).abs() < 1e-9 {
            continue;
        }
        // The culprits: custom transitions that were not defined (and not
        // named `t-...`) when the reel was measured.
        let mut culprits: Vec<&str> = Vec::new();
        for item in &items {
            if let Some([_, Value::Name(n)]) = item.as_array()
                && !n.starts_with("t-")
                && !TRANSITIONS.contains(&n.as_str())
                && plan
                    .iter()
                    .all(|c| !matches!(c.body, Value::Name(b) if b == n))
                && !culprits.contains(&n.as_str())
            {
                culprits.push(n);
            }
        }
        let fix = |n: &str| {
            format!(
                "define the transition before `reelduration` is called (`transitions /{n} {{ ... }} put`, above the declaration), or name it t-{n}"
            )
        };
        return Err(match culprits.as_slice() {
            [] => format!(
                "reelduration measured a reel as {reported} s, but it plays for {now} s: the definitions it depends on changed after it was measured"
            ),
            [n, rest @ ..] => {
                let more = if rest.is_empty() {
                    String::new()
                } else {
                    format!(" (and {} more)", rest.len())
                };
                format!(
                    "reelduration measured a reel as {reported} s, but it plays for {now} s: /{n}{more} was not yet a transition when the reel was measured, so it was counted as a clip. To fix it, {}",
                    fix(n)
                )
            }
        });
    }
    Ok(())
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
