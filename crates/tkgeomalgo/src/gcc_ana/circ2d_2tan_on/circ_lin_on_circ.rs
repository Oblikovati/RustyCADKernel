// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of GccAna_Circ2d2TanOn_7.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to a circle and a line with their centre on a circle: a
//! boundary case centred where the line from C1 to the locus centre meets the
//! locus, else the intersections of the locus with the circle-line bisectors.
//!
//! Native quirks kept: the boundary case never sets the centre point (its locus
//! parameter is the one of the origin) and rejects enclosed and outside circles on
//! the same side test; the general case keeps the radius of the previous centre
//! when the line qualifier rejects a centre, and scales the renormalized normal
//! for the tangency point on the line.
use super::circ_lin_on_line::{check_line, side_of};
use super::{Circ2d2TanOn, NAME};
use crate::gcc_ana::solutions::{circle_at, line_qualifier, line_tangency, qualify};
use crate::gcc_ana::{CircLin2dBisec, check_qualifier};
use crate::gcc_ent::{QualifiedCirc, QualifiedLin};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Point2};

struct Args {
    q1: QualifiedCirc,
    q2: QualifiedLin,
    on_circ: Circle2,
    tolerance: f64,
}

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Qualified2, OnCirc, Tolerance): circles
    /// tangent to a qualified circle and a qualified line, centred on `on_circ`;
    /// `Circ2d2TanOn::circ_lin_on_circ(q1, l2, circle, tol)?`.
    pub fn circ_lin_on_circ(
        qualified1: QualifiedCirc,
        qualified2: QualifiedLin,
        on_circ: Circle2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_qualifier(qualified1.qualifier(), NAME)?;
        check_line(qualified2)?;
        let a = Args {
            q1: qualified1,
            q2: qualified2,
            on_circ,
            tolerance,
        };
        if circ.boundary_7(&a)? {
            return Ok(circ);
        }
        let bis = CircLin2dBisec::new(qualified1.qualified(), qualified2.qualified());
        if !bis.is_done() {
            return Ok(circ);
        }
        let mut radius: f64 = 0.0;
        for i in 1..=bis.nb_solutions() {
            let sol: Handle<dyn Bisec> = bis.this_solution(i)?;
            let intp = match sol.arc_type() {
                IType::Lin => AnaIntersection::line_circle(sol.line()?, on_circ)?,
                IType::Par => {
                    AnaIntersection::circle_conic(on_circ, &Conic::from_parabola(sol.parabola()?))?
                }
                _ => AnaIntersection::new(),
            };
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? && !intp.parallel_elements()? && !intp.identical_elements()? {
                for j in 1..=intp.nb_points()? {
                    circ.candidate_7(&a, intp.point(j)?.value(), &mut radius)?;
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    fn boundary_7(&mut self, a: &Args) -> Result<bool, Failure> {
        let (c1, l2, tol) = (a.q1.qualified(), a.q2.qualified(), a.tolerance.abs());
        let (center1, r1) = (c1.location(), c1.radius());
        let ron: f64 = a.on_circ.radius();
        let distcco: f64 = a.on_circ.location().distance(center1);
        let dircc = Direction2::from_xy(a.on_circ.location().xy() - center1.xy())?;
        let pinterm = Point2::from(center1.xy() + (distcco - ron) * dircc.xy());
        let distpl2: f64 = l2.distance(pinterm);
        let distcc1: f64 = pinterm.distance(center1);
        let d1: f64 = (distpl2 - (distcc1 - r1).abs()).abs();
        let d2: f64 = (distpl2 - (distcc1 + r1)).abs();
        // Native retries with the same point and the same defects.
        if d1 > tol || d2 > tol {
            return Ok(false);
        }
        let mut rradius: Vec<f64> = if a.q1.is_enclosed() || a.q1.is_outside() {
            vec![(distcc1 - r1).abs()]
        } else if a.q1.is_enclosing() {
            vec![r1 + distcc1]
        } else {
            vec![(distcc1 - r1).abs(), r1 + distcc1]
        };
        let dirbid = Direction2::from_xy(l2.location().xy() - pinterm.xy())?;
        let normal = Direction2::new(-l2.direction().y(), l2.direction().x())?;
        if (a.q1.is_enclosed() || a.q1.is_outside()) && dirbid.dot(normal) < 0.0 {
            rradius.clear();
        }
        for r in rradius {
            if (r - distpl2).abs() <= tol {
                self.well_done = true;
                self.boundary_solution_7(a, pinterm, r)?;
            }
        }
        Ok(self.well_done)
    }

    fn boundary_solution_7(&mut self, a: &Args, pinterm: Point2, r: f64) -> Result<(), Failure> {
        let (c1, l2, tol) = (a.q1.qualified(), a.q2.qualified(), a.tolerance.abs());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(pinterm, r)?;
        self.cirsol.set(index, solution)?;
        let dc1 = Direction2::from_xy(c1.location().xy() - pinterm.xy())?;
        let distcc1: f64 = pinterm.distance(c1.location());
        let normal = Direction2::new(-l2.direction().y(), l2.direction().x())?;
        let q2 = line_qualifier(a.q2, pinterm, normal)?;
        self.set_qualifiers(
            index,
            qualify(a.q1.qualifier(), distcc1, r, c1.radius(), tol, false),
            q2,
        )?;
        let p1 = Point2::from(pinterm.xy() + r * dc1.xy());
        let p2: Point2 = line_tangency(l2, pinterm, r)?;
        self.tangency1.record(
            index,
            p1,
            elclib::parameter(&solution, p1)?,
            elclib::parameter(&c1, p1)?,
        )?;
        self.tangency2.record(
            index,
            p2,
            elclib::parameter(&solution, p2)?,
            elclib::parameter(&l2, p2)?,
        )?;
        let unset_centre: Point2 = self.pntcen.get(index)?;
        self.parcen3
            .set(index, elclib::parameter(&a.on_circ, unset_centre)?)
    }

    fn candidate_7(&mut self, a: &Args, center: Point2, radius: &mut f64) -> Result<(), Failure> {
        let (c1, l2, tolerance) = (a.q1.qualified(), a.q2.qualified(), a.tolerance);
        let r1: f64 = c1.radius();
        let dist1: f64 = center.distance(c1.location());
        let dist2: f64 = l2.distance(center);
        let mut ok: bool = false;
        if a.q1.is_enclosed() || a.q1.is_outside() {
            let inside: bool = if a.q1.is_enclosed() {
                dist1 - r1 < tolerance
            } else {
                r1 - dist1 < tolerance
            };
            if inside && ((r1 - dist1).abs() - dist2).abs() < tolerance {
                ok = true;
            }
        } else if a.q1.is_enclosing() || a.q1.is_unqualified() {
            ok = true;
        }
        let side: f64 = side_of(l2, center);
        let accepted: bool = if a.q2.is_enclosed() {
            side <= 0.0
        } else if a.q2.is_outside() {
            side >= 0.0
        } else {
            a.q2.is_unqualified()
        };
        if ok && accepted {
            *radius = dist2;
        }
        if ok {
            self.solution_7(a, center, *radius)?;
        }
        Ok(())
    }

    fn solution_7(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let (c1, l2, tol) = (a.q1.qualified(), a.q2.qualified(), a.tolerance.abs());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, r)?;
        self.cirsol.set(index, solution)?;
        let norm_l2 = Direction2::new(-l2.direction().y(), l2.direction().x())?;
        let dc2 = Direction2::from_xy(l2.location().xy() - center.xy())?;
        let distcc1: f64 = center.distance(c1.location());
        let q2 = line_qualifier(a.q2, center, norm_l2)?;
        self.set_qualifiers(
            index,
            qualify(a.q1.qualifier(), distcc1, r, c1.radius(), tol, false),
            q2,
        )?;
        if center.distance(c1.location()) <= a.tolerance && (r - c1.radius()).abs() <= a.tolerance {
            self.tangency1.same.set(index, 1)?;
        } else {
            let dc1 = Direction2::from_xy(c1.location().xy() - center.xy())?;
            let p1 = Point2::from(center.xy() + r * dc1.xy());
            self.tangency1.record(
                index,
                p1,
                elclib::parameter(&solution, p1)?,
                elclib::parameter(&c1, p1)?,
            )?;
        }
        // sign = dc2 . gp_Dir2d(normL2.XY()); dc2 = gp_Dir2d(sign * normL2.XY()).
        let sign: f64 = dc2.dot(Direction2::from_xy(norm_l2.xy())?);
        let foot = Direction2::from_xy(sign * norm_l2.xy())?;
        let p2 = Point2::from(center.xy() + r * foot.xy());
        self.tangency2.record(
            index,
            p2,
            elclib::parameter(&solution, p2)?,
            elclib::parameter(&l2, p2)?,
        )?;
        self.centre_on(index, center, elclib::parameter(&a.on_circ, center)?)
    }
}
