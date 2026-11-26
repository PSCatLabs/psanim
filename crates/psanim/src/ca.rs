//! Cellular automata: grids of cells that run a rule, natively.
//!
//! A grid is a PostScript string of one byte per cell, row after row from
//! the top, with its width given alongside. In PostScript a generation of
//! Life on 200 x 200 cells took about 50 ms, so a film could not show
//! hundreds of generations a second, and a pure movie (or a component,
//! which cannot `step`) could not reach generation 1000 at all. Here a
//! generation of 200 x 200 is a fraction of a millisecond.
//!
//! What a byte means:
//!
//! - `128..=255`: alive; `v - 128` is how many generations it has lived
//!   (held at 127).
//! - `1..=127`: dead after having lived; `127 - v` is how many generations
//!   ago it died (held at 126, so 1 means "lived here once").
//! - `0`: never alive.
//!
//! So one string carries the state and the history a film needs to fade a
//! dying cell (`cellcolors`). Brian's Brain uses the same bytes: its
//! "dying" state is 127, a cell that died this generation.
//!
//! The words are pure functions of their operands. `cellsteps` keeps a
//! cache of the states it has computed (keyed on everything it was given)
//! so that a pure film asking for generation n, then n + 1, does one
//! generation of work, not n + 1; the answer is the same either way.

use std::collections::HashMap;

use pscat::host::Value;
use pscat::{Interp, PsError};

use crate::ops::{Shared, fail, reg};

/// The most cells one grid may have (2048 x 2048).
pub const MAX_CELLS: usize = 1 << 22;
/// The most cell updates the words may do in one moment (a frame, or a
/// tick): under half a second of native work on an Apple M4. They are also charged to
/// the interpreter's budgets as they go.
pub const MAX_WORK: f64 = 1_073_741_824.0;
/// Cell updates charged as one interpreter step.
const CELLS_PER_STEP: u64 = 256;
/// States `cellsteps` remembers, across runs: grids with their rules.
const CACHE_ENTRIES: usize = 8;
/// Generations between remembered states, in cell updates.
const CHECKPOINT_WORK: u64 = 1 << 24;
/// The latest states kept per grid, besides the checkpoints.
const RECENT: usize = 3;
/// Remembered states per grid before every other one is dropped.
const MAX_CHECKPOINTS: usize = 48;

/// A rule for a two-dimensional grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rule {
    /// Born with these neighbour counts, survives with these (bit k for k
    /// neighbours): `B3/S23` is Conway's Life.
    LifeLike { born: u16, survive: u16 },
    /// Brian's Brain: an off cell with exactly two live neighbours is
    /// born, a live cell always dies, a dying cell rests one generation.
    Brain,
}

/// What lies beyond the edge of the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Edge {
    /// Dead cells, for ever.
    Dead,
    /// The opposite edge: the grid is a torus.
    Wrap,
    /// An open plane, as near as a finite grid gets: dead cells beyond the
    /// edge, and a lone glider that comes within three cells of the edge is
    /// taken off (it would fly on for ever, meeting nothing). Exact while
    /// nothing else reaches the edge.
    Open,
}

/// How near the edge a glider is taken off, in cells. A grid evolves as
/// the open plane would while no live cell is on its outermost row or
/// column; a glider moves a cell every four generations, so this catches
/// it first.
const OPEN_BAND: usize = 3;

/// Take off the lone gliders near the edge of `g`: a group of exactly five
/// live cells, joined, in a 3 x 3 box, with no other live cell within two
/// cells of it. They become cells that died this generation, so they fade.
fn release_gliders(g: &mut [u8], cols: usize, rows: usize) {
    let near = |x: usize, y: usize| {
        x < OPEN_BAND || y < OPEN_BAND || x + OPEN_BAND >= cols || y + OPEN_BAND >= rows
    };
    let alive = |g: &[u8], x: i64, y: i64| {
        x >= 0
            && y >= 0
            && (x as usize) < cols
            && (y as usize) < rows
            && g[y as usize * cols + x as usize] >= 128
    };
    let mut seen = vec![false; g.len()];
    let mut edge_cells = Vec::new();
    for y in 0..rows {
        for x in 0..cols {
            if near(x, y) && g[y * cols + x] >= 128 {
                edge_cells.push((x, y));
            }
        }
    }
    for (sx, sy) in edge_cells {
        if seen[sy * cols + sx] || g[sy * cols + sx] < 128 {
            continue;
        }
        // The group: live cells joined to this one (8-neighbours), at most 6 looked at.
        let mut group = vec![(sx as i64, sy as i64)];
        seen[sy * cols + sx] = true;
        let mut k = 0;
        while k < group.len() && group.len() <= 5 {
            let (x, y) = group[k];
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if alive(g, nx, ny) && !seen[ny as usize * cols + nx as usize] {
                        seen[ny as usize * cols + nx as usize] = true;
                        group.push((nx, ny));
                    }
                }
            }
            k += 1;
        }
        if group.len() != 5 {
            continue;
        }
        let (x0, x1) = (
            group.iter().map(|c| c.0).min().unwrap_or(0),
            group.iter().map(|c| c.0).max().unwrap_or(0),
        );
        let (y0, y1) = (
            group.iter().map(|c| c.1).min().unwrap_or(0),
            group.iter().map(|c| c.1).max().unwrap_or(0),
        );
        if x1 - x0 != 2 || y1 - y0 != 2 {
            continue;
        }
        // Alone: nothing else alive within two cells of its box.
        let mut others = 0;
        for y in y0 - 2..=y1 + 2 {
            for x in x0 - 2..=x1 + 2 {
                if alive(g, x, y) {
                    others += 1;
                }
            }
        }
        if others == 5 {
            for (x, y) in group {
                g[y as usize * cols + x as usize] = 127;
            }
        }
    }
}

