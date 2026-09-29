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
// Rust translation of GccAna_Circ2d2TanOn_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to a circle and a line with their centre on a line: a limit
//! case built from the foot of C1 on the locus, else the intersections of the
//! locus with the circle-line bisectors.
//!
//! Native quirk kept: the radius of the general case persists between the
//! candidate centres, so a centre the line qualifier rejects (without clearing
//! `ok`) gets the radius of the previous one.
use super::{Circ2d2TanOn, NAME};
use crate::gcc_ana::solutions::{circle_at, line_qualifier, line_tangency, qualify};
use crate::gcc_ana::{CircLin2dBisec, check_qualifier};
use crate::gcc_ent::{QualifiedCirc, QualifiedLin, bad_qualifier};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

pub(super) fn check_line(qualified: QualifiedLin) -> Result<(), Failure> {
    if qualified.is_enclosed() || qualified.is_outside() || qualified.is_unqualified() {
        return Ok(());
    }
    Err(bad_qualifier(&format!(
        "{NAME}: line qualifier {:?}; expected enclosed, outside or unqualified",
        qualified.qualifier()
    )))
}

// The side of `center` relative to the line: (ox - cx) * (-dy) + (oy - cy) * dx.
pub(super) fn side_of(line: Line2, center: Point2) -> f64 {
    let (origin, dir) = (line.location(), line.direction());
    ((origin.x() - center.x()) * (-dir.y())) + ((origin.y() - center.y()) * (dir.x()))
}

struct Args {
    q1: QualifiedCirc,
    q2: QualifiedLin,
    on_line: Line2,
    tolerance: f64,
}

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Qualified2, OnLine, Tolerance): circles
    /// tangent to a qualified circle and a qualified line, centred on `on_line`;
    /// `Circ2d2TanOn::circ_lin_on_line(q1, l2, line, tol)?`.
    pub fn circ_lin_on_line(
        qualified1: QualifiedCirc,
        qualified2: QualifiedLin,
        on_line: Line2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_qualifier(qualified1.qualifier(), NAME)?;
        check_line(qualified2)?;
        let a = Args {
            q1: qualified1,
            q2: qualified2,
            on_line,
            tolerance,
        };
        if circ.limit_case(&a)? {
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
                IType::Lin => AnaIntersection::lines(on_line, sol.line()?),
                IType::Par => {
                    AnaIntersection::line_conic(on_line, &Conic::from_parabola(sol.parabola()?))?
                }
                _ => AnaIntersection::new(),
            };
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for j in 1..=intp.nb_points()? {
                    circ.bisector_candidate(&a, intp.point(j)?.value(), &mut radius)?;
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    // The limit case: the point of the locus at the distance of C1 along its
    // normal, when it is also tangent to the line. Returns true when it is built.
    fn limit_case(&mut self, a: &Args) -> Result<bool, Failure> {
        let (c1, l2) = (a.q1.qualified(), a.q2.qualified());
        let (tol, r1, center1) = (a.tolerance.abs(), c1.radius(), c1.location());
        let distcl: f64 = a.on_line.distance(center1);
        let normal = Xy::new(-a.on_line.direction().y(), a.on_line.direction().x());
        // Native computes the same point again when it is off the locus.
        let pinterm = Point2::from(center1.xy() + distcl * normal);
        let dist2: f64 = l2.distance(pinterm);
        let ok: bool = if a.q1.is_enclosed() || a.q1.is_outside() {
            (distcl - r1 - dist2).abs() <= tol
        } else if a.q1.is_enclosing() {
            (dist2 - distcl - r1).abs() <= tol
        } else {
            true
        };
        let dir_l2: Direction2 = l2.direction();
        let normal2 = Xy::new(-dir_l2.y(), dir_l2.x());
        if ok && !a.q2.is_unqualified() {
            let pbid = if a.q2.is_outside() {
                Point2::from(pinterm.xy() + dist2 * normal2)
            } else {
                Point2::from(pinterm.xy() - dist2 * normal2)
            };
            if l2.distance(pbid) <= tol {
                self.well_done = true;
            }
        }
        if !self.well_done {
            return Ok(false);
        }
        self.limit_solution(a, pinterm, dist2)?;
        Ok(true)
    }

    fn limit_solution(&mut self, a: &Args, pinterm: Point2, dist2: f64) -> Result<(), Failure> {
        let (c1, l2) = (a.q1.qualified(), a.q2.qualified());
        let tol: f64 = a.tolerance.abs();
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(pinterm, dist2)?;
        self.cirsol.set(index, solution)?;
        let dc1 = Direction2::from_xy(c1.location().xy() - pinterm.xy())?;
        let distcc1: f64 = pinterm.distance(c1.location());
        let q2 = line_qualifier(
            a.q2,
            pinterm,
            Direction2::new(-l2.direction().y(), l2.direction().x())?,
        )?;
        self.set_qualifiers(
            index,
            qualify(a.q1.qualifier(), distcc1, dist2, c1.radius(), tol, false),
            q2,
        )?;
        let p1 = Point2::from(pinterm.xy() + dist2 * dc1.xy());
        let p2: Point2 = line_tangency(l2, pinterm, dist2)?;
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
        let centre: Point2 = solution.location();
        self.centre_on(index, centre, elclib::parameter(&a.on_line, centre)?)
    }

    fn bisector_candidate(
        &mut self,
        a: &Args,
        center: Point2,
        radius: &mut f64,
    ) -> Result<(), Failure> {
        let (c1, l2) = (a.q1.qualified(), a.q2.qualified());
        let (r1, tolerance) = (c1.radius(), a.tolerance);
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
            self.bisector_solution_1(a, center, *radius)?;
        }
        Ok(())
    }

    fn bisector_solution_1(
        &mut self,
        a: &Args,
        center: Point2,
        radius: f64,
    ) -> Result<(), Failure> {
        let (c1, l2) = (a.q1.qualified(), a.q2.qualified());
        let tol: f64 = a.tolerance.abs();
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        let dc1 = Direction2::from_xy(c1.location().xy() - center.xy())?;
        let distcc1: f64 = center.distance(c1.location());
        let q2 = line_qualifier(
            a.q2,
            center,
            Direction2::new(-l2.direction().y(), l2.direction().x())?,
        )?;
        self.set_qualifiers(
            index,
            qualify(a.q1.qualifier(), distcc1, radius, c1.radius(), tol, false),
            q2,
        )?;
        if center.distance(c1.location()) <= a.tolerance
            && (radius - c1.radius()).abs() <= a.tolerance
        {
            self.tangency1.same.set(index, 1)?;
        } else {
            let p1 = Point2::from(center.xy() + radius * dc1.xy());
            self.tangency1.record(
                index,
                p1,
                elclib::parameter(&solution, p1)?,
                elclib::parameter(&c1, p1)?,
            )?;
        }
        let p2: Point2 = line_tangency(l2, center, radius)?;
        self.tangency2.record(
            index,
            p2,
            elclib::parameter(&solution, p2)?,
            elclib::parameter(&l2, p2)?,
        )?;
        self.centre_on(index, center, elclib::parameter(&a.on_line, center)?)
    }
}
