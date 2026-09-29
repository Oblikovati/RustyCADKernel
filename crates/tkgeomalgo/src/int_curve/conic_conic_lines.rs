// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of IntCurve_IntConicConic_1.cxx (Perform for two lines, crossing
// lines) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IntConicConic::Perform(gp_Lin2d, gp_Lin2d)`: the zone where two
//! crossing lines are closer than the tolerance, clipped by both domains, gives a
//! point or (for nearly parallel lines) a segment.
use super::IntConicConic;
use super::conic_conic_line_tools::{
    LinePair, TOLERANCE_ANGULAIRE, check_ll_coincidence, compute_int_point, domain_intersection,
    find_position_ll, line_line_geometric_intersection,
};
use crate::int_res2d::{Domain, Position, Situation, Transition, TypeTrans};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Vector2};

/// The two lines, their domains and the quantities of the crossing case.
pub(super) struct Lines<'a> {
    pub(super) l1: &'a Line2,
    pub(super) l2: &'a Line2,
    pub(super) domain1: &'a Domain,
    pub(super) domain2: &'a Domain,
    pub(super) tol: f64,
    pub(super) is_opposite: bool,
    pub(super) prod_vect_tan: f64,
}

impl Lines<'_> {
    /// The transitions of a crossing at the positions, from the sign of the cross
    /// product of the directions (TOUCH-like unknown within 1e-15).
    pub(super) fn transitions(&self, pos1: Position, pos2: Position) -> (Transition, Transition) {
        let (mut t1, mut t2) = (Transition::default(), Transition::default());
        if self.prod_vect_tan >= TOLERANCE_ANGULAIRE {
            t1.set_value(false, pos1, TypeTrans::Out);
            t2.set_value(false, pos2, TypeTrans::In);
        } else if self.prod_vect_tan <= -TOLERANCE_ANGULAIRE {
            t1.set_value(false, pos1, TypeTrans::In);
            t2.set_value(false, pos2, TypeTrans::Out);
        } else {
            t1.set_touch_value(false, pos1, Situation::Unknown, self.is_opposite);
            t2.set_touch_value(false, pos2, Situation::Unknown, self.is_opposite);
        }
        (t1, t2)
    }
    // The type of the crossing for the first curve given as `first_out` (Out when the
    // cross product is positive), Undecided within 1e-15.
    fn crossing_type(&self, positive: TypeTrans, negative: TypeTrans) -> TypeTrans {
        if self.prod_vect_tan >= TOLERANCE_ANGULAIRE {
            positive
        } else if self.prod_vect_tan <= -TOLERANCE_ANGULAIRE {
            negative
        } else {
            TypeTrans::Undecided
        }
    }
}

impl IntConicConic {
    /// `Perform(gp_Lin2d, D1, gp_Lin2d, D2, TolConf, Tol)`: the confusion tolerance
    /// is unused; `Tol` (at least Precision::PConfusion()) is the distance under
    /// which the lines are merged. Trimmed lines whose ends are within `Tol` of the
    /// other line are treated as coincident.
    /// `inter.perform_lines(l1, &d1, l2, &d2, 1e-9, 1e-9)?`.
    pub fn perform_lines(
        &mut self,
        l1: Line2,
        domain1: &Domain,
        l2: Line2,
        domain2: &Domain,
        _tol_conf: f64,
        tol_r: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let tol: f64 = if tol_r < precision::p_confusion() {
            precision::p_confusion()
        } else {
            tol_r
        };
        let solution = line_line_geometric_intersection(&l1, &l2, tol);
        let tan1: Vector2 = Vector2::from(l1.direction());
        let tan2: Vector2 = Vector2::from(l2.direction());
        let a_cos_t1t2: f64 = tan1.dot(tan2);
        let lines = Lines {
            l1: &l1,
            l2: &l2,
            domain1,
            domain2,
            tol,
            is_opposite: a_cos_t1t2 < 0.0,
            prod_vect_tan: tan1.crossed(tan2),
        };
        self.base.done = true;
        let mut nbsol: i32 = solution.nbsol;
        if nbsol == 1 && check_ll_coincidence(&l1, &l2, domain1, domain2, tol)? {
            nbsol = 2;
        }
        if nbsol == 1 {
            return self.crossing_lines(
                &lines,
                (solution.u1, solution.u2),
                solution.sin_demi_angle,
                a_cos_t1t2,
            );
        }
        if nbsol == 2 {
            return self.coincident_lines(&lines);
        }
        Ok(())
    }