/// The named rules, as `B/S` text.
const NAMED_RULES: &[(&str, &str)] = &[
    ("life", "B3/S23"),
    ("highlife", "B36/S23"),
    ("seeds", "B2/S"),
    ("daynight", "B3678/S34678"),
    ("maze", "B3/S12345"),
    ("replicator", "B1357/S1357"),
    ("lifewithoutdeath", "B3/S012345678"),
];

/// A rule from its name (`life`, `brain`, ...) or `B3/S23` text.
pub fn parse_rule(text: &str) -> Result<Rule, String> {
    let t = text.trim().to_ascii_lowercase();
    if t == "brain" {
        return Ok(Rule::Brain);
    }
    let spec = NAMED_RULES
        .iter()
        .find(|r| r.0 == t)
        .map_or(t.clone(), |r| r.1.to_ascii_lowercase());
    let bad = || {
        format!(
            "rule ({text}): give a name ({}, brain) or births and survivals as B3/S23",
            NAMED_RULES
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let (b, s) = spec.split_once('/').ok_or_else(bad)?;
    let digits = |part: &str, tag: char| -> Result<u16, String> {
        let rest = part.strip_prefix(tag).ok_or_else(bad)?;
        let mut bits = 0u16;
        for ch in rest.chars() {
            let d = ch.to_digit(10).filter(|d| *d <= 8).ok_or_else(bad)?;
            bits |= 1 << d;
        }
        Ok(bits)
    };
    Ok(Rule::LifeLike {
        born: digits(b, 'b')?,
        survive: digits(s, 's')?,
    })
}

/// One generation of `rule` on a grid `cols` wide.
pub fn step(cells: &[u8], cols: usize, rule: Rule, edge: Edge) -> Vec<u8> {
    let rows = cells.len() / cols.max(1);
    let (pw, ph) = (cols + 2, rows + 2);
    // Live cells as 0/1, with a border of what lies past the edge.
    let mut a = vec![0u8; pw * ph];
    for y in 0..rows {
        for x in 0..cols {
            a[(y + 1) * pw + x + 1] = u8::from(cells[y * cols + x] >= 128);
        }
    }
    if edge == Edge::Wrap && cols > 0 && rows > 0 {
        for y in 1..=rows {
            a[y * pw] = a[y * pw + cols];
            a[y * pw + cols + 1] = a[y * pw + 1];
        }
        let (top, bottom) = (pw, rows * pw);
        a.copy_within(bottom..bottom + pw, 0);
        a.copy_within(top..top + pw, (rows + 1) * pw);
    }
    let mut out = vec![0u8; cells.len()];
    let open = edge == Edge::Open;
    for y in 0..rows {
        let (up, mid, down) = (y * pw, (y + 1) * pw, (y + 2) * pw);
        for x in 0..cols {
            let n = a[up + x]
                + a[up + x + 1]
                + a[up + x + 2]
                + a[mid + x]
                + a[mid + x + 2]
                + a[down + x]
                + a[down + x + 1]
                + a[down + x + 2];
            let v = cells[y * cols + x];
            let decay = if v > 1 && v < 128 { v - 1 } else { v };
            out[y * cols + x] = match rule {
                Rule::LifeLike { born, survive } => {
                    if v >= 128 {
                        if survive & (1 << n) != 0 {
                            v.saturating_add(1)
                        } else {
                            127
                        }
                    } else if born & (1 << n) != 0 {
                        128
                    } else {
                        decay
                    }
                }
                Rule::Brain => {
                    if v >= 128 {
                        127
                    } else if v != 127 && n == 2 {
                        128
                    } else {
                        decay
                    }
                }
            };
        }
    }
    if open {
        release_gliders(&mut out, cols, rows);
    }
    out
}

/// The history of an elementary (one-dimensional) automaton: `rows` rows
/// of `row.len()` cells, the first being `row`. A cell of the next row is
/// bit `4l + 2c + r` of `rule`, where l c r are the cells above-left,
/// above and above-right. Live cells are 255, dead 0.
pub fn elementary(row: &[u8], rule: u8, rows: usize, edge: Edge) -> Vec<u8> {
    let cols = row.len();
    let mut out = Vec::with_capacity(cols * rows);
    let mut cur: Vec<u8> = row.iter().map(|v| u8::from(*v >= 128)).collect();
    for _ in 0..rows {
        out.extend(cur.iter().map(|c| c * 255));
        let at = |x: isize| -> u8 {
            if x >= 0 && (x as usize) < cols {
                cur[x as usize]
            } else if edge == Edge::Wrap && cols > 0 {
                cur[x.rem_euclid(cols as isize) as usize]
            } else {
                0
            }
        };
        cur = (0..cols as isize)
            .map(|x| (rule >> ((at(x - 1) << 2) | (at(x) << 1) | at(x + 1))) & 1)
            .collect();
    }
    out
}

// ---- patterns -----------------------------------------------------------------

/// Named starting patterns, as LifeWiki plaintext (`O` alive, `.` dead).
pub const PATTERNS: &[(&str, &[&str])] = &[
    ("glider", &[".O.", "..O", "OOO"]),
    ("blinker", &["OOO"]),
    ("block", &["OO", "OO"]),
    ("beacon", &["OO..", "OO..", "..OO", "..OO"]),
    ("lwss", &[".O..O", "O....", "O...O", "OOOO."]),
    ("rpentomino", &[".OO", "OO.", ".O."]),
    ("acorn", &[".O.....", "...O...", "OO..OOO"]),
    ("diehard", &["......O.", "OO......", ".O...OOO"]),
    (
        "gun",
        &[
            "........................O...........",
            "......................O.O...........",
            "............OO......OO............OO",
            "...........O...O....OO............OO",
            "OO........O.....O...OO..............",
            "OO........O...O.OO....O.O...........",
            "..........O.....O.......O...........",
            "...........O...O....................",
            "............OO......................",
        ],
    ),
    (
        "pulsar",
        &[
            "..OOO...OOO..",
            ".............",
            "O....O.O....O",
            "O....O.O....O",
            "O....O.O....O",
            "..OOO...OOO..",
            ".............",
            "..OOO...OOO..",
            "O....O.O....O",
            "O....O.O....O",
            "O....O.O....O",
            ".............",
            "..OOO...OOO..",
        ],
    ),
    ("single", &["O"]),
];

/// A picture's live cells, as (x, y) from its top left, and its size.
type Shape = (Vec<(usize, usize)>, usize, usize);

fn alive_char(c: u8) -> bool {
    matches!(c, b'O' | b'o' | b'#' | b'*' | b'@' | b'1' | b'X' | b'x')
}

fn picture(rows: &[&[u8]]) -> Shape {
    let mut cells = Vec::new();
    let mut w = 0;
    for (y, r) in rows.iter().enumerate() {
        w = w.max(r.len());
        for (x, c) in r.iter().enumerate() {
            if alive_char(*c) {
                cells.push((x, y));
            }
        }
    }
    (cells, w, rows.len())
}

/// A pattern in run-length encoding (`bo$2bo$3o!`), as LifeWiki gives it.
/// Comment lines (`#...`) and the `x = .., y = ..` header are skipped.
pub fn rle(text: &str) -> Result<Shape, String> {
    let mut cells = Vec::new();
    let (mut x, mut y, mut w) = (0usize, 0usize, 0usize);
    let mut run = 0usize;
    'lines: for line in text.lines() {
        let l = line.trim();
        if l.starts_with('#') || l.starts_with("x ") || l.starts_with("x=") {
            continue;
        }
        for ch in l.chars() {
            let n = run.max(1);
            match ch {
                '0'..='9' => {
                    run = run * 10 + ch.to_digit(10).unwrap_or(0) as usize;
                    if run > 100_000 {
                        return Err("RLE: a run longer than 100000".into());
                    }
                    continue;
                }
                'b' | '.' => x += n,
                '$' => {
                    y += n;
                    x = 0;
                }
                '!' => break 'lines,
                c if c.is_ascii_alphabetic() => {
                    for k in 0..n {
                        cells.push((x + k, y));
                    }
                    x += n;
                }
                c if c.is_whitespace() => {}
                c => return Err(format!("RLE: unexpected `{c}`")),
            }
            run = 0;
            w = w.max(x);
            if cells.len() > MAX_CELLS {
                return Err("RLE: too many cells".into());
            }
        }
    }
    let h = cells.iter().map(|c| c.1 + 1).max().unwrap_or(0);
    Ok((cells, w, h))
}

fn shape_of(v: &Value) -> Result<Shape, String> {
    match v {
        Value::Name(n) => PATTERNS
            .iter()
            .find(|p| p.0 == n.as_str())
            .map(|p| picture(&p.1.iter().map(|r| r.as_bytes()).collect::<Vec<_>>()))
            .ok_or_else(|| {
                format!(
                    "no pattern /{n}: the named ones are {} and /random; or give a picture (an array of strings, O or # alive) or RLE text",
                    PATTERNS
                        .iter()
                        .map(|p| format!("/{}", p.0))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }),
        Value::String(s) => rle(&String::from_utf8_lossy(s)),
        Value::Array(rows) if rows.iter().all(|r| matches!(r, Value::String(_))) => {
            let rows: Vec<&[u8]> = rows
                .iter()
                .map(|r| match r {
                    Value::String(s) => s.as_slice(),
                    _ => &[],
                })
                .collect();
            Ok(picture(&rows))
        }
        _ => Err(
            "a pattern is a name (/glider), a picture (an array of strings) or RLE text".into(),
        ),
    }
}

/// How to fill a new grid.
#[derive(Clone, Debug)]
pub struct Seeding {
    /// What to place: a pattern centred, or [ [pattern x y] ... ] (x y the
    /// cell the pattern is centred on, from the top left), or /random.
    pub pattern: Value,
    pub density: f64,
    pub seed: i64,
}

/// A new grid: dead cells (0) and live ones of age 0 (128).
pub fn grid(cols: usize, rows: usize, how: &Seeding) -> Result<Vec<u8>, String> {
    let mut g = vec![0u8; cols * rows];
    let mut put = |shape: &Shape, cx: i64, cy: i64| {
        let (cells, w, h) = shape;
        let (x0, y0) = (cx - (*w as i64) / 2, cy - (*h as i64) / 2);
        for (x, y) in cells {
            let (gx, gy) = (x0 + *x as i64, y0 + *y as i64);
            if gx >= 0 && gy >= 0 && (gx as usize) < cols && (gy as usize) < rows {
                g[gy as usize * cols + gx as usize] = 128;
            }
        }
    };
    let (mx, my) = ((cols / 2) as i64, (rows / 2) as i64);
    match &how.pattern {
        Value::Name(n) if n == "random" => {
            for y in 0..rows {
                for x in 0..cols {
                    if crate::ctx::hash01(how.seed, &[x as f64, y as f64]) < how.density {
                        g[y * cols + x] = 128;
                    }
                }
            }
        }
        Value::Name(n) if n == "empty" => {}
        Value::Name(n) if n == "single" => put(&(vec![(0, 0)], 1, 1), mx, 0.max(my)),
        Value::Array(list) if list.iter().any(|v| matches!(v, Value::Array(_))) => {
            for item in list {
                let parts = item.as_array().unwrap_or(&[]);
                let ok =
                    parts.len() == 3 && parts[1].as_f64().is_some() && parts[2].as_f64().is_some();
                if !ok {
                    return Err("each placement is [ pattern x y ]: x y the cell its centre goes on, from the top left".into());
                }
                let shape = shape_of(&parts[0])?;
                put(
                    &shape,
                    parts[1].as_f64().unwrap_or(0.0).round() as i64,
                    parts[2].as_f64().unwrap_or(0.0).round() as i64,
                );
            }
        }
        v => put(&shape_of(v)?, mx, my),
    }
    Ok(g)
}

/// The box of the cells that are or once were alive, as [x0 y0 x1 y1]
/// (cells, from the top left, inclusive), counting only the rows and
/// columns that hold more than `share` of what the fullest one holds: 0.2
/// leaves out the thin tracks of gliders that flew away from a pattern.
/// None if nothing ever lived.
pub fn bounds(cells: &[u8], cols: usize, share: f64) -> Option<[usize; 4]> {
    let rows = cells.len() / cols.max(1);
    let mut colsum = vec![0usize; cols];
    let mut rowsum = vec![0usize; rows];
    for (k, v) in cells.iter().enumerate() {
        if *v > 0 {
            colsum[k % cols] += 1;
            rowsum[k / cols] += 1;
        }
    }
    let span = |sums: &[usize]| {
        let most = *sums.iter().max()?;
        let over = (most as f64 * share.clamp(0.0, 0.99)).floor() as usize;
        let over = over.min(most.saturating_sub(1));
        let lo = sums.iter().position(|s| *s > over)?;
        let hi = sums.iter().rposition(|s| *s > over)?;
        Some((lo, hi))
    };
    if cells.iter().all(|v| *v == 0) {
        return None;
    }
    let (x0, x1) = span(&colsum)?;
    let (y0, y1) = span(&rowsum)?;
    Some([x0, y0, x1, y1])
}

// ---- colour -------------------------------------------------------------------

/// How states become colours.
#[derive(Clone, Debug)]
pub struct Palette {
    pub born: [f64; 3],
    pub alive: [f64; 3],
    /// Generations a newborn cell takes to reach `alive`.
    pub mature: f64,
    pub trail: [f64; 3],
    /// Generations a dead cell takes to fade from `trail` to `ghost`.
    pub fade: f64,
    pub ghost: [f64; 3],
    pub ground: [f64; 3],
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t)
}

/// The colour of each state, `frac` of the way from this generation to the
/// next (a dying cell keeps fading between generations).
pub fn lut(p: &Palette, frac: f64) -> [[u8; 3]; 256] {
    let byte = |c: [f64; 3]| c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
    let mut t = [[0u8; 3]; 256];
    for (v, slot) in t.iter_mut().enumerate() {
        let c = if v >= 128 {
            let age = (v - 128) as f64;
            if p.mature > 0.0 {
                mix(p.born, p.alive, (age + frac) / p.mature)
            } else {
                p.alive
            }
        } else if v == 0 {
            p.ground
        } else {
            let since = (127 - v) as f64 + frac;
            if p.fade > 0.0 {
                let u = (since / p.fade).clamp(0.0, 1.0);
                mix(p.trail, p.ghost, 1.0 - (1.0 - u) * (1.0 - u))
            } else {
                p.ghost
            }
        };
        *slot = byte(c);
    }
    t
}

/// Three bytes per cell, for `colorimage`.
pub fn colors(cells: &[u8], table: &[[u8; 3]; 256]) -> Vec<u8> {
    let mut out = Vec::with_capacity(cells.len() * 3);
    for v in cells {
        out.extend_from_slice(&table[*v as usize]);
    }
    out
}

// ---- the cache ----------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    hash: u64,
    len: usize,
    cols: usize,
    rule: Rule,
    edge: Edge,
}

