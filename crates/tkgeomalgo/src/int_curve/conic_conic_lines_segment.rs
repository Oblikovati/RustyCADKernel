// Rust translation of IntCurve_IntConicConic_1.cxx (Perform for two lines, the segment of
// nearly parallel crossing lines) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The segment case of two crossing lines: the zone closer than the tolerance on
//! both domains, returned as a segment when an end of it is a domain end (segment
//! convention: extremity to extremity, or extremity to intersection), as a point
//! otherwise.
use super::IntConicConic;
use super::conic_conic_line_tools::segment_to_point;
use super::conic_conic_lines::{Lines, position_ll};
use crate::int_res2d::{IntersectionPoint, IntersectionSegment, Position, Transition};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Point2};

/// The zone on L2 clipped by Domain2, and the crossing parameters.
pub(super) struct Zone {
    pub(super) u: (f64, f64),
    pub(super) u1m_u2: f64,
    pub(super) u1p_u2: f64,
    pub(super) res2: (f64, f64),
    pub(super) pos2: (Position, Position),
}

/// The bounds of the segment on both lines with their positions: `a` is the start
/// (Res1inf, Res2inf, Pos1a, Pos2a), `b` the end.
struct SegmentBounds {
    res1inf: f64,
    res1sup: f64,
    res2inf: f64,
    res2sup: f64,
    pos1a: Position,
    pos1b: Position,
    pos2a: Position,
    pos2b: Position,
}

// `ElCLib::Value(u, L)`.
fn at(line: &Line2, u: f64) -> Point2 {
    elclib::line_value_2d(u, line.position())
}
// `ElCLib::Parameter(L, P)`.
fn param(line: &Line2, point: Point2) -> f64 {
    elclib::line_parameter_2d(line.position(), point)
}

impl IntConicConic {
    // The zone back on L1 (swapped on L2 for opposite lines), its positions, and
    // the point or segment it gives.
    pub(super) fn crossing_segment(
        &mut self,
        lines: &Lines<'_>,
        zone: Zone,
    ) -> Result<(), Failure> {
        let (mut res2inf, mut res2sup) = zone.res2;
        let (mut pos2a, mut pos2b) = zone.pos2;
        let (res1inf, res1sup) = if lines.is_opposite {
            let res1 = (zone.u1p_u2 - res2sup, zone.u1p_u2 - res2inf);
            (res2inf, res2sup) = (res2sup, res2inf);
            (pos2a, pos2b) = (pos2b, pos2a);
            res1
        } else {
            (zone.u1m_u2 + res2inf, zone.u1m_u2 + res2sup)
        };
        let (res1inf, pos1a) = position_ll(res1inf, lines.domain1)?;
        let (res1sup, pos1b) = position_ll(res1sup, lines.domain1)?;
        let mut b = SegmentBounds {
            res1inf,
            res1sup,
            res2inf,
            res2sup,
            pos1a,
            pos1b,
            pos2a,
            pos2b,
        };
        let (mut u1, mut u2) = zone.u;
        let result_is_a_point: bool = force_point(lines.tol, &b, (&mut u1, &mut u2));
        if !result_is_a_point && (b.pos1a != Position::Middle || b.pos2a != Position::Middle) {
            return self.segment_from_end(lines, &mut b, (u1, u2));
        }
        if b.pos1b == Position::Middle {
            b.pos1b = b.pos1a;
        }
        if b.pos2b == Position::Middle {
            b.pos2b = b.pos2a;
        }
        if result_is_a_point {
            return self.forced_point(lines, &mut b, (u1, u2));
        }
        self.segment_from_crossing(lines, &mut b, (u1, u2))
    }
}

