//! Typing and terminal sessions as pure functions of time.
//!
//! A *plan* is computed once, in setup, from a text or a session script and
//! a seed: a list of timed events (a key pressed, a key erased, a line
//! printed). `screenat` then answers what a character grid shows at any
//! time `t`, with no state between frames. The same function at an earlier
//! time is what makes phosphor persistence cheap: the ghost of the screen a
//! moment ago is just the screen at `t - lag`.
//!
//! The rhythm is a model, not a recording: each key's interval is a
//! log-normal jitter around a base speed, scaled by a per-word burst, with
//! longer gaps after a space, slower shifted and symbol keys, now and then
//! a thinking pause, and now and then a slip of the finger to a neighbouring
//! key that is noticed a character or two later, backspaced and retyped.

use pscat::host::Value;
use pscat::{Interp, PsError};

use crate::ctx::hash01;
use crate::ops::{Shared, fail, reg};

/// One thing that happens at a time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ev {
    /// A character appended to the line being typed.
    Char(u8),
    /// The last character of that line erased.
    Back,
    /// The line is finished; a new empty one begins.
    NewLine,
    /// A string appended to the current line (a prompt, a line of output).
    Text(usize),
    /// The last finished line replaced (a progress bar redrawn in place).
    Replace(usize),
    /// The screen cleared.
    Clear,
}

impl Ev {
    fn code(self) -> (f64, f64) {
        match self {
            Ev::Char(c) => (0.0, f64::from(c)),
            Ev::Back => (1.0, 0.0),
            Ev::NewLine => (2.0, 0.0),
            Ev::Text(i) => (3.0, i as f64),
            Ev::Replace(i) => (4.0, i as f64),
            Ev::Clear => (5.0, 0.0),
        }
    }

    fn from_code(kind: f64, arg: f64) -> Option<Ev> {
        Some(match kind as i64 {
            0 => Ev::Char(arg as u8),
            1 => Ev::Back,
            2 => Ev::NewLine,
            3 => Ev::Text(arg as usize),
            4 => Ev::Replace(arg as usize),
            5 => Ev::Clear,
            _ => return None,
        })
    }
}

/// A typed or scripted history: events in time order.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub events: Vec<(f64, Ev)>,
    pub texts: Vec<Vec<u8>>,
    pub marks: Vec<(String, f64)>,
    pub length: f64,
}

impl Plan {
    fn text(&mut self, s: &[u8]) -> usize {
        self.texts.push(s.to_vec());
        self.texts.len() - 1
    }

    /// The plan as the PostScript dictionary scripts carry around.
    pub fn to_value(&self) -> Value {
        let mut flat = Vec::with_capacity(self.events.len() * 3);
        for (t, ev) in &self.events {
            let (k, a) = ev.code();
            flat.extend([Value::Real(*t), Value::Real(k), Value::Real(a)]);
        }
        Value::Dict(vec![
            ("Events".into(), Value::Array(flat)),
            (
                "Texts".into(),
                Value::Array(
                    self.texts
                        .iter()
                        .map(|s| Value::String(s.clone()))
                        .collect(),
                ),
            ),
            (
                "Marks".into(),
                Value::Dict(
                    self.marks
                        .iter()
                        .map(|(n, t)| (n.clone(), Value::Real(*t)))
                        .collect(),
                ),
            ),
            ("Length".into(), Value::Real(self.length)),
        ])
    }