struct Entry {
    base: Vec<u8>,
    /// Generations between checkpoints, and the checkpoints, by generation.
    every: u64,
    checkpoints: Vec<(u64, Vec<u8>)>,
    /// The latest states asked for, newest last: a film that asks for
    /// generation n and n + 1 in one frame, then n again, costs nothing.
    recent: Vec<(u64, Vec<u8>)>,
    used: u64,
}

/// States `cellsteps` has computed, so the next generation costs one step.
#[derive(Default)]
pub struct Cache {
    entries: HashMap<Key, Entry>,
    clock: u64,
}

fn fnv(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in bytes {
        h = (h ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3);
    }
    h
}

impl Cache {
    /// Generation `n` of `base` under `rule`, starting from the nearest
    /// state already known. `charge` is told of each generation's work and
    /// may stop it.
    pub fn advance(
        &mut self,
        base: &[u8],
        cols: usize,
        rule: Rule,
        edge: Edge,
        n: u64,
        mut charge: impl FnMut(u64) -> Result<(), PsError>,
    ) -> Result<Vec<u8>, PsError> {
        let key = Key {
            hash: fnv(base),
            len: base.len(),
            cols,
            rule,
            edge,
        };
        self.clock += 1;
        let clock = self.clock;
        if self.entries.get(&key).is_none_or(|e| e.base != base) {
            if self.entries.len() >= CACHE_ENTRIES {
                let oldest = self.entries.iter().min_by_key(|e| e.1.used).map(|e| *e.0);
                if let Some(k) = oldest {
                    self.entries.remove(&k);
                }
            }
            let every = (CHECKPOINT_WORK / base.len().max(1) as u64).max(1);
            self.entries.insert(
                key,
                Entry {
                    base: base.to_vec(),
                    every,
                    checkpoints: Vec::new(),
                    recent: Vec::new(),
                    used: clock,
                },
            );
        }
        let e = self.entries.get_mut(&key).expect("just made");
        e.used = clock;
        // The latest known state at or before n.
        let mut from: (u64, &[u8]) = (0, &e.base);
        for (g, s) in &e.checkpoints {
            if *g <= n && *g > from.0 {
                from = (*g, s);
            }
        }
        for (g, st) in &e.recent {
            if *g <= n && *g >= from.0 {
                from = (*g, st);
            }
        }
        let (mut g, mut state) = (from.0, from.1.to_vec());
        while g < n {
            charge(base.len() as u64)?;
            state = step(&state, cols, rule, edge);
            g += 1;
            if g % e.every == 0 && !e.checkpoints.iter().any(|c| c.0 == g) {
                e.checkpoints.push((g, state.clone()));
                e.checkpoints.sort_by_key(|c| c.0);
                if e.checkpoints.len() > MAX_CHECKPOINTS {
                    e.every *= 2;
                    let every = e.every;
                    e.checkpoints.retain(|c| c.0 % every == 0);
                }
            }
        }
        e.recent.retain(|r| r.0 != n);
        e.recent.push((n, state.clone()));
        if e.recent.len() > RECENT {
            e.recent.remove(0);
        }
        Ok(state)
    }
}

