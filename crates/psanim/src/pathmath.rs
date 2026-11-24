//! Arc-length geometry on paths: flattening, measuring, sampling, trimming.
//!
//! Everything here is pure data in, data out, on [`pscat::host::PathSegment`]
//! lists in user space. The operators in [`crate::ops`] read the current path
//! from the interpreter, call into this module, and write the result back.

use pscat::host::PathSegment;

/// One subpath as a polyline: its points, and whether `closepath` ended it
/// (the closing edge back to the first point is then part of its length).
#[derive(Clone, Debug, PartialEq)]
pub struct Poly {
    pub points: Vec<(f64, f64)>,
    pub closed: bool,
}

impl Poly {
    /// The edges, including the closing one.
    fn edges(&self) -> impl Iterator<Item = ((f64, f64), (f64, f64))> + '_ {
        let n = self.points.len();
        let count = if self.closed && n > 1 { n } else { n.saturating_sub(1) };
        (0..count).map(move |i| (self.points[i], self.points[(i + 1) % n]))
    }

    pub fn length(&self) -> f64 {
        self.edges().map(|(a, b)| dist(a, b)).sum()
    }
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (b.0 - a.0).hypot(b.1 - a.1)
}

fn lerp(a: (f64, f64), b: (f64, f64), u: f64) -> (f64, f64) {
    (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u)
}

/// Flatten a path into polylines. `tolerance` is the chord length, in user
/// units, a curve is cut into (each curve gets between 4 and 200 chords).
pub fn flatten(path: &[PathSegment], tolerance: f64) -> Vec<Poly> {
    let tol = if tolerance.is_finite() && tolerance > 0.0 { tolerance } else { 1.0 };
    let mut out: Vec<Poly> = Vec::new();
    let mut cur: Option<Poly> = None;
    let mut start = (0.0, 0.0);
    for seg in path {
        match *seg {
            PathSegment::MoveTo(x, y) => {
                out.extend(cur.take().filter(|p| p.points.len() > 1));
                start = (x, y);
                cur = Some(Poly { points: vec![(x, y)], closed: false });
            }
            PathSegment::LineTo(x, y) => {
                if let Some(p) = cur.as_mut() {
                    p.points.push((x, y));
                }
            }
            PathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => {
                if let Some(p) = cur.as_mut() {
                    let p0 = *p.points.last().unwrap_or(&start);
                    let ctrl = dist(p0, (x1, y1)) + dist((x1, y1), (x2, y2)) + dist((x2, y2), (x3, y3));
                    let n = ((ctrl / tol).ceil() as usize).clamp(4, 200);
                    for i in 1..=n {
                        let t = i as f64 / n as f64;
                        let mt = 1.0 - t;
                        let (a, b, c, d) = (mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t);
                        p.points.push((
                            a * p0.0 + b * x1 + c * x2 + d * x3,
                            a * p0.1 + b * y1 + c * y2 + d * y3,
                        ));
                    }
                }
            }
            PathSegment::Close => {
                if let Some(mut p) = cur.take() {
                    // A closing edge of zero length adds nothing.
                    if p.points.len() > 1 && dist(*p.points.last().unwrap(), p.points[0]) < 1e-9 {
                        p.points.pop();
                    }
                    p.closed = true;
                    if p.points.len() > 1 {
                        out.push(p);
                    }
                    // PostScript leaves the current point at the subpath's
                    // start after closepath; a lineto continues from there.
                    cur = Some(Poly { points: vec![start], closed: false });
                }
            }
        }
    }
    out.extend(cur.filter(|p| p.points.len() > 1));
    out
}

pub fn total_length(polys: &[Poly]) -> f64 {
    polys.iter().map(Poly::length).sum()
}

/// The point at arc length `s` along the polylines taken one after another,
/// and the direction of travel there in degrees (PostScript's convention:
/// counterclockwise from +x). `s` is clamped to the path. `None` for an
/// empty path.
pub fn point_at(polys: &[Poly], s: f64) -> Option<(f64, f64, f64)> {
    let total = total_length(polys);
    let mut left = if s.is_nan() { 0.0 } else { s.clamp(0.0, total) };
    let mut last: Option<((f64, f64), (f64, f64))> = None;
    for poly in polys {
        for (a, b) in poly.edges() {
            let len = dist(a, b);
            if len <= 0.0 {
                continue;
            }
            last = Some((a, b));
            if left <= len {
                let p = lerp(a, b, left / len);
                return Some((p.0, p.1, (b.1 - a.1).atan2(b.0 - a.0).to_degrees()));
            }
            left -= len;
        }
    }
    // Rounding left a sliver past the last edge: its end point.
    last.map(|(a, b)| (b.0, b.1, (b.1 - a.1).atan2(b.0 - a.0).to_degrees()))
        .or_else(|| polys.first().and_then(|p| p.points.first()).map(|p| (p.0, p.1, 0.0)))
}