// Whether the zone is forced to a point: shorter than Tol on a line, or its
// crossing lies on the wrong side of a domain end (the crossing parameters are
// then moved to that end).
fn force_point(long_mini_seg: f64, b: &SegmentBounds, (u1, u2): (&mut f64, &mut f64)) -> bool {
    if (b.res1sup - b.res1inf) <= long_mini_seg || (b.res2sup - b.res2inf).abs() <= long_mini_seg {
        return true;
    }
    let mut forced: bool = false;
    if b.pos1a == Position::Head && b.pos1b != Position::End && *u1 < b.res1inf {
        (forced, *u1, *u2) = (true, b.res1inf, b.res2inf);
    }
    if b.pos1b == Position::End && b.pos1a != Position::Head && *u1 > b.res1sup {
        (forced, *u1, *u2) = (true, b.res1sup, b.res2sup);
    }
    if b.pos2a == Position::Head {
        if b.pos2b != Position::End && *u2 < b.res2inf {
            (forced, *u2, *u1) = (true, b.res2inf, b.res1inf);
        }
    } else if b.pos2a == Position::End && b.pos2b != Position::Head && *u2 > b.res2inf {
        (forced, *u2, *u1) = (true, b.res2inf, b.res1inf);
    }
    if b.pos2b == Position::Head {
        if b.pos2a != Position::End && *u2 < b.res2sup {
            (forced, *u2, *u1) = (true, b.res2sup, b.res1sup);
        }
    } else if b.pos2b == Position::End && b.pos2a != Position::Head && *u2 > b.res2sup {
        (forced, *u2, *u1) = (true, b.res2sup, b.res1sup);
    }
    forced
}

impl IntConicConic {
    // The end point of the segment when an end of it is a domain end: on L1 from
    // L2's value when it is L2's end, on L2 from L1's value otherwise.
    fn segment_end(lines: &Lines<'_>, b: &mut SegmentBounds) -> (Point2, (Transition, Transition)) {
        let tb = lines.transitions(b.pos1b, b.pos2b);
        let ptfin: Point2 = if b.pos1b == Position::Middle {
            let ptfin: Point2 = at(lines.l2, b.res2sup);
            b.res1sup = param(lines.l1, ptfin);
            ptfin
        } else {
            let ptfin: Point2 = at(lines.l1, b.res1sup);
            b.res2sup = param(lines.l2, ptfin);
            ptfin
        };
        (ptfin, tb)
    }

    // A segment starting at a domain end: extremity to extremity, or extremity to
    // the crossing (a point when the latter is shorter than Tol).
    fn segment_from_end(
        &mut self,
        lines: &Lines<'_>,
        b: &mut SegmentBounds,
        (u1, u2): (f64, f64),
    ) -> Result<(), Failure> {
        let ta = lines.transitions(b.pos1a, b.pos2a);
        let ptdebut: Point2 = if b.pos1a == Position::Middle {
            let at_head: bool = b.pos2a == Position::Head;
            let t3: f64 = if lines.is_opposite == at_head {
                b.res2sup
            } else {
                b.res2inf
            };
            let ptdebut: Point2 = at(lines.l2, t3);
            b.res1inf = param(lines.l1, ptdebut);
            ptdebut
        } else {
            let t4: f64 = if b.pos1a == Position::Head {
                b.res1inf
            } else {
                b.res1sup
            };
            let ptdebut: Point2 = at(lines.l1, t4);
            b.res2inf = param(lines.l2, ptdebut);
            ptdebut
        };
        let pt_seg1 = IntersectionPoint::new(ptdebut, b.res1inf, b.res2inf, ta.0, ta.1, false);
        if b.pos1b != Position::Middle || b.pos2b != Position::Middle {
            let (ptfin, tb) = Self::segment_end(lines, b);
            let pt_seg2 = IntersectionPoint::new(ptfin, b.res1sup, b.res2sup, tb.0, tb.1, false);
            self.base.append_segment(IntersectionSegment::bounded(
                pt_seg1,
                pt_seg2,
                lines.is_opposite,
                false,
            ));
            return Ok(());
        }
        // Extremity (L1 or L2) to a middle point of L1 and L2.
        let (u1, pos1b) = position_ll(u1, lines.domain1)?;
        let (u2, pos2b) = position_ll(u2, lines.domain2)?;
        let tb = lines.transitions(pos1b, pos2b);
        let pt_seg2 = IntersectionPoint::new(at(lines.l2, u2), u1, u2, tb.0, tb.1, false);
        let long_mini_seg: f64 = lines.tol;
        if (b.res1inf - u1).abs() > long_mini_seg && (b.res2inf - u2).abs() > long_mini_seg {
            self.base.append_segment(IntersectionSegment::bounded(
                pt_seg1,
                pt_seg2,
                lines.is_opposite,
                false,
            ));
        } else {
            self.base
                .append_point(segment_to_point(&pt_seg1, ta, &pt_seg2, tb));
        }
        Ok(())
    }

