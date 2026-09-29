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
// Rust translation of GccAna_Circ2d2TanOn_3.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to a circle and through a point with their centre on a line:
//! a boundary case centred at the foot of the point on the locus, else the
//! intersections of the locus with the circle-point bisectors.
//!
//! Native quirks kept: the boundary case leaves the second argument parameter
//! unset, and the general case writes the first argument parameter at the
//! bisector index instead of the solution index (unset slots are 0 here,
//! uninitialized natively); an unqualified circle takes the distance to the
//! point as radius whenever it differs from the distance to the circle centre.
use super::{Circ2d2TanOn, NAME};
use crate::gcc_ana::solutions::{circle_at, qualify};
use crate::gcc_ana::{CircPnt2dBisec, check_qualifier};
use crate::gcc_ent::{Position, QualifiedCirc};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

struct Args {
    q1: QualifiedCirc,
    point2: Point2,
    on_line: Line2,
    tolerance: f64,
}

// The foot of `point` on the locus (native tries +normal first, then -normal).
pub(super) fn foot_on_line(point: Point2, on_line: Line2, tol: f64) -> (Point2, f64) {
    let dp2l: f64 = on_line.distance(point);
    let donline: Direction2 = on_line.direction();
    let normal = Xy::new(-donline.y(), donline.x());
    let mut pinterm = Point2::from(point.xy() + dp2l * normal);
    if on_line.distance(pinterm) > tol {
        pinterm = Point2::from(point.xy() - dp2l * normal);
    }
    (pinterm, dp2l)
}

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Point2, OnLine, Tolerance): circles tangent
    /// to a qualified circle and through a point, centred on `on_line`;
    /// `Circ2d2TanOn::circ_pnt_on_line(q1, point, line, tol)?`.
    pub fn circ_pnt_on_line(
        qualified1: QualifiedCirc,
        point2: Point2,
        on_line: Line2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        let tol: f64 = tolerance.abs();
        check_qualifier(qualified1.qualifier(), NAME)?;
        let a = Args {
            q1: qualified1,
            point2,
            on_line,
            tolerance,
        };
        let c1: Circle2 = qualified1.qualified();
        let r1: f64 = c1.radius();
        let (pinterm, dp2l) = foot_on_line(point2, on_line, tol);
        let dist: f64 = pinterm.distance(c1.location());
        let near = |value: f64| value.abs() <= tol;
        circ.well_done = (qualified1.is_enclosed() && near(r1 - dist - dp2l))
            || (qualified1.is_enclosing() && near(r1 + dist - dp2l))
            || (qualified1.is_outside() && near(dist - dp2l))
            || (qualified1.is_unqualified()
                && (near(dist - dp2l) || near(r1 - dist - dp2l) || near(r1 + dist - dp2l)));
        if circ.well_done {
            circ.boundary_3(&a, pinterm, dp2l)?;
            return Ok(circ);
        }
        let mut radius = crate::gcc_ana::solutions::Slots::<f64>::new(2);
        let bis = CircPnt2dBisec::new(c1, point2);
        if !bis.is_done() {
            return Ok(circ);
        }
        for i in 1..=bis.nb_solutions() {
            let sol: Handle<dyn Bisec> = bis.this_solution(i)?;
            let intp = match sol.arc_type() {
                IType::Lin => AnaIntersection::lines(on_line, sol.line()?),
                IType::Cir => AnaIntersection::line_circle(on_line, sol.circle()?)?,
                IType::Ell => {
                    AnaIntersection::line_conic(on_line, &Conic::from_ellipse(sol.ellipse()?))?
                }
                IType::Hpr => {
                    AnaIntersection::line_conic(on_line, &Conic::from_hyperbola(sol.hyperbola()?))?
                }
                _ => AnaIntersection::new(),
            };
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for j in 1..=intp.nb_points()? {
                    circ.candidate_3(&a, i, intp.point(j)?.value(), &mut radius)?;
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    fn boundary_3(&mut self, a: &Args, pinterm: Point2, dp2l: f64) -> Result<(), Failure> {
        let c1: Circle2 = a.q1.qualified();
        let tol: f64 = a.tolerance.abs();
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(pinterm, dp2l)?;
        self.cirsol.set(index, solution)?;
        let dc1 = Direction2::from_xy(c1.location().xy() - pinterm.xy())?;
        let distcc1: f64 = pinterm.distance(c1.location());
        let q1 = qualify(a.q1.qualifier(), distcc1, dp2l, c1.radius(), tol, false);
        self.set_qualifiers(index, q1, Position::NoQualifier)?;
        let p1 = Point2::from(pinterm.xy() + dp2l * dc1.xy());
        self.tangency1.record(
            index,
            p1,
            elclib::parameter(&solution, p1)?,
            elclib::parameter(&c1, p1)?,
        )?;
        self.tangency2.point.set(index, a.point2)?;
        self.tangency2
            .par_sol
            .set(index, elclib::parameter(&solution, a.point2)?)?;
        let centre: Point2 = solution.location();
        self.centre_on(index, centre, elclib::parameter(&a.on_line, centre)?)
    }

    fn candidate_3(
        &mut self,
        a: &Args,
        i: i32,
        center: Point2,
        radius: &mut crate::gcc_ana::solutions::Slots<f64>,
    ) -> Result<(), Failure> {
        let c1: Circle2 = a.q1.qualified();
        let (r1, tolerance) = (c1.radius(), a.tolerance);
        let dist1: f64 = c1.location().distance(center);
        let mut ok: bool = false;
        if a.q1.is_enclosed() || a.q1.is_outside() {
            let inside: bool = if a.q1.is_enclosed() {
                dist1 - r1 <= tolerance
            } else {
                r1 - dist1 <= tolerance
            };
            if inside {
                ok = true;
                radius.set(1, (r1 - dist1).abs())?;
            }
        } else if a.q1.is_enclosing() {
            ok = true;
            radius.set(1, r1 + dist1)?;
        } else if a.q1.is_unqualified() {
            let popradius: f64 = center.distance(a.point2);
            if (popradius - dist1).abs() != 0.0 {
                ok = true;
                radius.set(1, popradius)?;
            }
        }
        if ok {
            self.solution_3(a, i, center, radius.get(1)?)?;
        }
        Ok(())
    }

    fn solution_3(&mut self, a: &Args, i: i32, center: Point2, radius: f64) -> Result<(), Failure> {
        let c1: Circle2 = a.q1.qualified();
        let tol: f64 = a.tolerance.abs();
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        let distcc1: f64 = center.distance(c1.location());
        let q1 = qualify(a.q1.qualifier(), distcc1, radius, c1.radius(), tol, false);
        self.set_qualifiers(index, q1, Position::NoQualifier)?;
        if center.distance(c1.location()) <= a.tolerance
            && (radius - c1.radius()).abs() <= a.tolerance
        {
            self.tangency1.same.set(index, 1)?;
        } else {
            self.tangency1.same.set(index, 0)?;
            let dc1 = Direction2::from_xy(c1.location().xy() - center.xy())?;
            let p1 = Point2::from(center.xy() + radius * dc1.xy());
            self.tangency1.point.set(index, p1)?;
            self.tangency1
                .par_sol
                .set(index, elclib::parameter(&solution, p1)?)?;
            // Native writes pararg1(i), i being the bisector index.
            self.tangency1.par_arg.set(i, elclib::parameter(&c1, p1)?)?;
        }
        self.tangency2.record(
            index,
            a.point2,
            elclib::parameter(&solution, a.point2)?,
            0.0,
        )?;
        self.centre_on(index, center, elclib::parameter(&a.on_line, center)?)
    }
}