/// The part of one polyline between arc lengths `s0` and `s1`.
fn trim_poly(poly: &Poly, s0: f64, s1: f64, out: &mut Vec<PathSegment>) {
    let len = poly.length();
    if s1 <= s0 || len <= 0.0 {
        return;
    }
    if s0 <= 0.0 && s1 >= len {
        let mut pts = poly.points.iter();
        if let Some(p) = pts.next() {
            out.push(PathSegment::MoveTo(p.0, p.1));
        }
        out.extend(pts.map(|p| PathSegment::LineTo(p.0, p.1)));
        if poly.closed {
            out.push(PathSegment::Close);
        }
        return;
    }
    let mut pos = 0.0;
    let mut started = false;
    for (a, b) in poly.edges() {
        let l = dist(a, b);
        let (e0, e1) = (pos, pos + l);
        pos = e1;
        if l <= 0.0 || e1 <= s0 {
            continue;
        }
        if e0 >= s1 {
            break;
        }
        if !started {
            let p = lerp(a, b, ((s0 - e0) / l).clamp(0.0, 1.0));
            out.push(PathSegment::MoveTo(p.0, p.1));
            started = true;
        }
        let p = lerp(a, b, ((s1 - e0) / l).clamp(0.0, 1.0));
        out.push(PathSegment::LineTo(p.0, p.1));
    }
}

/// The part of the path between fractions `u0` and `u1` of its total
/// length, the subpaths taken one after another: at `u1 = 0.5` the first
/// half of the ink is there, whichever subpaths that covers.
pub fn trim(polys: &[Poly], u0: f64, u1: f64) -> Vec<PathSegment> {
    let total = total_length(polys);
    let (s0, s1) = (u0.clamp(0.0, 1.0) * total, u1.clamp(0.0, 1.0) * total);
    let mut out = Vec::new();
    let mut pos = 0.0;
    for poly in polys {
        let len = poly.length();
        trim_poly(poly, s0 - pos, (s1 - pos).min(len + if u1 >= 1.0 { 1.0 } else { 0.0 }), &mut out);
        pos += len;
    }
    out
}

/// Each subpath trimmed to the same fractions of its own length, so all of
/// them grow together.
pub fn trim_each(polys: &[Poly], u0: f64, u1: f64) -> Vec<PathSegment> {
    let mut out = Vec::new();
    for poly in polys {
        let len = poly.length();
        let end = if u1 >= 1.0 { len + 1.0 } else { u1.clamp(0.0, 1.0) * len };
        trim_poly(poly, u0.clamp(0.0, 1.0) * len, end, &mut out);
    }
    out
}