    // The forced point: at the domain end of the zone's end when there is one, at
    // the crossing otherwise.
    fn forced_point(
        &mut self,
        lines: &Lines<'_>,
        b: &mut SegmentBounds,
        (u1, u2): (f64, f64),
    ) -> Result<(), Failure> {
        if b.pos1b == Position::Middle && b.pos2b == Position::Middle {
            let (u1, pos1b) = position_ll(u1, lines.domain1)?;
            let (u2, pos2b) = position_ll(u2, lines.domain2)?;
            let tb = lines.transitions(pos1b, pos2b);
            self.base.append_point(IntersectionPoint::new(
                at(lines.l2, u2),
                u1,
                u2,
                tb.0,
                tb.1,
                false,
            ));
            return Ok(());
        }
        let ptfin: Point2 = if b.pos1b == Position::Middle {
            let at_head: bool = b.pos2b == Position::Head;
            let t2: f64 = if lines.is_opposite == at_head {
                b.res2sup
            } else {
                b.res2inf
            };
            let ptfin: Point2 = at(lines.l2, t2);
            b.res1sup = param(lines.l1, ptfin);
            // modified by NIZHNY-MKK  Tue Feb 15 10:54:51 2000
            (b.res1sup, b.pos1b) = position_ll(b.res1sup, lines.domain1)?;
            ptfin
        } else {
            let t1: f64 = if b.pos1b == Position::Head {
                b.res1inf
            } else {
                b.res1sup
            };
            let ptfin: Point2 = at(lines.l1, t1);
            b.res2sup = param(lines.l2, ptfin);
            // modified by NIZHNY-MKK  Tue Feb 15 10:55:08 2000
            (b.res2sup, b.pos2b) = position_ll(b.res2sup, lines.domain2)?;
            ptfin
        };
        let tb = lines.transitions(b.pos1b, b.pos2b);
        self.base.append_point(IntersectionPoint::new(
            ptfin, b.res1sup, b.res2sup, tb.0, tb.1, false,
        ));
        Ok(())
    }

    // A zone starting in the middle of both lines: from the crossing to a domain
    // end (a point when shorter than Tol), or the crossing alone.
    fn segment_from_crossing(
        &mut self,
        lines: &Lines<'_>,
        b: &mut SegmentBounds,
        (u1, u2): (f64, f64),
    ) -> Result<(), Failure> {
        let ta = lines.transitions(b.pos1a, b.pos2a);
        let pt_seg1 = IntersectionPoint::new(at(lines.l2, u2), u1, u2, ta.0, ta.1, false);
        if b.pos1b == Position::Middle && b.pos2b == Position::Middle {
            self.base.append_point(pt_seg1);
            return Ok(());
        }
        let (ptfin, tb) = Self::segment_end(lines, b);
        let pt_seg2 = IntersectionPoint::new(ptfin, b.res1sup, b.res2sup, tb.0, tb.1, false);
        let long_mini_seg: f64 = lines.tol;
        if (u1 - b.res1sup).abs() > long_mini_seg || (u2 - b.res2sup).abs() > long_mini_seg {
            // Modif du 1er Octobre 92 (Pour Composites)
            self.base.append_segment(IntersectionSegment::bounded(
                pt_seg1,
                pt_seg2,
                lines.is_opposite,
                false,
            ));
        } else {
            self.base
                .append_point(segment_to_point(&pt_seg1, ta, &pt_seg2, tb));
        }
        Ok(())
    }
}
