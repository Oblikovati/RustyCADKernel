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
// Rust translation of GccAna_LinPnt2dBisec.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_LinPnt2dBisec: the bisecting curve of a line and a point.
use super::bisector;
use super::solutions::not_done;
use crate::gcc_int::{BLine, BParab, Bisec};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis2d, Direction2, Line2, Parabola2, Point2, Xy};

/// GccAna_LinPnt2dBisec: the locus equidistant from a line and a point: a parabola
/// of focus the point, or the normal line through the point when it lies on the line
/// (within 1e-10). There is always exactly one solution.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::LinPnt2dBisec;
/// use rustycad_tkgeomalgo::gcc_int::IType;
/// use rustycad_tkmath::gp::{Line2, Point2};
/// let bisec = LinPnt2dBisec::new(Line2::default(), Point2::new(0.0, 2.0))?;
/// assert_eq!(bisec.this_solution()?.arc_type(), IType::Par);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct LinPnt2dBisec {
    well_done: bool,
    bissol: Handle<dyn Bisec>,
}

impl LinPnt2dBisec {
    /// GccAna_LinPnt2dBisec(Line1, Point2); `LinPnt2dBisec::new(line, point)?`.
    pub fn new(line1: Line2, point2: Point2) -> Result<Self, Failure> {
        let (xdir, ydir) = (line1.direction().x(), line1.direction().y());
        let (xloc, yloc) = (line1.location().x(), line1.location().y());
        let dist: f64 = line1.distance(point2);
        // A former gp::Resolution() test is commented out natively.
        if dist > 1.0e-10 {
            let axeparab = parabola_axis(point2, (xdir, ydir), (xloc, yloc), dist)?;
            let bislinpnt = Parabola2::from_mirror_axis(axeparab, dist / 2.0, true)?;
            return Ok(Self::done(bisector(BParab::new(bislinpnt))));
        }
        let bislinpnt = Line2::new(point2, Direction2::new(-ydir, xdir)?);
        Ok(Self::done(bisector(BLine::new(bislinpnt))))
    }

    fn done(bissol: Handle<dyn Bisec>) -> Self {
        Self {
            well_done: true,
            bissol,
        }
    }

    /// IsDone: true; `bisec.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// ThisSolution: the bisecting curve; `bisec.this_solution()?.arc_type()`.
    pub fn this_solution(&self) -> Result<Handle<dyn Bisec>, Failure> {
        if !self.well_done {
            return Err(not_done("GccAna_LinPnt2dBisec::ThisSolution"));
        }
        Ok(self.bissol.clone())
    }
}

// The mirror axis of the bisecting parabola: its apex halfway between the point and
// the line, pointing away from the line.
fn parabola_axis(
    point2: Point2,
    (xdir, ydir): (f64, f64),
    (xloc, yloc): (f64, f64),
    dist: f64,
) -> Result<Axis2d, Failure> {
    let (xpoint2, ypoint2) = (point2.x(), point2.y());
    let normal = Xy::new(-ydir, xdir);
    if (-ydir * (xpoint2 - xloc) + xdir * (ypoint2 - yloc)) > 0.0 {
        let apex = Point2::from(point2.xy() - dist / 2.0 * normal);
        return Ok(Axis2d::new(apex, Direction2::new(-ydir, xdir)?));
    }
    let apex = Point2::from(point2.xy() + dist / 2.0 * normal);
    Ok(Axis2d::new(apex, Direction2::new(ydir, -xdir)?))
}