/// `n` points evenly spaced by arc length along the whole path, as a flat
/// `[x0, y0, x1, y1, ...]`. An open path's samples include both ends; a
/// path whose subpaths are all closed is sampled as a loop (no repeated end
/// point).
pub fn resample(polys: &[Poly], n: usize) -> Vec<f64> {
    let total = total_length(polys);
    let looped = !polys.is_empty() && polys.iter().all(|p| p.closed);
    let mut out = Vec::with_capacity(n * 2);
    for i in 0..n {
        let u = if looped {
            i as f64 / n as f64
        } else if n > 1 {
            i as f64 / (n - 1) as f64
        } else {
            0.0
        };
        if let Some((x, y, _)) = point_at(polys, u * total) {
            out.extend([x, y]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use PathSegment::*;

    fn square() -> Vec<PathSegment> {
        vec![MoveTo(0.0, 0.0), LineTo(10.0, 0.0), LineTo(10.0, 10.0), LineTo(0.0, 10.0), Close]
    }

    #[test]
    fn closed_square_measures_its_closing_edge() {
        let polys = flatten(&square(), 1.0);
        assert_eq!(polys.len(), 1);
        assert!(polys[0].closed);
        assert_eq!(total_length(&polys), 40.0);
    }

    #[test]
    fn circle_length_is_close_to_2_pi_r() {
        // Four Bezier quadrants of a radius-100 circle.
        let k = 55.228_474_983;
        let path = vec![
            MoveTo(100.0, 0.0),
            CurveTo(100.0, k, k, 100.0, 0.0, 100.0),
            CurveTo(-k, 100.0, -100.0, k, -100.0, 0.0),
            CurveTo(-100.0, -k, -k, -100.0, 0.0, -100.0),
            CurveTo(k, -100.0, 100.0, -k, 100.0, 0.0),
            Close,
        ];
        let len = total_length(&flatten(&path, 1.0));
        assert!((len - 200.0 * std::f64::consts::PI).abs() < 0.5, "{len}");
    }

    #[test]
    fn point_at_walks_edges_and_reports_heading() {
        let polys = flatten(&square(), 1.0);
        assert_eq!(point_at(&polys, 5.0), Some((5.0, 0.0, 0.0)));
        assert_eq!(point_at(&polys, 15.0), Some((10.0, 5.0, 90.0)));
        let (x, y, a) = point_at(&polys, 35.0).unwrap();
        assert_eq!((x, y, a), (0.0, 5.0, -90.0));
        // Clamped at both ends.
        assert_eq!(point_at(&polys, -4.0).unwrap().0, 0.0);
        let end = point_at(&polys, 1e9).unwrap();
        assert!(end.0.abs() < 1e-9 && end.1.abs() < 1e-9);
        assert_eq!(point_at(&[], 1.0), None);
    }

    #[test]
    fn trim_takes_the_middle() {
        let polys = flatten(&square(), 1.0);
        let t = trim(&polys, 0.125, 0.375);
        assert_eq!(t, vec![MoveTo(5.0, 0.0), LineTo(10.0, 0.0), LineTo(10.0, 5.0)]);
        assert!(trim(&polys, 0.5, 0.5).is_empty());
        assert!(trim(&polys, 0.7, 0.2).is_empty());
    }

    #[test]
    fn full_trim_keeps_the_path_closed() {
        let polys = flatten(&square(), 1.0);
        let t = trim(&polys, 0.0, 1.0);
        assert_eq!(t.last(), Some(&Close));
        assert_eq!(t.len(), 5);
    }

    #[test]
    fn trim_runs_through_subpaths_in_order() {
        let two = vec![MoveTo(0.0, 0.0), LineTo(10.0, 0.0), MoveTo(0.0, 5.0), LineTo(10.0, 5.0)];
        let polys = flatten(&two, 1.0);
        assert_eq!(polys.len(), 2);
        // 75% of the ink: all of the first line, half of the second.
        let t = trim(&polys, 0.0, 0.75);
        assert_eq!(t, vec![MoveTo(0.0, 0.0), LineTo(10.0, 0.0), MoveTo(0.0, 5.0), LineTo(5.0, 5.0)]);
        // trim_each grows both together.
        let t = trim_each(&polys, 0.0, 0.5);
        assert_eq!(t, vec![MoveTo(0.0, 0.0), LineTo(5.0, 0.0), MoveTo(0.0, 5.0), LineTo(5.0, 5.0)]);
    }

    #[test]
    fn trimmed_length_is_proportional() {
        let polys = flatten(&square(), 1.0);
        for (u0, u1) in [(0.0, 0.3), (0.2, 0.9), (0.45, 0.55), (0.0, 1.0)] {
            let t = trim(&polys, u0, u1);
            let len = total_length(&flatten(&t, 1.0));
            assert!((len - (u1 - u0) * 40.0).abs() < 1e-9, "{u0} {u1} {len}");
        }
    }

    #[test]
    fn resample_open_and_closed() {
        let line = flatten(&[MoveTo(0.0, 0.0), LineTo(10.0, 0.0)], 1.0);
        assert_eq!(resample(&line, 3), vec![0.0, 0.0, 5.0, 0.0, 10.0, 0.0]);
        let sq = flatten(&square(), 1.0);
        assert_eq!(resample(&sq, 4), vec![0.0, 0.0, 10.0, 0.0, 10.0, 10.0, 0.0, 10.0]);
        assert!(resample(&[], 4).is_empty());
    }

    #[test]
    fn lineto_after_close_starts_from_the_subpath_start() {
        let path = vec![MoveTo(0.0, 0.0), LineTo(4.0, 0.0), LineTo(4.0, 3.0), Close, LineTo(0.0, -5.0)];
        let polys = flatten(&path, 1.0);
        assert_eq!(polys.len(), 2);
        assert_eq!(polys[1].points, vec![(0.0, 0.0), (0.0, -5.0)]);
        assert_eq!(total_length(&polys), 12.0 + 5.0);
    }
}
