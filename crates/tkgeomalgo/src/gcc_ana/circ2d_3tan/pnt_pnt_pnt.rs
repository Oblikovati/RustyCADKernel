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
// Rust translation of GccAna_Circ2d3Tan_9.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circle through three points: the intersection of two perpendicular
//! bisectors, a null circle for three coincident points.
use super::{Argument, Circ2d3Tan};
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

// The perpendicular bisector of (a, b): through their middle, of direction
// (a.y - b.y, b.x - a.x).
fn bisector(a: Point2, b: Point2) -> Result<Line2, Failure> {
    let middle = Point2::from((a.xy() + b.xy()) / 2.0);
    Ok(Line2::new(
        middle,
        Direction2::new(a.y() - b.y(), b.x() - a.x())?,
    ))
}

// The line through `a` normal to (a, b), used when another pair coincides.
fn normal_through(a: Point2, b: Point2) -> Result<Line2, Failure> {
    Ok(Line2::new(
        a,
        Direction2::new(a.y() - b.y(), b.x() - a.x())?,
    ))
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Point1, Point2, Point3, Tolerance): the circle through
    /// three points; `Circ2d3Tan::pnt_pnt_pnt(p1, p2, p3, tol)?`.
    pub fn pnt_pnt_pnt(
        point1: Point2,
        point2: Point2,
        point3: Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(1);
        let dist1: f64 = point1.distance(point2);
        let dist2: f64 = point1.distance(point3);
        let dist3: f64 = point2.distance(point3);
        for which in 1..=3 {
            circ.set_qualifier(which, 1, Position::NoQualifier)?;
        }
        let points: [Point2; 3] = [point1, point2, point3];
        if (dist1 < tolerance) && (dist2 < tolerance) && (dist3 < tolerance) {
            circ.nbr_sol += 1;
            circ.well_done = true;
            circ.cirsol.set(1, circle_at(point1, 0.0)?)?;
            for (n, point) in points.iter().enumerate() {
                circ.tangency_mut(n + 1).record(1, *point, 0.0, 0.0)?;
            }
            return Ok(circ);
        }
        let (mut l1, mut l2) = (Line2::default(), Line2::default());
        if dist1 >= tolerance {
            l1 = bisector(point1, point2)?;
        }
        if dist2 >= tolerance {
            l2 = bisector(point1, point3)?;
        }
        if dist2 <= tolerance {
            l2 = normal_through(point1, point2)?;
        } else if dist1 <= tolerance {
            l1 = normal_through(point1, point3)?;
        } else if dist3 <= tolerance {
            l2 = normal_through(point1, point2)?;
        }
        let intp = AnaIntersection::lines(l1, l2);
        if !intp.is_done() {
            return Ok(circ);
        }
        if !intp.is_empty()? {
            for i in 1..=intp.nb_points()? {
                let center: Point2 = intp.point(i)?.value();
                let index: i32 = circ.push_circle(center, point1.distance(center))?;
                for (n, point) in points.iter().enumerate() {
                    circ.touch(n + 1, index, *point, Argument::Point)?;
                }
            }
        }
        circ.well_done = true;
        Ok(circ)
    }
}
