//! Particles: emitters whose every particle is a function of time.
//!
//! A particle is born at a time fixed by its emitter (a burst, or a slot of
//! continuous emission) with a position, velocity and life drawn from
//! `hash01` of its key. Its state at time `t` is computed from those alone:
//! in closed form for gravity, linear drag and a constant wind, and by
//! stepping through a fixed grid of cells anchored at time 0 when a gusting
//! wind varies over time. Nothing is remembered between frames, so any
//! frame renders on its own, in any order, and on any core.
//!
//! A native operator cannot call a procedure, so `particles` is split as
//! the time scopes are: `.particlesbegin` computes the live particles and
//! keeps them in [`Ctx::particles`], the prelude's loop calls the author's
//! procedure once per particle, and the `particle...` words read the
//! current one, the way `now` reads the clock.

use pscat::host::{PathSegment, Value};
use pscat::{Interp, PsError};

use crate::ctx::{hash01, noise2, wiggle};
use crate::ops::{Shared, eval_keys, fail, reg};

/// The most gust cells one emitter may tabulate for a frame (about 4.8
/// hours of particle life at 60 cells a second).
const MAX_GUST_CELLS: usize = 1 << 20;

/// The most gust cells all particles and trails may integrate in one
/// moment (about a second of native work).
const MAX_GUST_WORK: f64 = 40_000_000.0;

/// Charge `cells` of gust integration to this moment, failing when the
/// total would pass [`MAX_GUST_WORK`].
fn charge_gust(ctx: &Shared, cells: f64) -> Result<(), PsError> {
    let mut c = ctx.borrow_mut();
    c.gust_work += cells.max(0.0);
    if c.gust_work > MAX_GUST_WORK {
        c.hint = Some(format!(
            "particles: following the wind gusts would take {:.0} million steps this frame (at most {:.0} million): fewer particles, a shorter /Life, or no /Gust",
            c.gust_work / 1e6,
            MAX_GUST_WORK / 1e6
        ));
        return Err(PsError::Limitcheck);
    }
    Ok(())
}

/// How many particles one `particles` call may draw.
pub(crate) const MAX_PARTICLES: usize = 200_000;
/// The cell of the grid a gusting wind is held constant over, in seconds.
const GUST_CELL: f64 = 1.0 / 60.0;

