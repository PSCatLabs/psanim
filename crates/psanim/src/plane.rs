//! Perspective: a flat drawing laid on a plane in 3D and seen through a
//! pinhole camera.
//!
//! The PostScript CTM is affine and cannot foreshorten, so `onplane` and
//! `projectpath` take the path out of the interpreter, move every point
//! through [`Plane::project`] and put the result back. A projective map
//! sends straight lines to straight lines, so straight edges keep their two
//! ends; only curves are cut into chords, as many as their size on the
//! screen needs, and only edges that cross the near plane are split there.
//! Everything here is pure data in, data out, in the caller's user space.

use pscat::host::PathSegment;

/// Where the plane is and where the eye is, all in the user space of the
/// caller: the plane's own coordinates are that space's x and y.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    /// Degrees the plane is tipped back about the x axis through its
    /// origin: 0 faces the eye, 90 is edge-on. Positive sends +y away.
    pub tilt: f64,
    /// Degrees it is then turned about the vertical axis through its
    /// origin. Positive sends +x away.
    pub turn: f64,
    /// How far behind the screen the plane's origin is pushed.
    pub depth: f64,
    /// The point on the screen the eye is straight in front of: where
    /// lines running away from the viewer meet when tilt is 90.
    pub eye: (f64, f64),
    /// How far the eye is in front of the screen.
    pub distance: f64,
    /// Depth behind the screen at which ink starts to fade, and where it
    /// is gone.
    pub fade: Option<(f64, f64)>,
}

impl Plane {
    pub fn flat(distance: f64) -> Plane {
        Plane {
            tilt: 0.0,
            turn: 0.0,
            depth: 0.0,
            eye: (0.0, 0.0),
            distance,
            fade: None,
        }
    }

    /// A point of the plane in 3D: x right, y up, z away from the eye,
    /// with the screen at z = 0.
    pub fn place(&self, u: f64, v: f64) -> [f64; 3] {
        let (st, ct) = self.tilt.to_radians().sin_cos();
        let (sa, ca) = self.turn.to_radians().sin_cos();
        let (x, y, z) = (u, v * ct, v * st);
        [x * ca + z * sa, y, -x * sa + z * ca + self.depth]
    }

    /// The nearest depth a point may have and still be drawn: a little in
    /// front of the eye, so nothing is magnified more than 20 times.
    pub fn near(&self) -> f64 {
        -self.distance * 0.95
    }

    /// A 3D point on the screen. Only for points in front of [`near`].
    pub fn screen(&self, p: [f64; 3]) -> (f64, f64) {
        let s = self.distance / (self.distance + p[2]);
        (
            self.eye.0 + (p[0] - self.eye.0) * s,
            self.eye.1 + (p[1] - self.eye.1) * s,
        )
    }

    /// A point of the plane on the screen, or `None` when it is behind
    /// the near plane.
    pub fn project(&self, u: f64, v: f64) -> Option<(f64, f64)> {
        let p = self.place(u, v);
        (p[2] >= self.near()).then(|| self.screen(p))
    }

    /// The opacity ink at depth `z` keeps: 1 before the fade, 0 past it,
    /// a smooth step between.
    pub fn opacity(&self, z: f64) -> f64 {
        match self.fade {
            None => 1.0,
            Some((a, b)) if b <= a => {
                if z < a {
                    1.0
                } else {
                    0.0
                }
            }
            Some((a, b)) => {
                let u = ((z - a) / (b - a)).clamp(0.0, 1.0);
                1.0 - u * u * (3.0 - 2.0 * u)
            }
        }
    }
}

/// A subpath in 3D, and whether it was closed.
struct Sub {
    pts: Vec<[f64; 3]>,
    closed: bool,
}

fn lerp3(a: [f64; 3], b: [f64; 3], u: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * u,
        a[1] + (b[1] - a[1]) * u,
        a[2] + (b[2] - a[2]) * u,
    ]
}