    /// The inverse of [`to_value`](Self::to_value); `None` if it is not a plan.
    pub fn from_value(v: &Value) -> Option<Plan> {
        let flat = v.get("Events")?.as_numbers()?;
        if flat.len() % 3 != 0 {
            return None;
        }
        let texts: Vec<Vec<u8>> = v
            .get("Texts")?
            .as_array()?
            .iter()
            .map(|s| match s {
                Value::String(b) => Some(b.clone()),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let mut events = Vec::with_capacity(flat.len() / 3);
        for c in flat.chunks_exact(3) {
            let ev = Ev::from_code(c[1], c[2])?;
            if let Ev::Text(i) | Ev::Replace(i) = ev
                && i >= texts.len()
            {
                return None;
            }
            events.push((c[0], ev));
        }
        let marks = match v.get("Marks")? {
            Value::Dict(m) => m
                .iter()
                .map(|(k, v)| v.as_f64().map(|t| (k.clone(), t)))
                .collect::<Option<_>>()?,
            _ => return None,
        };
        Some(Plan {
            events,
            texts,
            marks,
            length: v.get("Length")?.as_f64()?,
        })
    }

    /// When the first mark of that name was made.
    pub fn mark(&self, name: &str) -> Option<f64> {
        self.marks.iter().find(|m| m.0 == name).map(|m| m.1)
    }
}

// --- typing -------------------------------------------------------------------

/// How a text is typed.
#[derive(Clone, Debug)]
pub struct TypeOpts {
    /// Keys-per-second multiplier: 1 is a brisk typist (about nine keys a second).
    pub speed: f64,
    /// Chance of a slip at each letter.
    pub typos: f64,
    /// Positions (characters of the text) that always slip.
    pub slips: Vec<usize>,
    /// Multiplier on word pauses and thinking pauses.
    pub pauses: f64,
}

impl Default for TypeOpts {
    fn default() -> Self {
        TypeOpts {
            speed: 1.0,
            typos: 0.02,
            slips: Vec::new(),
            pauses: 1.0,
        }
    }
}

/// Sequential draws from a seed: the n-th call is `hash01(seed, [n])`, so a
/// plan depends on nothing but its inputs.
struct Rng {
    seed: i64,
    n: f64,
}

impl Rng {
    fn new(seed: i64, salt: f64) -> Rng {
        Rng {
            seed: seed.wrapping_mul(2_654_435_761).wrapping_add(salt as i64),
            n: 0.0,
        }
    }
    fn draw(&mut self) -> f64 {
        self.n += 1.0;
        hash01(self.seed, &[self.n])
    }
    /// Roughly normal, mean 0, sd 1.
    fn gauss(&mut self) -> f64 {
        ((self.draw() + self.draw() + self.draw() + self.draw()) - 2.0) * 1.732
    }
}

const KEY_ROWS: [&[u8]; 3] = [b"qwertyuiop", b"asdfghjkl", b"zxcvbnm"];

/// A key next to this one on a QWERTY keyboard, or `None` for a key
/// (space, punctuation) that is not slipped on.
fn neighbour(c: u8, rng: &mut Rng) -> Option<u8> {
    let lower = c.to_ascii_lowercase();
    let mut found: Vec<u8> = Vec::new();
    if lower.is_ascii_digit() {
        if lower > b'1' {
            found.push(lower - 1);
        }
        if lower < b'9' {
            found.push(lower + 1);
        }
    } else {
        for (r, row) in KEY_ROWS.iter().enumerate() {
            if let Some(col) = row.iter().position(|k| *k == lower) {
                let mut add = |r2: isize, c2: isize| {
                    if r2 >= 0
                        && (r2 as usize) < KEY_ROWS.len()
                        && c2 >= 0
                        && (c2 as usize) < KEY_ROWS[r2 as usize].len()
                    {
                        found.push(KEY_ROWS[r2 as usize][c2 as usize]);
                    }
                };
                let (r, col) = (r as isize, col as isize);
                add(r, col - 1);
                add(r, col + 1);
                add(r - 1, col);
                add(r - 1, col + 1);
                add(r + 1, col - 1);
                add(r + 1, col);
            }
        }
    }
    if found.is_empty() {
        return None;
    }
    let k = found[(rng.draw() * found.len() as f64) as usize % found.len()];
    Some(if c.is_ascii_uppercase() {
        k.to_ascii_uppercase()
    } else {
        k
    })
}

const BASE: f64 = 0.095;

/// The keystrokes that type `text`: (seconds after the first key, key),
/// where a key is `Ev::Char` or `Ev::Back`.
pub fn typing_keys(text: &[u8], seed: i64, o: &TypeOpts) -> Vec<(f64, Ev)> {
    let speed = o.speed.max(0.05);
    let mut rng = Rng::new(seed, 11.0);
    let mut out = Vec::with_capacity(text.len() + 8);
    let mut t = 0.0;
    let mut burst = 0.55 + 0.8 * rng.draw();
    let mut hot = 0.0f64; // a run of words typed quickly
    let mut slips_left = 2;
    let mut i = 0;
    while i < text.len() {
        let c = text[i];
        let prev = if i > 0 { text[i - 1] } else { b' ' };
        if i > 0 {
            let mut f = burst * (0.32 * rng.gauss()).exp();
            if hot > 0.0 {
                f *= 0.8;
            }
            if c.is_ascii_uppercase() || !(c.is_ascii_alphanumeric() || c == b' ') {
                f *= 1.25;
            }
            let mut dt = BASE / speed * f;
            if prev == b' ' {
                dt += BASE / speed * (0.5 + rng.draw()) * o.pauses;
                if rng.draw() < 0.1 {
                    dt += (0.35 + 0.7 * rng.draw()) * o.pauses / speed.sqrt();
                }
            }
            t += dt.max(0.03);
        }
        let forced = o.slips.contains(&i);
        let slip = c.is_ascii_alphabetic()
            && i > 0
            && (forced || (slips_left > 0 && rng.draw() < o.typos));
        if slip && let Some(wrong) = neighbour(c, &mut rng) {
            if !forced {
                slips_left -= 1;
            }
            out.push((t, Ev::Char(wrong)));
            // The finger goes on for a character or two before the eye notices.
            let r = rng.draw();
            let extra = (if r < 0.5 {
                0
            } else if r < 0.8 {
                1
            } else {
                2
            })
            .min(text.len() - i - 1);
            for j in 1..=extra {
                t += (BASE / speed * (0.7 + 0.6 * rng.draw())).max(0.03);
                out.push((t, Ev::Char(text[i + j])));
            }
            t += (0.2 + 0.3 * rng.draw()) / speed.sqrt(); // noticing it
            for _ in 0..=extra {
                t += 0.055 + 0.04 * rng.draw();
                out.push((t, Ev::Back));
            }
            t += 0.08 + 0.1 * rng.draw();
            out.push((t, Ev::Char(c)));
        } else {
            out.push((t, Ev::Char(c)));
        }
        if c == b' ' {
            burst = 0.55 + 0.8 * rng.draw();
            hot = (hot - 1.0).max(0.0);
            if hot == 0.0 && rng.draw() < 0.2 {
                hot = 3.0;
            }
        }
        i += 1;
    }
    out
}

// --- sessions -----------------------------------------------------------------

/// Defaults a session script's items can override.
#[derive(Clone, Debug)]
struct SessionOpts {
    prompt: Vec<u8>,
    typing: TypeOpts,
    think: f64,
    latency: f64,
    gap: f64,
    start: f64,
    final_prompt: bool,
    tail: f64,
}

fn num(v: &Value, key: &str, default: f64) -> Result<f64, String> {
    match v.get(key) {
        None => Ok(default),
        Some(x) => x.as_f64().ok_or_else(|| format!("/{key} must be a number")),
    }
}

fn type_opts(v: &Value, base: &TypeOpts) -> Result<TypeOpts, String> {
    let mut o = base.clone();
    o.speed = num(v, "Speed", o.speed)?;
    o.typos = num(v, "Typos", o.typos)?;
    o.pauses = num(v, "Pauses", o.pauses)?;
    if let Some(s) = v.get("Slips") {
        o.slips = s
            .as_numbers()
            .ok_or("/Slips must be an array of character positions")?
            .into_iter()
            .map(|x| x.max(0.0) as usize)
            .collect();
    }
    Ok(o)
}

fn session_opts(v: &Value) -> Result<SessionOpts, String> {
    let prompt = match v.get("Prompt") {
        None => b"$ ".to_vec(),
        Some(Value::String(s)) => s.clone(),
        Some(_) => return Err("/Prompt must be a string".into()),
    };
    let typing = type_opts(v, &TypeOpts::default())?;
    Ok(SessionOpts {
        prompt,
        typing,
        think: num(v, "Think", 0.6)?,
        latency: num(v, "Latency", 0.18)?,
        gap: num(v, "Gap", 0.07)?,
        start: num(v, "Start", 0.0)?,
        final_prompt: v.get("FinalPrompt").map_or(Ok(true), |x| {
            x.as_bool().ok_or("/FinalPrompt must be a bool")
        })?,
        tail: num(v, "Tail", 0.0)?,
    })
}

fn lines_of(s: &[u8]) -> Vec<&[u8]> {
    let mut v: Vec<&[u8]> = s.split(|b| *b == b'\n').collect();
    if v.len() > 1 && v.last().is_some_and(|l| l.is_empty()) {
        v.pop();
    }
    v
}

/// Compile a session script: `[ [/type (ls)] [/out (a\nb)] [/wait 1] ... ]`.
fn compile_session(items: &[Value], seed: i64, o: &SessionOpts) -> Result<Plan, String> {
    let mut plan = Plan::default();
    let mut prompt = o.prompt.clone();
    let mut t = o.start;
    let mut line_empty = true;
    let mut after_type = false;
    for (n, item) in items.iter().enumerate() {
        let parts = item
            .as_array()
            .ok_or_else(|| format!("item {n} must be an array like [/type (ls)]"))?;
        let kind = match parts.first() {
            Some(Value::Name(k)) => k.as_str(),
            _ => {
                return Err(format!(
                    "item {n} must start with a name: /type /out /lines /progress /wait /clear /mark /prompt"
                ));
            }
        };
        let arg = parts.get(1);
        let extra = parts.get(2);
        let text = |what: &str| -> Result<Vec<u8>, String> {
            match arg {
                Some(Value::String(s)) => Ok(s.clone()),
                _ => Err(format!("item {n} (/{kind}) needs {what} as a string")),
            }
        };
        let iseed = seed.wrapping_mul(1_000_003).wrapping_add(n as i64);
        match kind {
            "type" => {
                let cmd = text("the command")?;
                let opts = extra.cloned().unwrap_or(Value::Dict(vec![]));
                let topts = type_opts(&opts, &o.typing)?;
                let think = num(&opts, "Think", o.think)?;
                if line_empty {
                    let i = plan.text(&prompt);
                    plan.events.push((t, Ev::Text(i)));
                }
                t += think * (0.7 + 0.6 * hash01(iseed, &[1.0]));
                let keys = typing_keys(&cmd, iseed, &topts);
                let end = keys.last().map_or(0.0, |k| k.0);
                for (kt, ev) in keys {
                    plan.events.push((t + kt, ev));
                }
                t += end + 0.12 + 0.25 * hash01(iseed, &[2.0]);
                plan.events.push((t, Ev::NewLine));
                line_empty = true;
                after_type = true;
            }
            "out" | "lines" | "progress" => {
                let body = text("the output")?;
                if after_type {
                    t += o.latency;
                    after_type = false;
                }
                let gap = match extra {
                    Some(v) if v.as_f64().is_some() => v.as_f64().unwrap(),
                    None => {
                        if kind == "progress" {
                            0.12
                        } else {
                            o.gap
                        }
                    }
                    _ => {
                        return Err(format!(
                            "item {n} (/{kind}): the second argument is a gap in seconds"
                        ));
                    }
                };
                let mut rng = Rng::new(iseed, 5.0);
                for (k, line) in lines_of(&body).into_iter().enumerate() {
                    if kind != "out" && k > 0 {
                        let mut d = gap * (0.5 + rng.draw());
                        if kind == "lines" && rng.draw() < 0.08 {
                            d += gap * 4.0;
                        }
                        t += d;
                    }
                    let i = plan.text(line);
                    if kind == "progress" && k > 0 {
                        plan.events.push((t, Ev::Replace(i)));
                    } else {
                        if !line.is_empty() {
                            plan.events.push((t, Ev::Text(i)));
                        }
                        plan.events.push((t, Ev::NewLine));
                    }
                }
                line_empty = true;
            }
            "wait" => {
                t += arg
                    .and_then(Value::as_f64)
                    .ok_or_else(|| format!("item {n} (/wait) needs seconds"))?
                    .max(0.0);
            }
            "clear" => {
                plan.events.push((t, Ev::Clear));
                line_empty = true;
            }
            "mark" => match arg {
                Some(Value::Name(m)) => plan.marks.push((m.clone(), t)),
                _ => return Err(format!("item {n} (/mark) needs a name: [/mark /built]")),
            },
            "prompt" => prompt = text("the prompt")?,
            other => {
                return Err(format!(
                    "item {n}: unknown kind /{other}. Kinds: /type /out /lines /progress /wait /clear /mark /prompt"
                ));
            }
        }
    }
    if o.final_prompt && line_empty {
        let i = plan.text(&prompt);
        plan.events.push((t, Ev::Text(i)));
    }
    plan.length = t + o.tail;
    // Events are made in time order, except that a typed line's keys may
    // overlap nothing; sort defensively (stable).
    plan.events
        .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    Ok(plan)
}

/// The plan for typing one string on an empty line.
pub fn typing_plan(text: &[u8], seed: i64, o: &TypeOpts, think: f64) -> Plan {
    let mut plan = Plan::default();
    for (kt, ev) in typing_keys(text, seed, o) {
        plan.events.push((think + kt, ev));
    }
    plan.length = plan.events.last().map_or(think, |e| e.0);
    plan
}

// --- the screen -----------------------------------------------------------------

/// The state of the screen at `t`: its rows (the last `rows` of them, each
/// at most `cols` wide), the cursor's column in the last, and when the last
/// event up to `t` happened (-1 if none).
pub fn screen_at(plan: &Plan, t: f64, rows: usize, cols: usize) -> (Vec<Vec<u8>>, usize, f64) {
    let (lines, last) = lines_at(plan, t);
    let cols = cols.max(1);
    let mut phys: Vec<Vec<u8>> = Vec::new();
    for l in &lines {
        if l.is_empty() {
            phys.push(Vec::new());
        } else {
            for c in l.chunks(cols) {
                phys.push(c.to_vec());
            }
        }
    }
    // A cursor at the end of a full row starts the next.
    if lines
        .last()
        .is_some_and(|l| !l.is_empty() && l.len() % cols == 0)
    {
        phys.push(Vec::new());
    }
    let col = phys.last().map_or(0, Vec::len);
    let skip = phys.len().saturating_sub(rows.max(1));
    (phys.split_off(skip), col, last)
}

/// The logical lines at `t`, the last being the one under the cursor.
fn lines_at(plan: &Plan, t: f64) -> (Vec<Vec<u8>>, f64) {
    let mut lines: Vec<Vec<u8>> = vec![Vec::new()];
    let mut last = -1.0;
    for (et, ev) in &plan.events {
        if *et > t + 1e-9 {
            break;
        }
        last = *et;
        match *ev {
            Ev::Char(c) => lines.last_mut().expect("a line").push(c),
            Ev::Back => {
                lines.last_mut().expect("a line").pop();
            }
            Ev::NewLine => lines.push(Vec::new()),
            Ev::Text(i) => lines
                .last_mut()
                .expect("a line")
                .extend_from_slice(&plan.texts[i]),
            Ev::Replace(i) => {
                let n = lines.len();
                if n >= 2 {
                    lines[n - 2] = plan.texts[i].clone();
                }
            }
            Ev::Clear => lines = vec![Vec::new()],
        }
    }
    (lines, last)
}

// --- operators ------------------------------------------------------------------

fn pop_plan(i: &mut Interp, ctx: &Shared, who: &str) -> Result<Plan, PsError> {
    let v = i.pop_value()?;
    Plan::from_value(&v).ok_or_else(|| {
        ctx.borrow_mut().hint = Some(format!(
            "{who} takes a plan made by typingplan or sessionplan, not {}",
            match &v {
                Value::Dict(_) => "another dictionary",
                _ => "this operand",
            }
        ));
        PsError::Typecheck
    })
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    // (text) seed opts  typingplan  plan
    reg(interp, ctx, "typingplan", |i, ctx| {
        let opts = i.pop_value()?;
        let seed = i.pop_f64()? as i64;
        let text = i.pop_string()?;
        let (o, think) = type_opts(&opts, &TypeOpts::default())
            .and_then(|o| num(&opts, "Think", 0.0).map(|t| (o, t)))
            .or_else(|e| fail(ctx, PsError::Typecheck, format!("typingplan: {e}")))?;
        i.push_value(&typing_plan(&text, seed, &o, think).to_value())
    });
    // [ items ] seed opts  sessionplan  plan
    reg(interp, ctx, "sessionplan", |i, ctx| {
        let opts = i.pop_value()?;
        let seed = i.pop_f64()? as i64;
        let items = i.pop_value()?;
        let run = || -> Result<Plan, String> {
            let o = session_opts(&opts)?;
            let items = items
                .as_array()
                .ok_or("the session must be an array of items")?;
            compile_session(items, seed, &o)
        };
        match run() {
            Ok(p) => i.push_value(&p.to_value()),
            Err(e) => fail(ctx, PsError::Rangecheck, format!("sessionplan: {e}")),
        }
    });
    // plan  planlength  seconds
    reg(interp, ctx, "planlength", |i, ctx| {
        let p = pop_plan(i, ctx, "planlength")?;
        i.push_real(p.length);
        Ok(())
    });
    // plan /name  planmark  seconds     (-1 if the plan has no such mark)
    reg(interp, ctx, "planmark", |i, ctx| {
        let name = i.pop_name()?;
        let p = pop_plan(i, ctx, "planmark")?;
        i.push_real(p.mark(&name).unwrap_or(-1.0));
        Ok(())
    });
    // plan t  typedat  (string)     the line under the cursor at time t
    reg(interp, ctx, "typedat", |i, ctx| {
        let t = i.pop_f64()?;
        let p = pop_plan(i, ctx, "typedat")?;
        let (mut lines, _) = lines_at(&p, t);
        i.push_string(&lines.pop().unwrap_or_default());
        Ok(())
    });
    // plan t rows cols  screenat  [ (row) ... ] col last
    reg(interp, ctx, "screenat", |i, ctx| {
        let cols = i.pop_int()?.clamp(1, 1000) as usize;
        let rows = i.pop_int()?.clamp(1, 1000) as usize;
        let t = i.pop_f64()?;
        let p = pop_plan(i, ctx, "screenat")?;
        let (r, col, last) = screen_at(&p, t, rows, cols);
        i.push_value(&Value::Array(r.into_iter().map(Value::String).collect()))?;
        i.push_int(col as i64);
        i.push_real(last);
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(plan: &Plan, t: f64) -> String {
        String::from_utf8(lines_at(plan, t).0.pop().unwrap()).unwrap()
    }

    #[test]
    fn typing_is_deterministic_and_ends_with_the_text() {
        let o = TypeOpts {
            typos: 0.2,
            ..TypeOpts::default()
        };
        let a = typing_plan(b"cargo build --release", 3, &o, 0.0);
        let b = typing_plan(b"cargo build --release", 3, &o, 0.0);
        assert_eq!(a.events, b.events);
        assert_eq!(typed(&a, a.length + 1.0), "cargo build --release");
        let c = typing_plan(b"cargo build --release", 4, &o, 0.0);
        assert_ne!(a.events, c.events, "another seed types another way");
    }

    #[test]
    fn keys_come_in_order_at_a_human_pace() {
        let text = b"the quick brown fox jumps over the lazy dog";
        let steady = TypeOpts {
            typos: 0.0,
            ..TypeOpts::default()
        };
        let p = typing_plan(text, 1, &steady, 0.0);
        assert!(p.events.windows(2).all(|w| w[0].0 <= w[1].0));
        let cps = text.len() as f64 / p.length;
        assert!((4.0..16.0).contains(&cps), "{cps} keys per second");
        let fast = typing_plan(
            text,
            1,
            &TypeOpts {
                speed: 2.0,
                ..steady.clone()
            },
            0.0,
        );
        assert!(
            fast.length < p.length * 0.8,
            "{} vs {}",
            fast.length,
            p.length
        );
        // Intervals vary: it is not a metronome.
        let gaps: Vec<f64> = p.events.windows(2).map(|w| w[1].0 - w[0].0).collect();
        let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
        let var = gaps.iter().map(|g| (g - mean).powi(2)).sum::<f64>() / gaps.len() as f64;
        assert!(var.sqrt() > mean * 0.3, "sd {} mean {mean}", var.sqrt());
        // A word gap is longer than the gaps inside words, on average.
        let (mut sp, mut ns, mut nsp, mut nn) = (0.0, 0.0, 0, 0);
        for (k, g) in gaps.iter().enumerate() {
            if text[k] == b' ' {
                sp += g;
                nsp += 1;
            } else {
                ns += g;
                nn += 1;
            }
        }
        assert!(sp / f64::from(nsp) > 1.3 * ns / f64::from(nn));
    }

    #[test]
    fn a_slip_is_typed_wrong_erased_and_fixed() {
        let o = TypeOpts {
            typos: 0.0,
            slips: vec![6],
            ..TypeOpts::default()
        };
        let p = typing_plan(b"make test", 9, &o, 0.0);
        let backs = p.events.iter().filter(|e| e.1 == Ev::Back).count();
        assert!(backs >= 1);
        // Somewhere along the way the line was not a prefix of the text.
        let mut off = false;
        for (t, _) in &p.events {
            let s = typed(&p, *t);
            if !b"make test".starts_with(s.as_bytes()) {
                off = true;
            }
        }
        assert!(off, "no wrong text was ever on screen");
        assert_eq!(typed(&p, p.length + 1.0), "make test");
        // No typos when asked for none, however long.
        let clean = typing_plan(
            b"a long command with many letters in it",
            2,
            &TypeOpts {
                typos: 0.0,
                ..TypeOpts::default()
            },
            0.0,
        );
        assert!(clean.events.iter().all(|e| matches!(e.1, Ev::Char(_))));
    }

    fn session(items: &[Value]) -> Plan {
        compile_session(items, 1, &session_opts(&Value::Dict(vec![])).unwrap()).unwrap()
    }
    fn item(kind: &str, s: &str) -> Value {
        Value::Array(vec![
            Value::Name(kind.into()),
            Value::String(s.as_bytes().to_vec()),
        ])
    }
    fn rows_of(p: &Plan, t: f64, rows: usize, cols: usize) -> Vec<String> {
        screen_at(p, t, rows, cols)
            .0
            .into_iter()
            .map(|r| String::from_utf8(r).unwrap())
            .collect()
    }

    #[test]
    fn a_session_prints_prompt_types_and_answers() {
        let p = session(&[
            item("type", "ls"),
            item("out", "a.txt\nb.txt"),
            Value::Array(vec![Value::Name("mark".into()), Value::Name("done".into())]),
        ]);
        assert_eq!(rows_of(&p, 0.0, 10, 40), vec!["$ "]);
        let end = rows_of(&p, p.length + 1.0, 10, 40);
        assert_eq!(end, vec!["$ ls", "a.txt", "b.txt", "$ "]);
        // The prompt comes back at the end, and the mark is where the output ended.
        let m = p.mark("done").unwrap();
        assert!(m > 0.0 && m <= p.length);
        // Output is not there before the command is finished.
        let mid = rows_of(&p, m - 0.05, 10, 40);
        assert_eq!(mid, vec!["$ ls", ""]);
        assert_eq!(rows_of(&p, m, 10, 40).len(), 4);
    }

    #[test]
    fn output_scrolls_when_the_screen_is_full_and_long_lines_wrap() {
        let body: String = (1..=12).map(|n| format!("line {n}\n")).collect();
        let p = session(&[item("out", &body)]);
        let rows = rows_of(&p, p.length + 1.0, 5, 40);
        assert_eq!(rows, vec!["line 9", "line 10", "line 11", "line 12", "$ "]);
        let p = session(&[item("out", "abcdefghij")]);
        let rows = rows_of(&p, p.length + 1.0, 5, 4);
        assert_eq!(rows, vec!["abcd", "efgh", "ij", "$ "]);
    }

    #[test]
    fn lines_arrive_one_after_another_and_progress_redraws_in_place() {
        let p = session(&[item("lines", "one\ntwo\nthree")]);
        let times: Vec<f64> = p
            .events
            .iter()
            .filter(|e| e.1 == Ev::NewLine)
            .map(|e| e.0)
            .collect();
        assert!(times.windows(2).all(|w| w[1] > w[0]));
        let p = session(&[item("progress", "[#   ]\n[##  ]\n[### ]")]);
        assert_eq!(rows_of(&p, p.length + 1.0, 5, 40), vec!["[### ]", "$ "]);
        let first = p.events[0].0;
        assert_eq!(rows_of(&p, first, 5, 40)[0], "[#   ]");
    }

    #[test]
    fn clear_empties_the_screen_and_mistakes_are_explained() {
        let p = session(&[
            item("out", "hello"),
            Value::Array(vec![Value::Name("clear".into())]),
        ]);
        assert_eq!(rows_of(&p, p.length + 1.0, 5, 40), vec!["$ "]);
        let bad = compile_session(
            &[Value::Array(vec![Value::Name("nope".into())])],
            1,
            &session_opts(&Value::Dict(vec![])).unwrap(),
        );
        assert!(bad.unwrap_err().contains("unknown kind /nope"));
    }

    #[test]
    fn a_plan_survives_the_round_trip_through_postscript_values() {
        let p = session(&[item("type", "echo hi"), item("out", "hi")]);
        let q = Plan::from_value(&p.to_value()).unwrap();
        assert_eq!(p.events, q.events);
        assert_eq!(p.texts, q.texts);
        assert_eq!(p.length, q.length);
    }
}