/// Where an emitter is: a point, or keyframes of points.
#[derive(Clone, Debug)]
enum Place {
    Point(f64, f64),
    Keys(Vec<Value>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Shape {
    Random,
    Even,
    Sphere,
}

/// One emitter, as parsed from its dictionary.
#[derive(Clone, Debug)]
pub(crate) struct Emitter {
    at: Place,
    area: (f64, f64),
    rate: f64,
    from: f64,
    to: f64,
    bursts: Vec<f64>,
    count: usize,
    life: (f64, f64),
    speed: (f64, f64),
    angle: f64,
    spread: f64,
    shape: Shape,
    gravity: (f64, f64),
    drag: f64,
    wind: (f64, f64),
    gust: (f64, f64),
    gust_freq: f64,
    sway: (f64, f64),
    sway_freq: f64,
    inherit: f64,
    period: f64,
    salt: f64,
    /// The gust in each grid cell from `gust_j0` on, worked out once per
    /// frame for the cells a live particle can have flown through.
    gust_cells: Vec<(f64, f64)>,
    gust_j0: f64,
}

/// One live particle at the moment being drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub age: f64,
    pub life: f64,
    pub born: f64,
    pub key: f64,
    pub emitter: usize,
    pub burst: i64,
    /// Where it was born and how fast it was going, for its trail.
    x0: f64,
    y0: f64,
    vx0: f64,
    vy0: f64,
}

/// The particles of one open `particles` call, and which one is current.
#[derive(Clone, Debug, Default)]
pub(crate) struct ParticleSet {
    pub items: Vec<Particle>,
    pub emitters: Vec<Emitter>,
    pub current: Option<usize>,
    pub seed: i64,
    pub t: f64,
}

fn num(d: &Value, key: &str, default: f64) -> Result<f64, String> {
    match d.get(key) {
        None => Ok(default),
        Some(v) => v
            .as_f64()
            .ok_or_else(|| format!("particles: /{key} must be a number")),
    }
}

/// A number or `[lo hi]`.
fn range(d: &Value, key: &str, default: f64) -> Result<(f64, f64), String> {
    match d.get(key) {
        None => Ok((default, default)),
        Some(v) => {
            if let Some(x) = v.as_f64() {
                return Ok((x, x));
            }
            match v.as_numbers().as_deref() {
                Some([a, b]) => Ok((*a, *b)),
                _ => Err(format!("particles: /{key} must be a number or [low high]")),
            }
        }
    }
}

/// `[x y]`; for /Gravity a bare number is a pull downwards.
fn pair(d: &Value, key: &str, down: bool) -> Result<(f64, f64), String> {
    match d.get(key) {
        None => Ok((0.0, 0.0)),
        Some(v) => {
            if let (true, Some(g)) = (down, v.as_f64()) {
                return Ok((0.0, -g));
            }
            match v.as_numbers().as_deref() {
                Some([a, b]) => Ok((*a, *b)),
                _ => Err(format!(
                    "particles: /{key} must be [x y]{}",
                    if down { " or a number (downwards)" } else { "" }
                )),
            }
        }
    }
}

impl Emitter {
    fn parse(d: &Value, centre: (f64, f64), index: usize) -> Result<Emitter, String> {
        if !matches!(d, Value::Dict(_)) {
            return Err("particles takes an emitter dictionary, or an array of them".into());
        }
        let at = match d.get("At") {
            None => Place::Point(centre.0, centre.1),
            Some(v) => match v.as_numbers().as_deref() {
                Some([x, y]) => Place::Point(*x, *y),
                _ => match v {
                    Value::Array(items) => Place::Keys(items.clone()),
                    _ => {
                        return Err(
                            "particles: /At must be [x y] or keyframes [t [x y] /ease t [x y] ...]"
                                .into(),
                        );
                    }
                },
            },
        };
        let bursts = match d.get("Bursts") {
            None => Vec::new(),
            Some(v) => {
                if let Some(t) = v.as_f64() {
                    vec![t]
                } else {
                    v.as_numbers()
                        .ok_or("particles: /Bursts must be an array of times")?
                }
            }
        };
        let shape = match d.get("Shape") {
            None => Shape::Random,
            Some(Value::Name(n)) => match n.as_str() {
                "random" => Shape::Random,
                "even" => Shape::Even,
                "sphere" => Shape::Sphere,
                _ => return Err("particles: /Shape is /random, /even or /sphere".into()),
            },
            Some(_) => return Err("particles: /Shape is /random, /even or /sphere".into()),
        };
        let life = range(d, "Life", 2.0)?;
        if life.0 <= 0.0 || life.1 <= 0.0 {
            return Err("particles: /Life must be more than 0".into());
        }
        let count = num(d, "Count", 50.0)?;
        if !(0.0..=MAX_PARTICLES as f64).contains(&count) {
            return Err(format!("particles: /Count must be 0 to {MAX_PARTICLES}"));
        }
        let e = Emitter {
            at,
            area: match d.get("Area") {
                None => (0.0, 0.0),
                Some(_) => pair(d, "Area", false)?,
            },
            rate: num(d, "Rate", 0.0)?.max(0.0),
            from: num(d, "From", 0.0)?,
            to: num(d, "To", f64::INFINITY)?,
            bursts,
            count: count as usize,
            life,
            speed: range(d, "Speed", 100.0)?,
            angle: num(d, "Angle", 90.0)?,
            spread: num(d, "Spread", 360.0)?,
            shape,
            gravity: pair(d, "Gravity", true)?,
            drag: num(d, "Drag", 0.0)?.max(0.0),
            wind: pair(d, "Wind", false)?,
            gust: pair(d, "Gust", false)?,
            gust_freq: num(d, "GustFreq", 0.5)?,
            sway: pair(d, "Sway", false)?,
            sway_freq: num(d, "SwayFreq", 0.7)?,
            inherit: num(d, "Inherit", 0.0)?,
            period: num(d, "Period", 0.0)?.max(0.0),
            salt: num(d, "Seed", index as f64)?,
            gust_cells: Vec::new(),
            gust_j0: 0.0,
        };
        if e.rate * e.life.0.max(e.life.1) > MAX_PARTICLES as f64 {
            return Err(format!(
                "particles: /Rate x /Life would keep more than {MAX_PARTICLES} alive"
            ));
        }
        Ok(e)
    }