// ---- the words ----------------------------------------------------------------

fn dim(v: &Value, what: &str, ctx: &Shared) -> Result<usize, PsError> {
    match v.as_f64() {
        Some(x) if x >= 1.0 && x.is_finite() => Ok(x as usize),
        _ => fail(
            ctx,
            PsError::Rangecheck,
            format!("{what} must be a whole number of cells, 1 or more"),
        ),
    }
}

fn rgb_of(v: Option<&Value>, default: [f64; 3]) -> [f64; 3] {
    match v.and_then(Value::as_numbers) {
        Some(c) if c.len() == 3 => [c[0], c[1], c[2]],
        Some(c) if c.len() == 1 => [c[0]; 3],
        _ => match v.and_then(Value::as_f64) {
            Some(g) => [g; 3],
            None => default,
        },
    }
}

fn rule_edge(v: &Value, ctx: &Shared, who: &str) -> Result<(Value, Edge), PsError> {
    let (rule, edge) = match v {
        Value::Dict(_) => (
            v.get("Rule").cloned().unwrap_or(Value::Name("life".into())),
            v.get("Edge").cloned(),
        ),
        other => (other.clone(), None),
    };
    let edge = match edge {
        None => Edge::Dead,
        Some(Value::Name(n)) if n == "dead" => Edge::Dead,
        Some(Value::Name(n)) if n == "wrap" => Edge::Wrap,
        Some(Value::Name(n)) if n == "open" => Edge::Open,
        Some(_) => {
            return fail(
                ctx,
                PsError::Rangecheck,
                format!("{who}: /Edge is /dead, /wrap or /open"),
            );
        }
    };
    Ok((rule, edge))
}

