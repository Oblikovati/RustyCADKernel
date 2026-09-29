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
// Rust translation of GccAna_Circ2d2TanOn_5.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles through two points with their centre on a line: the intersection of
//! the locus with the perpendicular bisector of the points.
use super::Circ2d2TanOn;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Point1, Point2, OnLine, Tolerance): the circle through
    /// two points centred on `on_line` (none for points closer than |Tolerance|);
    /// `Circ2d2TanOn::pnt_pnt_on_line(p1, p2, line, tol)?`.
    pub fn pnt_pnt_on_line(
        point1: Point2,
        point2: Point2,
        on_line: Line2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        if point1.distance(point2) < tolerance.abs() {
            circ.well_done = true;
            return Ok(circ);
        }
        let middle = Point2::from((point1.xy() + point2.xy()) / 2.0);
        let l1 = Line2::new(
            middle,
            Direction2::new(point1.y() - point2.y(), point2.x() - point1.x())?,
        );
        let intp = AnaIntersection::lines(l1, on_line);
        if !intp.is_done() {
            return Ok(circ);
        }
        if !intp.is_empty()? {
            for i in 1..=intp.nb_points()? {
                let center: Point2 = intp.point(i)?.value();
                circ.through_both(center, (point1, point2), &on_line_locus(on_line))?;
            }
        }
        circ.well_done = true;
        Ok(circ)
    }

    // A circle centred at `center` through both points, of centre parameter on the
    // locus given by `locus`.
    pub(super) fn through_both(
        &mut self,
        center: Point2,
        (point1, point2): (Point2, Point2),
        locus: &dyn Fn(Point2) -> Result<f64, Failure>,
    ) -> Result<(), Failure> {
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, point1.distance(center))?;
        self.cirsol.set(index, solution)?;
        self.set_qualifiers(index, Position::NoQualifier, Position::NoQualifier)?;
        let centre: Point2 = solution.location();
        self.tangency1
            .record(index, point1, elclib::parameter(&solution, point1)?, 0.0)?;
        self.tangency2
            .record(index, point2, elclib::parameter(&solution, point2)?, 0.0)?;
        self.centre_on(index, centre, locus(centre)?)
    }
}

// ElCLib::Parameter(OnLine, centre).
fn on_line_locus(on_line: Line2) -> impl Fn(Point2) -> Result<f64, Failure> {
    move |centre: Point2| elclib::parameter(&on_line, centre)
}