    fn max_life(&self) -> f64 {
        self.life.0.max(self.life.1)
    }

    /// The length of one gust cell: 1/60 s, or for a periodic emitter the
    /// nearest length that divides its period, so the grid (and so every
    /// path) repeats exactly each period.
    fn cell(&self) -> f64 {
        if self.period > 0.0 {
            self.period / (self.period / GUST_CELL).round().max(1.0)
        } else {
            GUST_CELL
        }
    }

    fn gusty(&self) -> bool {
        self.gust != (0.0, 0.0)
    }

    fn place_at(&self, ctx: &Shared, t: f64) -> Result<(f64, f64), PsError> {
        match &self.at {
            Place::Point(x, y) => Ok((*x, *y)),
            Place::Keys(items) => {
                let v = eval_keys(ctx, items, t)?;
                match v.as_numbers().as_deref() {
                    Some([x, y]) => Ok((*x, *y)),
                    _ => fail(
                        ctx,
                        PsError::Typecheck,
                        "particles: /At keyframes must hold points [x y]",
                    ),
                }
            }
        }
    }

    /// Noise over time for this emitter: `wiggle`, or for a periodic
    /// emitter the same noise sampled on a circle, so it repeats.
    fn noise(&self, seed: i64, key: f64, s: f64, freq: f64) -> f64 {
        if self.period > 0.0 {
            let a = s / self.period * std::f64::consts::TAU;
            let r = freq * self.period / std::f64::consts::TAU;
            noise2(seed, key, r * a.cos(), r * a.sin())
        } else {
            wiggle(seed, key, s * freq)
        }
    }

    /// The gust cells particles alive at `t` have seen, as (first cell,
    /// count), or None when no particle needs a gust; an error when the
    /// span is too long to follow. Nothing is computed yet, so the caller
    /// can charge the work first.
    fn table_span(&self, t: f64) -> Result<Option<(f64, usize)>, String> {
        if !self.gusty() {
            return Ok(None);
        }
        let first_burst = self.bursts.iter().copied().fold(f64::INFINITY, f64::min);
        let first_stream = if self.rate > 0.0 {
            self.from
        } else {
            f64::INFINITY
        };
        let first = first_burst.min(first_stream);
        if !first.is_finite() || first > t {
            return Ok(None); // nothing emitted yet, so no particle needs a gust
        }
        // Nothing alive: every particle was born more than a life ago.
        let last_burst = self
            .bursts
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let last_stream = if self.rate > 0.0 {
            self.to
        } else {
            f64::NEG_INFINITY
        };
        if t - last_burst.max(last_stream) > self.max_life() {
            return Ok(None);
        }
        let start = (t - self.max_life()).max(first);
        let h = self.cell();
        let j0 = (start / h).floor() - 1.0;
        let j1 = (t / h).floor() + 1.0;
        // Cells are counted from j0 in floating point: past 2^53 adding one
        // no longer moves, so such times are refused, not looped over.
        if !(j0.is_finite() && j1.is_finite())
            || j0.abs() > 9.0e15
            || j1.abs() > 9.0e15
            || j1 - j0 > MAX_GUST_CELLS as f64
        {
            return Err(format!(
                "particles: a /Gust over {:.0} s of particle life is too long to follow (at most {:.0} s): shorten /Life, or start emitting later",
                t - start,
                MAX_GUST_CELLS as f64 * h
            ));
        }
        Ok(Some((j0, (j1 - j0).max(0.0) as usize + 1)))
    }

