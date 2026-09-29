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
// Rust translation of GccAna_Pnt2dBisec.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Pnt2dBisec: the bisecting line of two points.
use super::no_solution;
use super::solutions::not_done;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

/// GccAna_Pnt2dBisec: the perpendicular bisector of the segment joining two points;
/// no solution when the points are closer than 1e-10.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Pnt2dBisec;
/// use rustycad_tkmath::gp::Point2;
/// let bisec = Pnt2dBisec::new(Point2::new(0.0, 0.0), Point2::new(2.0, 0.0))?;
/// assert_eq!(bisec.this_solution()?.location().x(), 1.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Pnt2dBisec {
    well_done: bool,
    has_sol: bool,
    linsol: Line2,
}

impl Pnt2dBisec {
    /// GccAna_Pnt2dBisec(Point1, Point2). The native test is `Distance > 1.e-10`
    /// (a former `gp::Resolution()` test is commented out natively);
    /// `Pnt2dBisec::new(p1, p2)?`.
    pub fn new(point1: Point2, point2: Point2) -> Result<Self, Failure> {
        if point1.distance(point2) > 1.0e-10 {
            let dir1 = Direction2::from_xy(point2.xy() - point1.xy())?;
            let middle = Point2::new(
                (point2.x() + point1.x()) / 2.0,
                (point2.y() + point1.y()) / 2.0,
            );
            let linsol = Line2::new(middle, Direction2::new(-dir1.y(), dir1.x())?);
            return Ok(Self {
                well_done: true,
                has_sol: true,
                linsol,
            });
        }
        Ok(Self {
            well_done: true,
            has_sol: false,
            linsol: Line2::default(),
        })
    }

    /// IsDone: always true; `bisec.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// HasSolution: false for coincident points; `bisec.has_solution()`.
    pub fn has_solution(&self) -> bool {
        self.has_sol
    }

    /// ThisSolution: the bisecting line; GccAna_NoSolution for coincident points;
    /// `bisec.this_solution()?`.
    pub fn this_solution(&self) -> Result<Line2, Failure> {
        if !self.well_done {
            return Err(not_done("GccAna_Pnt2dBisec::ThisSolution"));
        }
        if !self.has_sol {
            return Err(no_solution(
                "GccAna_Pnt2dBisec::ThisSolution: the points are coincident (distance <= 1e-10)",
            ));
        }
        Ok(self.linsol)
    }
}