/// The path's subpaths placed in 3D, curves cut into chords about `tol`
/// long on the screen.
fn lift(plane: &Plane, path: &[PathSegment], tol: f64) -> Vec<Sub> {
    let near = plane.near();
    let mut subs: Vec<Sub> = Vec::new();
    let mut cur: Option<Sub> = None;
    let mut last = (0.0, 0.0);
    let mut start = (0.0, 0.0);
    for seg in path {
        match *seg {
            PathSegment::MoveTo(x, y) => {
                subs.extend(cur.take());
                start = (x, y);
                last = (x, y);
                cur = Some(Sub {
                    pts: vec![plane.place(x, y)],
                    closed: false,
                });
            }
            PathSegment::LineTo(x, y) => {
                let s = cur.get_or_insert_with(|| Sub {
                    pts: vec![plane.place(last.0, last.1)],
                    closed: false,
                });
                s.pts.push(plane.place(x, y));
                last = (x, y);
            }
            PathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => {
                let s = cur.get_or_insert_with(|| Sub {
                    pts: vec![plane.place(last.0, last.1)],
                    closed: false,
                });
                let ctrl = [last, (x1, y1), (x2, y2), (x3, y3)];
                let lifted = ctrl.map(|(u, v)| plane.place(u, v));
                // Size on the screen of the control polygon, which bounds
                // the curve's; behind the near plane, use the plane's own
                // size, finely.
                let screen_len = if lifted.iter().all(|p| p[2] >= near) {
                    let q = lifted.map(|p| plane.screen(p));
                    q.windows(2)
                        .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
                        .sum::<f64>()
                } else {
                    ctrl.windows(2)
                        .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
                        .sum::<f64>()
                        * 20.0
                };
                let n = ((screen_len / tol).ceil() as usize).clamp(2, 200);
                let p0 = last;
                for k in 1..=n {
                    let t = k as f64 / n as f64;
                    let mt = 1.0 - t;
                    let (a, b, c, d) =
                        (mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t);
                    s.pts.push(plane.place(
                        a * p0.0 + b * x1 + c * x2 + d * x3,
                        a * p0.1 + b * y1 + c * y2 + d * y3,
                    ));
                }
                last = (x3, y3);
            }
            PathSegment::Close => {
                if let Some(mut s) = cur.take() {
                    s.closed = true;
                    subs.push(s);
                }
                last = start;
            }
        }
    }
    subs.extend(cur);
    subs.retain(|s| s.pts.len() > 1);
    subs
}

/// A closed polygon cut to the part in front of the near plane
/// (Sutherland-Hodgman against one plane: the cut runs along the near
/// plane, so the winding of what is left is kept).
fn clip_closed(pts: &[[f64; 3]], near: f64) -> Vec<[f64; 3]> {
    let n = pts.len();
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let (ina, inb) = (a[2] >= near, b[2] >= near);
        if ina {
            out.push(a);
        }
        if ina != inb {
            out.push(lerp3(a, b, (near - a[2]) / (b[2] - a[2])));
        }
    }
    out
}

/// An open polyline cut into the runs in front of the near plane.
fn clip_open(pts: &[[f64; 3]], near: f64) -> Vec<Vec<[f64; 3]>> {
    let mut runs = Vec::new();
    let mut run: Vec<[f64; 3]> = Vec::new();
    for (i, &b) in pts.iter().enumerate() {
        let inb = b[2] >= near;
        if i > 0 {
            let a = pts[i - 1];
            let ina = a[2] >= near;
            if ina != inb {
                let c = lerp3(a, b, (near - a[2]) / (b[2] - a[2]));
                run.push(c);
                if !inb {
                    runs.push(std::mem::take(&mut run));
                }
            }
        }
        if inb {
            run.push(b);
        }
    }
    runs.push(run);
    runs.retain(|r| r.len() > 1);
    runs
}

/// What [`project_path`] made.
#[derive(Debug, Default)]
pub struct Projected {
    pub path: Vec<PathSegment>,
    /// The nearest and farthest depth of what was kept, if anything was.
    pub depth: Option<(f64, f64)>,
}

impl Projected {
    /// The opacity the plane's fade gives this path: one value for the
    /// whole of it, from the depth of its middle. Zero when nothing is
    /// left in front of the eye.
    pub fn opacity(&self, plane: &Plane) -> f64 {
        self.depth
            .map_or(0.0, |(a, b)| plane.opacity((a + b) / 2.0))
    }
}