    /// Tabulate the gust over `span` (from [`Emitter::table_span`]).
    fn fill_table(&mut self, seed: i64, span: Option<(f64, usize)>) {
        self.gust_cells = Vec::new();
        let Some((j0, count)) = span else {
            return;
        };
        self.gust_j0 = j0;
        self.gust_cells.reserve(count);
        for k in 0..count {
            let g = self.gust_in(seed, j0 + k as f64);
            self.gust_cells.push(g);
        }
    }

    /// The gust in grid cell `j`, from the table when it is there.
    fn gust_cell(&self, seed: i64, j: f64) -> (f64, f64) {
        let k = j - self.gust_j0;
        if k >= 0.0 && (k as usize) < self.gust_cells.len() {
            self.gust_cells[k as usize]
        } else {
            self.gust_in(seed, j)
        }
    }

    /// The gusting part of the wind in grid cell `j`.
    fn gust_in(&self, seed: i64, j: f64) -> (f64, f64) {
        let s = (j + 0.5) * self.cell();
        (
            self.gust.0 * self.noise(seed, self.salt * 7.0 + 0.25, s, self.gust_freq),
            self.gust.1 * self.noise(seed, self.salt * 7.0 + 0.75, s, self.gust_freq),
        )
    }

    /// A particle's own sway: a wandering offset, in units.
    fn sway_at(&self, seed: i64, key: f64, s: f64) -> (f64, f64) {
        if self.sway == (0.0, 0.0) {
            return (0.0, 0.0);
        }
        (
            self.sway.0 * self.noise(seed, key + 0.125, s, self.sway_freq),
            self.sway.1 * self.noise(seed, key + 0.625, s, self.sway_freq),
        )
    }
}

/// Move a body through `dt` seconds under a constant acceleration `a` and
/// linear drag `k`, exactly.
fn advance(
    p: (f64, f64),
    v: (f64, f64),
    a: (f64, f64),
    k: f64,
    dt: f64,
) -> ((f64, f64), (f64, f64)) {
    if dt <= 0.0 {
        return (p, v);
    }
    if k < 1e-9 {
        return (
            (
                p.0 + v.0 * dt + 0.5 * a.0 * dt * dt,
                p.1 + v.1 * dt + 0.5 * a.1 * dt * dt,
            ),
            (v.0 + a.0 * dt, v.1 + a.1 * dt),
        );
    }
    let e = (-k * dt).exp();
    let f = (1.0 - e) / k;
    let term = (a.0 / k, a.1 / k);
    (
        (
            p.0 + term.0 * dt + (v.0 - term.0) * f,
            p.1 + term.1 * dt + (v.1 - term.1) * f,
        ),
        (term.0 + (v.0 - term.0) * e, term.1 + (v.1 - term.1) * e),
    )
}

/// Where a particle born at `born` at `p0` with `v0` is at time `t`, and
/// how fast it is going (sway not included).
fn flight(
    e: &Emitter,
    seed: i64,
    p0: (f64, f64),
    v0: (f64, f64),
    born: f64,
    t: f64,
) -> ((f64, f64), (f64, f64)) {
    let base = (e.gravity.0 + e.wind.0, e.gravity.1 + e.wind.1);
    if !e.gusty() {
        return advance(p0, v0, base, e.drag, t - born);
    }
    // The gust is held constant over each cell of a grid anchored at time 0,
    // so a particle's path depends on its birth and the time alone.
    let (mut p, mut v, mut s) = (p0, v0, born);
    let k = e.drag;
    // Whole cells all take the same factors: work them out once.
    let h = e.cell();
    let (eh, fh) = if k < 1e-9 {
        (1.0, h)
    } else {
        ((-k * h).exp(), (1.0 - (-k * h).exp()) / k)
    };
    // Count cells by index, so a boundary that rounds onto s cannot skip one.
    let mut j = (s / h).floor();
    while s < t {
        let end = ((j + 1.0) * h).min(t);
        let g = e.gust_cell(seed, j);
        let a = (base.0 + g.0, base.1 + g.1);
        if end - s >= h * (1.0 - 1e-9) {
            if k < 1e-9 {
                p = (
                    p.0 + v.0 * h + 0.5 * a.0 * h * h,
                    p.1 + v.1 * h + 0.5 * a.1 * h * h,
                );
                v = (v.0 + a.0 * h, v.1 + a.1 * h);
            } else {
                let term = (a.0 / k, a.1 / k);
                p = (
                    p.0 + term.0 * h + (v.0 - term.0) * fh,
                    p.1 + term.1 * h + (v.1 - term.1) * fh,
                );
                v = (term.0 + (v.0 - term.0) * eh, term.1 + (v.1 - term.1) * eh);
            }
        } else {
            let next = advance(p, v, a, k, end - s);
            p = next.0;
            v = next.1;
        }
        s = end;
        j += 1.0;
    }
    (p, v)
}

/// The particle with key `key`, born at `born`, as it is at `t`; None if
/// it is not alive then.
#[allow(clippy::too_many_arguments)]
fn particle(
    ctx: &Shared,
    e: &Emitter,
    seed: i64,
    index: usize,
    key: f64,
    born: f64,
    t: f64,
    slot: (usize, usize),
    burst: i64,
) -> Result<Option<Particle>, PsError> {
    let age = t - born;
    if age < 0.0 {
        return Ok(None);
    }
    let r = |m: f64| hash01(seed, &[key, m]);
    let life = e.life.0 + (e.life.1 - e.life.0) * r(1.0);
    if age >= life {
        return Ok(None);
    }
    let (k, n) = slot;
    let (ax, ay) = e.place_at(ctx, born)?;
    let p0 = (
        ax + (r(2.0) - 0.5) * e.area.0,
        ay + (r(3.0) - 0.5) * e.area.1,
    );
    let mut speed = e.speed.0 + (e.speed.1 - e.speed.0) * r(4.0);
    let dir = match e.shape {
        Shape::Even if n > 0 => {
            // Evenly round a full circle; across a partial spread, ends included.
            let full = e.spread.abs() >= 359.999;
            let step = if full || n == 1 {
                e.spread / n as f64
            } else {
                e.spread / (n - 1) as f64
            };
            let first = if full || n == 1 { 0.0 } else { -e.spread / 2.0 };
            e.angle + first + step * k as f64
        }
        _ => e.angle + (r(5.0) - 0.5) * e.spread,
    };
    if e.shape == Shape::Sphere {
        // A sphere of sparks seen from the side: more of them near the rim.
        let z = r(6.0) * 2.0 - 1.0;
        speed *= (1.0 - z * z).sqrt();
    }
    let rad = dir.to_radians();
    let mut v0 = (speed * rad.cos(), speed * rad.sin());
    if e.inherit != 0.0
        && let Place::Keys(_) = e.at
    {
        let h = 1.0 / 240.0;
        let (a, b) = (e.place_at(ctx, born - h)?, e.place_at(ctx, born + h)?);
        v0.0 += e.inherit * (b.0 - a.0) / (2.0 * h);
        v0.1 += e.inherit * (b.1 - a.1) / (2.0 * h);
    }
    if e.gusty() {
        charge_gust(ctx, age / e.cell() + 1.0)?;
    }
    let (mut p, mut v) = flight(e, seed, p0, v0, born, t);
    if e.sway != (0.0, 0.0) {
        let (s1, s0) = (e.sway_at(seed, key, t), e.sway_at(seed, key, born));
        p.0 += s1.0 - s0.0;
        p.1 += s1.1 - s0.1;
        let h = 1.0 / 240.0;
        let (a, b) = (e.sway_at(seed, key, t - h), e.sway_at(seed, key, t + h));
        v.0 += (b.0 - a.0) / (2.0 * h);
        v.1 += (b.1 - a.1) / (2.0 * h);
    }
    Ok(Some(Particle {
        x: p.0,
        y: p.1,
        vx: v.0,
        vy: v.1,
        age,
        life,
        born,
        key,
        emitter: index,
        burst,
        x0: p0.0,
        y0: p0.1,
        vx0: v0.0,
        vy0: v0.1,
    }))
}

/// A whole number standing for one particle, for `[key i] hash01`.
fn key_of(seed: i64, parts: &[f64]) -> f64 {
    (hash01(seed, parts) * 4_294_967_296.0).floor()
}

/// Every particle of `emitters` alive at `t`, emitter by emitter, oldest
/// first within each.
pub(crate) fn live(
    ctx: &Shared,
    emitters: &[Emitter],
    seed: i64,
    t: f64,
) -> Result<Vec<Particle>, PsError> {
    let mut out = Vec::new();
    let too_many = |ctx: &Shared| {
        fail(
            ctx,
            PsError::Limitcheck,
            format!(
                "particles: more than {MAX_PARTICLES} alive at once; lower /Count, /Rate or /Life"
            ),
        )
    };
    for (ei, e) in emitters.iter().enumerate() {
        let max_life = e.max_life();
        // Bursts, in time order.
        let mut bursts: Vec<(usize, f64)> = e.bursts.iter().copied().enumerate().collect();
        bursts.sort_by(|a, b| a.1.total_cmp(&b.1));
        for (bi, bt) in bursts {
            if bt > t || t - bt >= max_life {
                continue;
            }
            for k in 0..e.count {
                let key = key_of(seed, &[e.salt, 1.0, bi as f64, k as f64]);
                if let Some(p) = particle(ctx, e, seed, ei, key, bt, t, (k, e.count), bi as i64)? {
                    if out.len() >= MAX_PARTICLES {
                        return too_many(ctx);
                    }
                    out.push(p);
                }
            }
        }
        // Continuous emission: slot n is born at from + (n + jitter) / rate.
        if e.rate > 0.0 {
            let (from, rate, wrap) = if e.period > 0.0 {
                let n = (e.rate * e.period).round().max(1.0);
                (0.0, n / e.period, Some(n))
            } else {
                (e.from, e.rate, None)
            };
            let first = ((t - max_life - from) * rate).floor() - 1.0;
            let first = if wrap.is_none() {
                first.max(0.0)
            } else {
                first
            };
            let mut last = ((t - from) * rate).floor();
            if wrap.is_none() && e.to.is_finite() {
                // Nothing is emitted after /To, however late it is now.
                last = last.min(((e.to - from) * rate).ceil());
            }
            if first > last {
                continue;
            }
            // Slots are counted in floating point: past 2^53 adding one no
            // longer moves, so refuse such a time rather than loop for ever
            // (this loop is native and does not see the budgets).
            if !(first.is_finite() && last.is_finite())
                || first.abs() > 9.0e15
                || last.abs() > 9.0e15
            {
                return fail(
                    ctx,
                    PsError::Limitcheck,
                    "particles: the time is too large to count a stream's particles at; a stream that keeps emitting needs a time it can count to",
                );
            }
            let span = (last - first) as u64 + 1;
            if span > 4 * MAX_PARTICLES as u64 + 4 {
                return too_many(ctx);
            }
            for k in 0..span {
                let n = first + k as f64;
                let slot = match wrap {
                    Some(m) => n.rem_euclid(m),
                    None => n,
                };
                let jitter = hash01(seed, &[e.salt, 2.0, slot]);
                let born = from + (n + jitter) / rate;
                if born <= t && (wrap.is_some() || (born >= from && born < e.to)) {
                    let key = key_of(seed, &[e.salt, 3.0, slot]);
                    if let Some(p) = particle(ctx, e, seed, ei, key, born, t, (0, 0), -1)? {
                        if out.len() >= MAX_PARTICLES {
                            return too_many(ctx);
                        }
                        out.push(p);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// The track a particle took over its last `back` seconds, as `n + 1`
/// points relative to where it is now, oldest first.
pub(crate) fn trail(set: &ParticleSet, p: &Particle, back: f64, n: usize) -> Vec<(f64, f64)> {
    let e = &set.emitters[p.emitter];
    let start = (set.t - back.max(0.0)).max(p.born);
    let n = n.max(1);
    (0..=n)
        .map(|i| {
            let s = start + (set.t - start) * i as f64 / n as f64;
            let (mut q, _) = flight(e, set.seed, (p.x0, p.y0), (p.vx0, p.vy0), p.born, s);
            if e.sway != (0.0, 0.0) {
                let (s1, s0) = (
                    e.sway_at(set.seed, p.key, s),
                    e.sway_at(set.seed, p.key, p.born),
                );
                q.0 += s1.0 - s0.0;
                q.1 += s1.1 - s0.1;
            }
            (q.0 - p.x, q.1 - p.y)
        })
        .collect()
}

fn current(ctx: &Shared, word: &str) -> Result<Particle, PsError> {
    let c = ctx.borrow();
    match c
        .particles
        .last()
        .and_then(|s| s.current.map(|i| s.items[i]))
    {
        Some(p) => Ok(p),
        None => {
            drop(c);
            fail(
                ctx,
                PsError::Undefined(word.to_string()),
                format!(
                    "`{word}` reads the particle being drawn: call it inside the procedure given to `particles`"
                ),
            )
        }
    }
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    // spec .particlesbegin n     compute the live particles; open the set.
    reg(interp, ctx, ".particlesbegin", |i, ctx| {
        let spec = i.pop_value()?;
        let centre = {
            let c = ctx.borrow();
            let (w, h) = c.views.last().copied().unwrap_or((c.width, c.height));
            (w / 2.0, h / 2.0)
        };
        let dicts: Vec<Value> = match spec {
            Value::Array(a) => a,
            d => vec![d],
        };
        let (seed, t) = {
            let c = ctx.borrow();
            (c.seed(), c.scope().t)
        };
        let mut emitters = Vec::with_capacity(dicts.len());
        for (k, d) in dicts.iter().enumerate() {
            match Emitter::parse(d, centre, k) {
                Ok(mut e) => {
                    // Every emitter's table counts against the moment's
                    // gust budget before it is built.
                    let span = match e.table_span(t) {
                        Ok(span) => span,
                        Err(msg) => return fail(ctx, PsError::Limitcheck, msg),
                    };
                    if let Some((_, count)) = span {
                        charge_gust(ctx, count as f64)?;
                    }
                    e.fill_table(seed, span);
                    emitters.push(e)
                }
                Err(msg) => return fail(ctx, PsError::Typecheck, msg),
            }
        }
        let items = live(ctx, &emitters, seed, t)?;
        let n = items.len();
        let mut c = ctx.borrow_mut();
        if c.particles.len() >= 16 {
            drop(c);
            return fail(
                ctx,
                PsError::Limitcheck,
                "particles nested more than 16 deep",
            );
        }
        c.particles.push(ParticleSet {
            items,
            emitters,
            current: None,
            seed,
            t,
        });
        drop(c);
        i.push_int(n as i64);
        Ok(())
    });
    // i .particlepick x y emitter     make particle i current.
    reg(interp, ctx, ".particlepick", |i, ctx| {
        let k = i.pop_int()?;
        let mut c = ctx.borrow_mut();
        let Some(set) = c.particles.last_mut() else {
            return Err(PsError::Undefined(".particlepick".into()));
        };
        let p = *set
            .items
            .get(usize::try_from(k).map_err(|_| PsError::Rangecheck)?)
            .ok_or(PsError::Rangecheck)?;
        set.current = Some(k as usize);
        drop(c);
        i.push_real(p.x);
        i.push_real(p.y);
        i.push_int(p.emitter as i64);
        Ok(())
    });
    reg(interp, ctx, ".particlesend", |_, ctx| {
        ctx.borrow_mut().particles.pop();
        Ok(())
    });
    reg(interp, ctx, "particleage", |i, ctx| {
        i.push_real(current(ctx, "particleage")?.age);
        Ok(())
    });
    reg(interp, ctx, "particleu", |i, ctx| {
        let p = current(ctx, "particleu")?;
        i.push_real((p.age / p.life).clamp(0.0, 1.0));
        Ok(())
    });
    reg(interp, ctx, "particlelife", |i, ctx| {
        i.push_real(current(ctx, "particlelife")?.life);
        Ok(())
    });
    reg(interp, ctx, "particlepos", |i, ctx| {
        let p = current(ctx, "particlepos")?;
        i.push_real(p.x);
        i.push_real(p.y);
        Ok(())
    });
    reg(interp, ctx, "particlevel", |i, ctx| {
        let p = current(ctx, "particlevel")?;
        i.push_real(p.vx);
        i.push_real(p.vy);
        Ok(())
    });
    reg(interp, ctx, "particlekey", |i, ctx| {
        i.push_real(current(ctx, "particlekey")?.key);
        Ok(())
    });
    reg(interp, ctx, "particleemitter", |i, ctx| {
        i.push_int(current(ctx, "particleemitter")?.emitter as i64);
        Ok(())
    });
    reg(interp, ctx, "particleburst", |i, ctx| {
        i.push_int(current(ctx, "particleburst")?.burst);
        Ok(())
    });
    // seconds n particletrail -     the particle's track as the current path.
    reg(interp, ctx, "particletrail", |i, ctx| {
        let n = i.pop_int()?;
        let back = i.pop_f64()?;
        let p = current(ctx, "particletrail")?;
        let n = n.clamp(1, 512);
        {
            let c = ctx.borrow();
            let set = c.particles.last().expect("current() found a set");
            let cost = if set.emitters[p.emitter].gusty() {
                (n as f64 + 1.0) * ((set.t - p.born) / set.emitters[p.emitter].cell() + 1.0)
            } else {
                0.0
            };
            drop(c);
            charge_gust(ctx, cost)?;
        }
        let pts = {
            let c = ctx.borrow();
            let set = c.particles.last().expect("current() found a set");
            trail(set, &p, back, n as usize)
        };
        let mut segs = Vec::with_capacity(pts.len());
        for (k, (x, y)) in pts.into_iter().enumerate() {
            segs.push(if k == 0 {
                PathSegment::MoveTo(x, y)
            } else {
                PathSegment::LineTo(x, y)
            });
        }
        i.set_current_path(&segs)?;
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_free_flight_is_ballistic() {
        let (p, v) = advance((0.0, 0.0), (10.0, 20.0), (0.0, -10.0), 0.0, 2.0);
        assert!((p.0 - 20.0).abs() < 1e-9 && (p.1 - 20.0).abs() < 1e-9);
        assert!((v.0 - 10.0).abs() < 1e-9 && (v.1 - 0.0).abs() < 1e-9);
    }

    #[test]
    fn drag_settles_at_terminal_velocity() {
        let (_, v) = advance((0.0, 0.0), (300.0, 0.0), (0.0, -100.0), 2.0, 20.0);
        assert!(v.0.abs() < 1e-6, "{v:?}");
        assert!((v.1 + 50.0).abs() < 1e-6, "{v:?}");
    }

    #[test]
    fn steps_compose_exactly() {
        // Two half steps land where one whole step does.
        let a = (3.0, -40.0);
        let (p1, v1) = advance((1.0, 2.0), (50.0, 80.0), a, 1.3, 0.4);
        let (p2, v2) = advance(p1, v1, a, 1.3, 0.6);
        let (p, v) = advance((1.0, 2.0), (50.0, 80.0), a, 1.3, 1.0);
        for (x, y) in [(p2.0, p.0), (p2.1, p.1), (v2.0, v.0), (v2.1, v.1)] {
            assert!((x - y).abs() < 1e-9, "{x} {y}");
        }
    }
}