    // The crossing lines: the zone `U1 +/- d` where they are closer than Tol,
    // enlarged to the domain ends within tolerance of the other line.
    fn crossing_lines(
        &mut self,
        lines: &Lines<'_>,
        (u1, u2): (f64, f64),
        a_half_sin_l1l2: f64,
        a_cos_t1t2: f64,
    ) -> Result<(), Failure> {
        let d: f64 = 0.5 * lines.tol / a_half_sin_l1l2;
        let (mut u1inf, mut u1sup) = (u1 - d, u1 + d);
        for p in end_parameters_near(lines)? {
            if u1inf > p {
                u1inf = p;
            }
            if u1sup < p {
                u1sup = p;
            }
        }
        let (res1inf, res1sup, pos1a, pos1b) = domain_intersection(lines.domain1, u1inf, u1sup)?;
        if (res1sup - res1inf) < 0.0 {
            return Ok(());
        }
        let long_mini_seg: f64 = lines.tol;
        if (res1sup - res1inf) <= long_mini_seg || (pos1a == pos1b && pos1a != Position::Middle) {
            // One point only: the zone is too short, or both ends are the same end.
            let a_cur_trans: TypeTrans = lines.crossing_type(TypeTrans::Out, TypeTrans::In);
            let pair = LinePair {
                cur_domain: lines.domain1,
                domain_other: lines.domain2,
                cur_lin: lines.l1,
                other_lin: lines.l2,
            };
            if let Some(point) = compute_int_point(
                &pair,
                a_cos_t1t2,
                (u1, u2),
                (res1inf, res1sup),
                1,
                a_cur_trans,
            )? {
                self.base.append_point(point);
            }
            return Ok(());
        }
        self.crossing_zone(lines, (u1, u2), (res1inf, res1sup), a_cos_t1t2)
    }
}

// The parameters on L1 of the domain ends within their tolerance of the other line:
// the ends of L1 itself, then the projections of those of L2.
fn end_parameters_near(lines: &Lines<'_>) -> Result<Vec<f64>, Failure> {
    let mut parameters: Vec<f64> = Vec::with_capacity(4);
    let (d1, d2) = (lines.domain1, lines.domain2);
    if d1.has_first_point() && lines.l2.distance(d1.first_point()?) < d1.first_tolerance()? {
        parameters.push(d1.first_parameter()?);
    }
    if d1.has_last_point() && lines.l2.distance(d1.last_point()?) < d1.last_tolerance()? {
        parameters.push(d1.last_parameter()?);
    }
    if d2.has_first_point() && lines.l1.distance(d2.first_point()?) < d2.first_tolerance()? {
        parameters.push(elclib::line_parameter_2d(
            lines.l1.position(),
            d2.first_point()?,
        ));
    }
    if d2.has_last_point() && lines.l1.distance(d2.last_point()?) < d2.last_tolerance()? {
        parameters.push(elclib::line_parameter_2d(
            lines.l1.position(),
            d2.last_point()?,
        ));
    }
    Ok(parameters)
}

impl IntConicConic {
    // The zone on L1 is longer than Tol: its image on L2 clipped by Domain2 is a
    // segment, or a point when shorter than Tol there.
    fn crossing_zone(
        &mut self,
        lines: &Lines<'_>,
        (u1, u2): (f64, f64),
        (res1inf, res1sup): (f64, f64),
        a_cos_t1t2: f64,
    ) -> Result<(), Failure> {
        let (u1m_u2, u1p_u2) = (u1 - u2, u1 + u2);
        let (u2inf, u2sup) = if lines.is_opposite {
            (u1p_u2 - res1sup, u1p_u2 - res1inf)
        } else {
            (res1inf - u1m_u2, res1sup - u1m_u2)
        };
        let (res2inf, res2sup, pos2a, pos2b) = domain_intersection(lines.domain2, u2inf, u2sup)?;
        let res2sup_m_res2inf: f64 = res2sup - res2inf;
        if res2sup_m_res2inf < 0.0 {
            return Ok(());
        }
        if res2sup_m_res2inf > lines.tol || (pos2a == pos2b && pos2a != Position::Middle) {
            let zone = super::conic_conic_lines_segment::Zone {
                u: (u1, u2),
                u1m_u2,
                u1p_u2,
                res2: (res2inf, res2sup),
                pos2: (pos2a, pos2b),
            };
            return self.crossing_segment(lines, zone);
        }
        // (Intersection and Domain1) and Domain2 is a point; Res1sup may differ
        // from U2 but Res1sup - Res1inf < Tol.
        let a_cur_trans: TypeTrans = lines.crossing_type(TypeTrans::In, TypeTrans::Out);
        let pair = LinePair {
            cur_domain: lines.domain2,
            domain_other: lines.domain1,
            cur_lin: lines.l2,
            other_lin: lines.l1,
        };
        if let Some(point) = compute_int_point(
            &pair,
            a_cos_t1t2,
            (u2, u1),
            (res2inf, res2sup),
            2,
            a_cur_trans,
        )? {
            self.base.append_point(point);
        }
        Ok(())
    }
}

/// `FindPositionLL` on a copy of the parameter: the snapped parameter and position.
pub(super) fn position_ll(param: f64, domain: &Domain) -> Result<(f64, Position), Failure> {
    let mut snapped: f64 = param;
    let position: Position = find_position_ll(&mut snapped, domain)?;
    Ok((snapped, position))
}
