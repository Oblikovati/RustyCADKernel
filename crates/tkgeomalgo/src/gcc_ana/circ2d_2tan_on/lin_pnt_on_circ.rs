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
// Rust translation of GccAna_Circ2d2TanOn_10.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to a line and through a point with their centre on a circle: a
//! boundary case at the far side of the locus from the point, else the
//! intersections of the locus with the line-point bisector.
//!
//! Native quirks kept: the boundary case overwrites its centre parameter with 0;
//! the general case places the tangency point on the line at the distance dist1
//! of the boundary candidate, not at the solution radius.
use super::Circ2d2TanOn;
use super::circ_lin_on_line::{check_line, side_of};
use crate::gcc_ana::LinPnt2dBisec;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::{Position, QualifiedLin};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Point2, OnCirc, Tolerance): circles tangent
    /// to a qualified line and through a point, centred on `on_circ`;
    /// `Circ2d2TanOn::lin_pnt_on_circ(l1, point, circle, tol)?`.
    pub fn lin_pnt_on_circ(
        qualified1: QualifiedLin,
        point2: Point2,
        on_circ: Circle2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_line(qualified1)?;
        let tol: f64 = tolerance.abs();
        let l1: Line2 = qualified1.qualified();
        let ron: f64 = on_circ.radius();
        let distpc: f64 = on_circ.location().distance(point2);
        let dir = Direction2::from_xy(on_circ.location().xy() - point2.xy())?;
        let pinterm = Point2::from(point2.xy() + (distpc + ron) * dir.xy());
        let dist1: f64 = l1.distance(pinterm);
        if (dist1 - distpc + ron).abs() <= tol {
            let normal = Direction2::new(-l1.direction().y(), l1.direction().x())?;
            let direc = Direction2::from_xy(l1.location().xy() - pinterm.xy())?;
            circ.well_done = if qualified1.is_outside() {
                direc.dot(normal) >= 0.0
            } else if qualified1.is_enclosed() {
                direc.dot(normal) <= 0.0
            } else {
                true
            };
            if circ.well_done {
                circ.side_solution((qualified1, point2), pinterm, dist1, dist1)?;
                return circ.boundary_parameter_10();
            }
        }
        let bis = LinPnt2dBisec::new(l1, point2)?;
        if !bis.is_done() {
            return Ok(circ);
        }
        let sol: Handle<dyn Bisec> = bis.this_solution()?;
        let intp = match sol.arc_type() {
            IType::Lin => AnaIntersection::line_circle(sol.line()?, on_circ)?,
            IType::Par => {
                AnaIntersection::circle_conic(on_circ, &Conic::from_parabola(sol.parabola()?))?
            }
            _ => AnaIntersection::new(),
        };
        if !intp.is_done() {
            return Ok(circ);
        }
        if !intp.is_empty()? {
            for j in 1..=intp.nb_points()? {
                let center: Point2 = intp.point(j)?.value();
                let side: f64 = side_of(l1, center);
                let ok: bool = (qualified1.is_enclosed() && side <= 0.0)
                    || (qualified1.is_outside() && side >= 0.0)
                    || qualified1.is_unqualified();
                if ok {
                    circ.side_solution((qualified1, point2), center, l1.distance(center), dist1)?;
                    let index: i32 = circ.nbr_sol;
                    circ.centre_on(index, center, elclib::parameter(&on_circ, center)?)?;
                }
            }
        }
        circ.well_done = true;
        Ok(circ)
    }

    // The boundary case stores its centre, computes its parameter, then sets it
    // to 0 (native order).
    fn boundary_parameter_10(mut self) -> Result<Self, Failure> {
        let index: i32 = self.nbr_sol;
        let centre: Point2 = self.cirsol.get(index)?.location();
        self.centre_on(index, centre, 0.0)?;
        Ok(self)
    }

    // A circle centred at `center` of radius `radius`, tangent to the line at
    // `center + reach * Dir(sign * (-dy, dx))` and through the point.
    fn side_solution(
        &mut self,
        (q1, point2): (QualifiedLin, Point2),
        center: Point2,
        radius: f64,
        reach: f64,
    ) -> Result<(), Failure> {
        let l1: Line2 = q1.qualified();
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        let norm_l1 = Direction2::new(-l1.direction().y(), l1.direction().x())?;
        let dc1 = Direction2::from_xy(l1.location().xy() - center.xy())?;
        let sign: f64 = dc1.dot(norm_l1);
        let q = if !q1.is_unqualified() {
            q1.qualifier()
        } else if dc1.dot(norm_l1) > 0.0 {
            Position::Outside
        } else {
            Position::Enclosed
        };
        self.set_qualifiers(index, q, Position::NoQualifier)?;
        let foot = Direction2::from_xy(sign * Xy::new(-l1.direction().y(), l1.direction().x()))?;
        let p1 = Point2::from(center.xy() + reach * foot.xy());
        self.tangency1.record(
            index,
            p1,
            elclib::parameter(&solution, p1)?,
            elclib::parameter(&l1, p1)?,
        )?;
        self.tangency2
            .record(index, point2, elclib::parameter(&solution, point2)?, 0.0)?;
        self.pntcen.set(index, center)
    }
}