/// Count `cells` updates against this moment's cap and the interpreter's
/// budgets.
fn charge(i: &mut Interp, ctx: &Shared, cells: u64) -> Result<(), PsError> {
    {
        let mut c = ctx.borrow_mut();
        c.cell_work += cells as f64;
        if c.cell_work > MAX_WORK {
            c.hint = Some(format!(
                "cellular automata: more than {:.0} million cell updates in one frame. A smaller grid, fewer generations at once (seeking far into a film costs every generation before it), or compute the history in setup",
                MAX_WORK / 1e6
            ));
            return Err(PsError::Limitcheck);
        }
    }
    i.check_limits(cells / CELLS_PER_STEP + 1)
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    // cols rows opts  cellgrid  cells
    reg(interp, ctx, "cellgrid", |i, ctx| {
        let opts = i.pop_value()?;
        let rows = dim(&i.pop_value()?, "cellgrid: rows", ctx)?;
        let cols = dim(&i.pop_value()?, "cellgrid: cols", ctx)?;
        if cols.saturating_mul(rows) > MAX_CELLS {
            return fail(
                ctx,
                PsError::Limitcheck,
                format!("cellgrid: at most {MAX_CELLS} cells (2048 x 2048)"),
            );
        }
        let (pattern, density, seed) = match &opts {
            Value::Dict(_) => (
                opts.get("Pattern")
                    .cloned()
                    .unwrap_or(Value::Name("random".into())),
                opts.get("Density").and_then(Value::as_f64).unwrap_or(0.3),
                opts.get("Seed")
                    .and_then(Value::as_f64)
                    .map_or_else(|| ctx.borrow().seed(), |s| s as i64),
            ),
            other => (other.clone(), 0.3, ctx.borrow().seed()),
        };
        match grid(
            cols,
            rows,
            &Seeding {
                pattern,
                density,
                seed,
            },
        ) {
            Ok(g) => {
                i.push_string(&g);
                Ok(())
            }
            Err(e) => fail(ctx, PsError::Rangecheck, format!("cellgrid: {e}")),
        }
    });

    // cells cols opts n  cellsteps  cells'
    reg(interp, ctx, "cellsteps", |i, ctx| {
        let n = i.pop_value()?;
        let opts = i.pop_value()?;
        let cols = dim(&i.pop_value()?, "cellsteps: cols", ctx)?;
        let cells = i.pop_string()?;
        let n = match n.as_f64() {
            Some(x) if x >= 0.0 && x.is_finite() => x.floor() as u64,
            _ => {
                return fail(
                    ctx,
                    PsError::Rangecheck,
                    "cellsteps: the generation must be 0 or more",
                );
            }
        };
        if cells.len() % cols != 0 || cells.len() > MAX_CELLS {
            return fail(
                ctx,
                PsError::Rangecheck,
                format!(
                    "cellsteps: the grid's length ({}) must be a whole number of rows of {cols}, at most {MAX_CELLS} cells",
                    cells.len()
                ),
            );
        }
        let (rule, edge) = rule_edge(&opts, ctx, "cellsteps")?;
        let rule = match rule.as_text().map(parse_rule) {
            Some(Ok(r)) => r,
            Some(Err(e)) => return fail(ctx, PsError::Rangecheck, format!("cellsteps: {e}")),
            None => {
                return fail(
                    ctx,
                    PsError::Typecheck,
                    "cellsteps: the rule is a name (/life) or text ((B3/S23)); an elementary rule (a number) is for cellrows",
                );
            }
        };
        let mut cache = std::mem::take(&mut ctx.borrow_mut().cells);
        let out = cache.advance(&cells, cols, rule, edge, n, |w| charge(i, ctx, w));
        ctx.borrow_mut().cells = cache;
        i.push_string(&out?);
        Ok(())
    });

    // row rule rows  cellrows  cells
    reg(interp, ctx, "cellrows", |i, ctx| {
        let rows = dim(&i.pop_value()?, "cellrows: rows", ctx)?;
        let opts = i.pop_value()?;
        let row = i.pop_string()?;
        let (rule, edge) = rule_edge(&opts, ctx, "cellrows")?;
        let rule = match rule.as_f64() {
            Some(r) if (0.0..=255.0).contains(&r) && r.fract() == 0.0 => r as u8,
            _ => {
                return fail(
                    ctx,
                    PsError::Rangecheck,
                    "cellrows: the rule is an elementary rule, a whole number 0 to 255 (30, 90, 110)",
                );
            }
        };
        if row.len().saturating_mul(rows) > MAX_CELLS || row.is_empty() {
            return fail(
                ctx,
                PsError::Limitcheck,
                format!(
                    "cellrows: the row must not be empty, and rows x its length at most {MAX_CELLS} cells"
                ),
            );
        }
        charge(i, ctx, (row.len() * rows) as u64)?;
        i.push_string(&elementary(&row, rule, rows, edge));
        Ok(())
    });

    // cells palette frac  cellcolors  rgb
    reg(interp, ctx, "cellcolors", |i, ctx| {
        let frac = i.pop_value()?.as_f64().unwrap_or(0.0).clamp(0.0, 1.0);
        let p = i.pop_value()?;
        let cells = i.pop_string()?;
        if !matches!(p, Value::Dict(_)) {
            return fail(
                ctx,
                PsError::Typecheck,
                "cellcolors: the palette is a dictionary: << /Alive [r g b] /Ground [r g b] ... >>",
            );
        }
        let ground = rgb_of(p.get("Ground"), [0.0; 3]);
        let alive = rgb_of(p.get("Alive"), [1.0; 3]);
        let trail = rgb_of(p.get("Trail"), alive);
        let pal = Palette {
            born: rgb_of(p.get("Born"), alive),
            alive,
            mature: p
                .get("Mature")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                .max(0.0),
            trail,
            fade: p
                .get("Fade")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                .max(0.0),
            ghost: rgb_of(p.get("Ghost"), ground),
            ground,
        };
        charge(i, ctx, cells.len() as u64 / 8)?;
        i.push_string(&colors(&cells, &lut(&pal, frac)));
        Ok(())
    });

    // cells cols share  cellbounds  [x0 y0 x1 y1] | null
    reg(interp, ctx, "cellbounds", |i, ctx| {
        let share = i.pop_value()?.as_f64().unwrap_or(0.0);
        let cols = dim(&i.pop_value()?, "cellbounds: cols", ctx)?;
        let cells = i.pop_string()?;
        if cells.len() % cols != 0 {
            return fail(
                ctx,
                PsError::Rangecheck,
                "cellbounds: the grid's length must be a whole number of rows",
            );
        }
        match bounds(&cells, cols, share) {
            Some(b) => i.push_numbers(&b.map(|v| v as f64)),
            None => i.push_value(&Value::Null)?,
        }
        Ok(())
    });

    // cells  cellcount  n
    reg(interp, ctx, "cellcount", |i, _| {
        let cells = i.pop_string()?;
        i.push_int(cells.iter().filter(|v| **v >= 128).count() as i64);
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn life() -> Rule {
        parse_rule("life").unwrap()
    }

    fn from_picture(rows: &[&str], cols: usize, h: usize, at: (i64, i64)) -> Vec<u8> {
        let shape = picture(&rows.iter().map(|r| r.as_bytes()).collect::<Vec<_>>());
        let mut g = vec![0u8; cols * h];
        for (x, y) in shape.0 {
            g[(at.1 as usize + y) * cols + at.0 as usize + x] = 128;
        }
        g
    }

    fn live(g: &[u8], cols: usize) -> Vec<(usize, usize)> {
        g.iter()
            .enumerate()
            .filter(|(_, v)| **v >= 128)
            .map(|(k, _)| (k % cols, k / cols))
            .collect()
    }

    #[test]
    fn rules_parse() {
        assert_eq!(
            life(),
            Rule::LifeLike {
                born: 1 << 3,
                survive: (1 << 2) | (1 << 3)
            }
        );
        assert_eq!(
            parse_rule("B36/S23").unwrap(),
            parse_rule("highlife").unwrap()
        );
        assert_eq!(parse_rule("brain").unwrap(), Rule::Brain);
        assert!(parse_rule("B9/S2").is_err());
        assert!(parse_rule("conway").is_err());
    }

    #[test]
    fn a_glider_moves_one_cell_diagonally_in_four_generations() {
        let mut g = from_picture(&[".O.", "..O", "OOO"], 12, 12, (2, 2));
        let before = live(&g, 12);
        for _ in 0..4 {
            g = step(&g, 12, life(), Edge::Dead);
        }
        let moved: Vec<_> = before.iter().map(|(x, y)| (x + 1, y + 1)).collect();
        assert_eq!(live(&g, 12), moved);
    }

    #[test]
    fn a_blinker_oscillates_and_a_block_stays() {
        let g0 = from_picture(&["OOO"], 5, 5, (1, 2));
        let g1 = step(&g0, 5, life(), Edge::Dead);
        assert_eq!(live(&g1, 5), vec![(2, 1), (2, 2), (2, 3)]);
        let g2 = step(&g1, 5, life(), Edge::Dead);
        assert_eq!(live(&g2, 5), live(&g0, 5));
        let b = from_picture(&["OO", "OO"], 4, 4, (1, 1));
        assert_eq!(live(&step(&b, 4, life(), Edge::Dead), 4), live(&b, 4));
    }

    #[test]
    fn a_glider_wraps_round_a_torus() {
        let mut g = from_picture(&[".O.", "..O", "OOO"], 6, 6, (0, 0));
        let before = live(&g, 6);
        for _ in 0..24 {
            g = step(&g, 6, life(), Edge::Wrap);
        }
        assert_eq!(live(&g, 6), before, "6 cells of travel on a 6-cell torus");
    }

    #[test]
    fn the_dead_remember_how_long_ago() {
        let g0 = from_picture(&["OOO"], 5, 5, (1, 2));
        let g1 = step(&g0, 5, life(), Edge::Dead);
        assert_eq!(g1[2 * 5 + 1], 127, "the blinker's end died this generation");
        assert_eq!(g1[2 * 5 + 2], 129, "its middle is a generation old");
        let g2 = step(&g1, 5, life(), Edge::Dead);
        assert_eq!(g2[5 + 2], 127);
        assert_eq!(g2[2 * 5 + 1], 128, "reborn");
        // A lone cell dies, and fades to "lived here once", never to never.
        let mut g = from_picture(&["O"], 3, 3, (1, 1));
        for _ in 0..300 {
            g = step(&g, 3, life(), Edge::Dead);
        }
        assert_eq!(g, vec![0, 0, 0, 0, 1, 0, 0, 0, 0]);
    }

    #[test]
    fn rule_30_from_one_cell() {
        let mut row = vec![0u8; 11];
        row[5] = 128;
        let h = elementary(&row, 30, 5, Edge::Dead);
        let text: Vec<String> = h
            .chunks(11)
            .map(|r| r.iter().map(|v| if *v > 0 { '1' } else { '0' }).collect())
            .collect();
        assert_eq!(
            text,
            [
                "00000100000",
                "00001110000",
                "00011001000",
                "00110111100",
                "01100100010"
            ]
        );
    }

    #[test]
    fn brian_rests_one_generation() {
        let g0 = from_picture(&["OO"], 4, 4, (1, 1));
        let g1 = step(&g0, 4, Rule::Brain, Edge::Dead);
        assert_eq!(g1[5], 127);
        assert_eq!(g1[6], 127);
        // cells above and below the pair had exactly two live neighbours
        assert_eq!(live(&g1, 4), vec![(1, 0), (2, 0), (1, 2), (2, 2)]);
    }

    #[test]
    fn bounds_leave_out_a_gliders_track_when_asked() {
        let mut g = from_picture(&["OOOO", "OOOO", "OOOO"], 30, 20, (2, 2));
        for k in 0..12 {
            g[(6 + k) * 30 + 8 + k] = 1; // a thin diagonal track of cells that once lived
        }
        assert_eq!(bounds(&g, 30, 0.0), Some([2, 2, 19, 17]));
        assert_eq!(bounds(&g, 30, 0.4), Some([2, 2, 5, 4]));
        assert_eq!(bounds(&[0u8; 9], 3, 0.2), None);
    }

    #[test]
    fn rle_reads_a_glider() {
        let (cells, w, h) = rle("#N Glider\nx = 3, y = 3, rule = B3/S23\nbo$2bo$3o!").unwrap();
        assert_eq!((w, h), (3, 3));
        assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
    }

    #[test]
    fn the_cache_gives_the_same_answer_in_any_order() {
        let how = Seeding {
            pattern: Value::Name("random".into()),
            density: 0.35,
            seed: 4,
        };
        let base = grid(40, 30, &how).unwrap();
        let mut direct = base.clone();
        let mut want = vec![base.clone()];
        for _ in 0..200 {
            direct = step(&direct, 40, life(), Edge::Wrap);
            want.push(direct.clone());
        }
        let mut c = Cache::default();
        for n in [5u64, 200, 3, 150, 151, 0, 199, 77] {
            let got = c
                .advance(&base, 40, life(), Edge::Wrap, n, |_| Ok(()))
                .unwrap();
            assert_eq!(got, want[n as usize], "generation {n}");
        }
    }
}