/// The path seen on the plane. `tol` is the chord length curves are cut
/// into, on the screen. With `as_fill`, every subpath is treated as closed
/// (as `fill` treats it) so cutting it at the near plane keeps its area;
/// otherwise an open subpath stays open and is cut into pieces.
pub fn project_path(plane: &Plane, path: &[PathSegment], tol: f64, as_fill: bool) -> Projected {
    let tol = if tol.is_finite() && tol > 0.0 {
        tol
    } else {
        1.0
    };
    let near = plane.near();
    let mut out = Projected::default();
    let emit = |pts: &[[f64; 3]], close: bool, out: &mut Projected| {
        if pts.len() < 2 {
            return;
        }
        for (k, p) in pts.iter().enumerate() {
            let (lo, hi) = out.depth.unwrap_or((p[2], p[2]));
            out.depth = Some((lo.min(p[2]), hi.max(p[2])));
            let (x, y) = plane.screen(*p);
            out.path.push(if k == 0 {
                PathSegment::MoveTo(x, y)
            } else {
                PathSegment::LineTo(x, y)
            });
        }
        if close {
            out.path.push(PathSegment::Close);
        }
    };
    for sub in lift(plane, path, tol) {
        if sub.closed || as_fill {
            let kept = if sub.pts.iter().all(|p| p[2] >= near) {
                sub.pts
            } else {
                clip_closed(&sub.pts, near)
            };
            emit(&kept, true, &mut out);
        } else {
            for run in clip_open(&sub.pts, near) {
                emit(&run, false, &mut out);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use PathSegment::*;

    fn square(s: f64) -> Vec<PathSegment> {
        vec![
            MoveTo(-s, -s),
            LineTo(s, -s),
            LineTo(s, s),
            LineTo(-s, s),
            Close,
        ]
    }

    fn points(path: &[PathSegment]) -> Vec<(f64, f64)> {
        path.iter()
            .filter_map(|s| match *s {
                MoveTo(x, y) | LineTo(x, y) => Some((x, y)),
                _ => None,
            })
            .collect()
    }

    /// Twice the signed area of each closed subpath.
    fn areas(path: &[PathSegment]) -> Vec<f64> {
        let mut out = Vec::new();
        let mut cur: Vec<(f64, f64)> = Vec::new();
        for s in path {
            match *s {
                MoveTo(x, y) => cur = vec![(x, y)],
                LineTo(x, y) => cur.push((x, y)),
                Close => {
                    let n = cur.len();
                    out.push(
                        (0..n)
                            .map(|i| {
                                let (a, b) = (cur[i], cur[(i + 1) % n]);
                                a.0 * b.1 - b.0 * a.1
                            })
                            .sum(),
                    );
                }
                _ => {}
            }
        }
        out
    }

    #[test]
    fn facing_plane_at_the_screen_is_the_identity() {
        let p = Plane::flat(1000.0);
        let out = project_path(&p, &square(50.0), 1.0, true);
        assert_eq!(
            points(&out.path),
            vec![(-50.0, -50.0), (50.0, -50.0), (50.0, 50.0), (-50.0, 50.0)]
        );
        assert_eq!(out.opacity(&p), 1.0);
    }

    #[test]
    fn pushed_back_plane_is_scaled_about_the_eye() {
        let p = Plane {
            depth: 1000.0,
            eye: (10.0, 0.0),
            ..Plane::flat(1000.0)
        };
        // Twice as far from the eye: half the size, about the eye point.
        assert_eq!(p.project(110.0, 40.0), Some((60.0, 20.0)));
    }

    #[test]
    fn tilted_plane_converges_on_the_horizon() {
        let p = Plane {
            tilt: 60.0,
            ..Plane::flat(1000.0)
        };
        // The line v = const is level and shrinks with distance.
        let (x0, y0) = p.project(-100.0, 0.0).unwrap();
        let (x1, y1) = p.project(100.0, 0.0).unwrap();
        assert!((y0 - y1).abs() < 1e-9 && (x1 - x0 - 200.0).abs() < 1e-9);
        let (xa, ya) = p.project(100.0, 500.0).unwrap();
        let (xb, yb) = p.project(100.0, 5000.0).unwrap();
        assert!(xb < xa && xa < 100.0);
        // Far along the plane, points rise towards distance * cot(tilt).
        let horizon = 1000.0 / 60f64.to_radians().tan();
        assert!(ya < yb && yb < horizon);
        let (_, yfar) = p.project(0.0, 1e9).unwrap();
        assert!((yfar - horizon).abs() < 1e-3, "{yfar} {horizon}");
    }

    #[test]
    fn straight_edges_stay_straight() {
        let p = Plane {
            tilt: 70.0,
            turn: 20.0,
            depth: 100.0,
            eye: (30.0, 40.0),
            ..Plane::flat(800.0)
        };
        // Points along a line on the plane are collinear on the screen.
        let a = p.project(-200.0, 50.0).unwrap();
        let b = p.project(300.0, 400.0).unwrap();
        for k in 1..10 {
            let u = k as f64 / 10.0;
            let c = p.project(-200.0 + 500.0 * u, 50.0 + 350.0 * u).unwrap();
            let cross = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
            assert!(cross.abs() < 1e-6, "{cross}");
        }
        // And a straight path keeps its point count: no needless chords.
        let out = project_path(&p, &square(100.0), 1.0, true);
        assert_eq!(points(&out.path).len(), 4);
    }

    #[test]
    fn curves_are_cut_by_their_size_on_screen() {
        let k = 55.228_474_983;
        let circle = |r: f64| {
            let c = k * r / 100.0;
            vec![
                MoveTo(r, 0.0),
                CurveTo(r, c, c, r, 0.0, r),
                CurveTo(-c, r, -r, c, -r, 0.0),
                CurveTo(-r, -c, -c, -r, 0.0, -r),
                CurveTo(c, -r, r, -c, r, 0.0),
                Close,
            ]
        };
        let near = Plane::flat(1000.0);
        let far = Plane {
            depth: 9000.0,
            ..near
        };
        let n_near = points(&project_path(&near, &circle(100.0), 1.0, true).path).len();
        let n_far = points(&project_path(&far, &circle(100.0), 1.0, true).path).len();
        assert!(n_near > 4 * n_far, "{n_near} {n_far}");
    }

    #[test]
    fn the_part_behind_the_eye_is_cut_away() {
        // Tipped forward 90 degrees: -y comes towards the eye.
        let p = Plane {
            tilt: 90.0,
            ..Plane::flat(100.0)
        };
        let big = square(1000.0);
        let out = project_path(&p, &big, 1.0, true);
        let near = p.near();
        assert!(out.depth.unwrap().0 >= near - 1e-9);
        // One closed polygon, still wound the same way.
        assert_eq!(areas(&out.path).len(), 1);
        let ours = project_path(&p, &square(10.0), 1.0, true);
        assert_eq!(areas(&out.path)[0].signum(), areas(&ours.path)[0].signum());
        // Wholly behind: nothing, and nothing to see.
        let behind = vec![
            MoveTo(-10.0, -500.0),
            LineTo(10.0, -500.0),
            LineTo(10.0, -400.0),
            Close,
        ];
        let gone = project_path(&p, &behind, 1.0, true);
        assert!(gone.path.is_empty());
        assert_eq!(gone.opacity(&p), 0.0);
    }

    #[test]
    fn an_open_line_through_the_near_plane_is_split() {
        let p = Plane {
            tilt: 90.0,
            ..Plane::flat(100.0)
        };
        // Comes towards the eye, passes behind it, and back out to the side.
        let line = vec![MoveTo(0.0, 50.0), LineTo(0.0, -500.0), LineTo(40.0, 50.0)];
        let out = project_path(&p, &line, 1.0, false);
        let moves = out.path.iter().filter(|s| matches!(s, MoveTo(..))).count();
        assert_eq!(moves, 2);
        assert!(!out.path.contains(&Close));
    }

    #[test]
    fn counters_keep_their_winding() {
        // An outer ring and an inner hole wound the other way, as a glyph's
        // "o" is: both keep their signs on a tilted, turned plane.
        let mut ring = square(100.0);
        ring.extend([
            MoveTo(-50.0, -50.0),
            LineTo(-50.0, 50.0),
            LineTo(50.0, 50.0),
            LineTo(50.0, -50.0),
            Close,
        ]);
        let flat = areas(&ring);
        let p = Plane {
            tilt: 65.0,
            turn: -15.0,
            ..Plane::flat(600.0)
        };
        let seen = areas(&project_path(&p, &ring, 1.0, true).path);
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].signum(), flat[0].signum());
        assert_eq!(seen[1].signum(), flat[1].signum());
    }

    #[test]
    fn fade_is_a_smooth_step_in_depth() {
        let p = Plane {
            fade: Some((100.0, 300.0)),
            ..Plane::flat(1000.0)
        };
        assert_eq!(p.opacity(50.0), 1.0);
        assert_eq!(p.opacity(200.0), 0.5);
        assert_eq!(p.opacity(400.0), 0.0);
    }
}
