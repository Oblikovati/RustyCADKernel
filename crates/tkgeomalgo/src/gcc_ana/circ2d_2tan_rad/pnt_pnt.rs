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
// Rust translation of GccAna_Circ2d2TanRad_5.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius through two points: the intersections of the circles of
//! that radius around them. Points just beyond the diameter (within the
//! tolerance) give the circle centred at their middle, as natively.
use super::Circ2d2TanRad;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2};

impl Circ2d2TanRad {
    /// GccAna_Circ2d2TanRad(Point1, Point2, Radius, Tolerance): circles of radius
    /// `radius` through two points; `Circ2d2TanRad::pnt_pnt(p1, p2, radius, tol)?`.
    pub fn pnt_pnt(
        point1: Point2,
        point2: Point2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        let tol: f64 = tolerance.abs();
        Self::check_radius(radius)?;
        if point1.distance(point2) - radius * 2.0 > tol {
            circ.well_done = true;
        } else if point1.distance(point2) - radius * 2.0 > 0.0 {
            circ.well_done = true;
            let middle = Point2::from(0.5 * (point1.xy() + point2.xy()));
            circ.store_through(middle, radius, (point1, point2))?;
        } else {
            let intp =
                AnaIntersection::circles(circle_at(point1, radius)?, circle_at(point2, radius)?)?;
            if intp.is_done() {
                if !intp.is_empty()? {
                    for i in 1..=intp.nb_points()? {
                        circ.store_through(intp.point(i)?.value(), radius, (point1, point2))?;
                    }
                }
                circ.well_done = true;
            }
        }
        for i in 1..=circ.nbr_sol {
            let solution: Circle2 = circ.cirsol.get(i)?;
            circ.tangency1.par_arg.set(i, 0.0)?;
            circ.tangency2.par_arg.set(i, 0.0)?;
            circ.tangency1
                .par_sol
                .set(i, elclib::parameter(&solution, point1)?)?;
            circ.tangency2
                .par_sol
                .set(i, elclib::parameter(&solution, point2)?)?;
        }
        Ok(circ)
    }

    fn store_through(
        &mut self,
        center: Point2,
        radius: f64,
        (point1, point2): (Point2, Point2),
    ) -> Result<(), Failure> {
        let index: i32 = self.push_circle(center, radius)?;
        self.set_qualifiers(index, Position::NoQualifier, Position::NoQualifier)?;
        self.tangency1.same.set(index, 0)?;
        self.tangency2.same.set(index, 0)?;
        self.tangency1.point.set(index, point1)?;
        self.tangency2.point.set(index, point2)
    }
}
