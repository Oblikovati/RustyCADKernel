// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyData.cxx (hideByOneTriangle) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The hiding of a projected segment by one projected triangle.
//!
//! The native member repeats one block for the three triangle edges (V1V2 with the
//! mask EMskGrALin1, V2V3 with EMskGrALin2, V3V1 with EMskGrALin3); the port runs that
//! block once per edge on one state holding the locals the native blocks share.
use super::bi_point::BiPointPoints;
use super::edge_status::EdgeStatus;
use super::poly_data::PolyTriangle;
use super::poly_records::PolyMask;
use rustycad_tkmath::gp::Xy;

/// The locals of hideByOneTriangle that live across the three edge blocks: the side
/// of the last crossing (`n1`), the hidden range `[psta, pend]` of the segment, and
/// up to two crossing points (`p` parameters, `o` entering flags, `m` multiple flags).
struct HideState {
    n1: i32,
    psta: f64,
    pend: f64,
    npi: i32,
    npi_rej: i32,
    o: [bool; 2],
    m: [bool; 2],
    p: [f64; 2],
}

/// hideByOneTriangle: hides into `status` the part of the segment `points` behind the
/// triangle `(v1, v2, v3)` of `triangle`, clipped by the triangle edges. With
/// `crossing` the segment crosses the triangle plane at `triangle.param` and only the
/// part before (`hide_before`) or after it is behind. `tr_flags` chooses, per edge,
/// whether its parameter is measured along X (EMskGrALin*) or along Y.
pub(super) fn hide_by_one_triangle(
    points: &BiPointPoints,
    triangle: &PolyTriangle,
    crossing: bool,
    hide_before: bool,
    tr_flags: i32,
    status: &mut EdgeStatus,
) {
    let mut state = HideState {
        n1: 0,
        psta: 0.0,
        pend: 1.0,
        npi: -1,
        npi_rej: 0,
        o: [false; 2],
        m: [false; 2],
        p: [0.0; 2],
    };
    let edges = [
        (triangle.v1, triangle.v2, PolyMask::EMskGrALin1),
        (triangle.v2, triangle.v3, PolyMask::EMskGrALin2),
        (triangle.v3, triangle.v1, PolyMask::EMskGrALin3),
    ];
    for (start, end, mask) in edges {
        let along_x = tr_flags & mask as i32 != 0;
        if !state.clip_by_edge(points, triangle, start, end, along_x) {
            return;
        }
    }
    if !state.collect_crossings() {
        return;
    }
    if crossing && !state.clip_by_plane(triangle, hide_before) {
        return;
    }
    state.hide_range(triangle.tol_param, status);
}

/// The position of a parameter along an edge of the triangle: 1 before its start, 2
/// at its start, 3 inside, 4 at its end, 5 after it (at the parameter tolerance).
fn edge_zone(pd: f64, tol_param: f64) -> i32 {
    if pd < -tol_param {
        1
    } else if pd < tol_param {
        2
    } else if pd - 1. < -tol_param {
        3
    } else if pd - 1. < tol_param {
        4
    } else {
        5
    }
}

/// The parameter of the segment point `(px, py)` on the edge from `start` along
/// `delta`, measured along X or Y.
fn edge_parameter(px: f64, py: f64, start: Xy, delta: Xy, along_x: bool) -> f64 {
    if along_x {
        (px - start.x()) / delta.x()
    } else {
        (py - start.y()) / delta.y()
    }
}

impl HideState {
    /// One edge block: the signed distances of the segment ends to the edge line.
    /// Returns false where the native block returns (the segment is outside the
    /// triangle on this edge).
    fn clip_by_edge(
        &mut self,
        points: &BiPointPoints,
        triangle: &PolyTriangle,
        start: Xy,
        end: Xy,
        along_x: bool,
    ) -> bool {
        let delta = Xy::new(end.x() - start.x(), end.y() - start.y());
        let inverse = 1. / delta.modulus();
        let normal = Xy::new(-delta.y() * inverse, delta.x() * inverse);
        let dot = normal.dot(start);
        let d1 = normal.dot(points.pnt_p1_2d()) - dot;
        let d2 = normal.dot(points.pnt_p2_2d()) - dot;
        let tolerance = triangle.tolerance;
        let crossing = if d1 > tolerance {
            if d2 < -tolerance {
                self.n1 = 2;
            }
            d2 < -tolerance
        } else if d1 < -tolerance {
            // The native test is `d2 > tolerance`, so a NaN distance returns too.
            if d2 > tolerance {
                self.n1 = -1;
                true
            } else {
                return false;
            }
        } else if d2 > tolerance {
            false
        } else if d2 < -tolerance {
            return false;
        } else {
            return self.clip_on_edge_line(points, triangle, start, delta, along_x);
        };
        if crossing {
            self.add_crossing(points, triangle, start, delta, along_x, [d1, d2]);
        }
        true
    }

    /// The segment lies on the edge line: its parameters on the edge restrict the
    /// hidden range, or show that nothing is hidden (false).
    fn clip_on_edge_line(
        &mut self,
        points: &BiPointPoints,
        triangle: &PolyTriangle,
        start: Xy,
        delta: Xy,
        along_x: bool,
    ) -> bool {
        let (p1, p2) = (points.pnt_p1, points.pnt_p2);
        let pd1 = edge_parameter(p1.x(), p1.y(), start, delta, along_x);
        let pd2 = edge_parameter(p2.x(), p2.y(), start, delta, along_x);
        let nn1 = edge_zone(pd1, triangle.tol_param);
        let nn2 = edge_zone(pd2, triangle.tol_param);
        match nn1 {
            3 => self.clip_end(pd1, pd2, nn2),
            1 => {
                if nn2 <= 2 {
                    return false;
                }
                self.psta = -pd1 / (pd2 - pd1);
                if nn2 == 5 {
                    self.pend = (1. - pd1) / (pd2 - pd1);
                }
            }
            5 => {
                if nn2 >= 4 {
                    return false;
                }
                self.psta = (pd1 - 1.) / (pd1 - pd2);
                if nn2 == 1 {
                    self.pend = pd1 / (pd1 - pd2);
                }
            }
            2 if nn2 == 1 => return false,
            4 if nn2 == 5 => return false,
            _ => self.clip_end(pd1, pd2, nn2),
        }
        true
    }

    /// The end of the hidden range when the segment starts inside, at the start or at
    /// the end of the edge: where it leaves the edge before its start or after its end.
    fn clip_end(&mut self, pd1: f64, pd2: f64, nn2: i32) {
        if nn2 == 1 {
            self.pend = pd1 / (pd1 - pd2);
        } else if nn2 == 5 {
            self.pend = (1. - pd1) / (pd2 - pd1);
        }
    }

    /// The segment crosses the edge line: its crossing point is kept, merged with an
    /// earlier one at a triangle vertex, or rejected when outside the edge.
    fn add_crossing(
        &mut self,
        points: &BiPointPoints,
        triangle: &PolyTriangle,
        start: Xy,
        delta: Xy,
        along_x: bool,
        [d1, d2]: [f64; 2],
    ) {
        let ad1 = if d1 < 0. { -d1 } else { d1 };
        let ad2 = if d2 < 0. { -d2 } else { d2 };
        let pp = ad1 / (ad1 + ad2);
        let (p1, p2) = (points.pnt_p1, points.pnt_p2);
        let pdp = if along_x {
            (p1.x() + (p2.x() - p1.x()) * pp - start.x()) / delta.x()
        } else {
            (p1.y() + (p2.y() - p1.y()) * pp - start.y()) / delta.y()
        };
        let tol_param = triangle.tol_param;
        // The native chain, kept as written so that a NaN parameter is outside.
        let at_vertex = if pdp < -tol_param {
            None
        } else if pdp < tol_param {
            Some(true)
        } else if pdp - 1. < -tol_param {
            Some(false)
        } else if pdp - 1. < tol_param {
            Some(true)
        } else {
            None
        };
        let (outside, multiple) = match at_vertex {
            None => (true, false),
            Some(false) => (false, false),
            Some(true) => (self.reject_multiples(), true),
        };
        self.keep_crossing(outside, pp, multiple);
    }

    /// The loop of the native code over the kept crossings when the new one is at a
    /// triangle vertex: a kept crossing at a vertex makes the new one outside, and is
    /// removed too when it runs the other way. The loop bound `npi` shrinks as it
    /// removes, as natively.
    fn reject_multiples(&mut self) -> bool {
        let mut outside = false;
        let mut l = 0;
        while l <= self.npi {
            let Some(&multiple) = self.m.get(l as usize) else {
                break;
            };
            if multiple {
                outside = true;
                if self.o[l as usize] != (self.n1 == -1) {
                    if l == 0 && self.npi == 1 {
                        self.p[0] = self.p[1];
                        self.o[0] = self.o[1];
                        self.m[0] = self.m[1];
                    }
                    self.npi -= 1;
                    self.npi_rej += 1;
                }
            }
            l += 1;
        }
        outside
    }

    /// Counts a rejected crossing, or keeps the crossing (at most two are stored; a
    /// third only counts, as natively).
    fn keep_crossing(&mut self, outside: bool, pp: f64, multiple: bool) {
        if outside {
            self.npi_rej += 1;
            return;
        }
        self.npi += 1;
        if let Ok(k) = usize::try_from(self.npi)
            && k < 2
        {
            self.p[k] = pp;
            self.o[k] = self.n1 == -1;
            self.m[k] = multiple;
        }
    }

    /// The hidden range from the kept crossings: none (nothing hidden when two or more
    /// were rejected), one (from it to the end the segment enters by, or from the
    /// start), or two (between them).
    fn collect_crossings(&mut self) -> bool {
        match self.npi {
            -1 => return self.npi_rej < 2,
            0 if self.o[0] => (self.psta, self.pend) = (self.p[0], 1.),
            0 => (self.psta, self.pend) = (0., self.p[0]),
            1 if self.p[0] > self.p[1] => (self.psta, self.pend) = (self.p[1], self.p[0]),
            1 => (self.psta, self.pend) = (self.p[0], self.p[1]),
            _ => {}
        }
        true
    }

    /// A segment crossing the triangle plane is hidden only before or after the
    /// crossing parameter; false when that leaves nothing hidden.
    fn clip_by_plane(&mut self, triangle: &PolyTriangle, hide_before: bool) -> bool {
        let (param, tol_param) = (triangle.param, triangle.tol_param);
        if hide_before {
            if param - self.psta < tol_param {
                return false;
            }
            if param < self.pend {
                self.pend = param;
            }
        } else {
            if self.pend - param < tol_param {
                return false;
            }
            if self.psta < param {
                self.psta = param;
            }
        }
        true
    }

    /// Hides `[psta, pend]` in `status`, or the whole edge when the range covers
    /// `[0, 1]` at the parameter tolerance.
    fn hide_range(&self, tol_param: f64, status: &mut EdgeStatus) {
        let near = |value: f64| {
            if value > 0. {
                value < tol_param
            } else {
                value > -tol_param
            }
        };
        if near(self.psta) && near(self.pend - 1.) {
            status.hide_all();
            return;
        }
        let tol = tol_param as f32;
        status.hide(self.psta, tol, self.pend, tol, false, false);
    }
}
