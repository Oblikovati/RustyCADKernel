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
// Rust translation of GccAna_CircPnt2dBisec.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_CircPnt2dBisec: the bisecting curves of a circle and a point.
use super::bisector;
use super::solutions::check_solution;
use crate::gcc_int::{BCirc, BElips, BHyper, BLine, Bisec};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{
    Axis2d, Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Point2, StandardDirection2,
};

/// GccAna_CircPnt2dBisec: the loci equidistant from a circle and a point: an
/// ellipse for a point inside the circle, the line through the point and the centre
/// for a point on it (within the tolerance), the two branches of a hyperbola for a
/// point outside, and a circle for a point at the centre.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::CircPnt2dBisec;
/// use rustycad_tkmath::gp::{Circle2, Point2};
/// let bisec = CircPnt2dBisec::new(Circle2::default(), Point2::default());
/// assert_eq!(bisec.nb_solutions(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct CircPnt2dBisec {
    well_done: bool,
    nbr_sol: i32,
    circle: Circle2,
    point: Point2,
    theposition: i32,
    my_tolerance: f64,
}

impl CircPnt2dBisec {
    /// GccAna_CircPnt2dBisec(Circle1, Point2) with the tolerance 1e-10;
    /// `CircPnt2dBisec::new(circle, point)`.
    pub fn new(circle: Circle2, point: Point2) -> Self {
        Self::define_solutions(circle, point, 1.0e-10)
    }

    /// GccAna_CircPnt2dBisec(Circle1, Point2, Tolerance): the tolerance is the
    /// larger of 1e-10 and `tolerance`; `CircPnt2dBisec::with_tolerance(c, p, 1e-7)`.
    pub fn with_tolerance(circle: Circle2, point: Point2, tolerance: f64) -> Self {
        let mut my_tolerance: f64 = 1.0e-10;
        if my_tolerance < tolerance {
            my_tolerance = tolerance;
        }
        Self::define_solutions(circle, point, my_tolerance)
    }

    // DefineSolutions: classify the point as on (0), inside (-1) or outside (1).
    fn define_solutions(circle: Circle2, point: Point2, my_tolerance: f64) -> Self {
        let dist: f64 = circle.radius() - point.distance(circle.location());
        let (theposition, nbr_sol) = if dist.abs() < my_tolerance {
            (0, 1)
        } else if dist > 0.0 {
            (-1, 1)
        } else {
            (1, 2)
        };
        Self {
            well_done: true,
            nbr_sol,
            circle,
            point,
            theposition,
            my_tolerance,
        }
    }

    /// IsDone: true; `bisec.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions: 1, or 2 for a point outside the circle; `bisec.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// ThisSolution(Index): the bisecting curve `index`; `bisec.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Handle<dyn Bisec>, Failure> {
        let member = "GccAna_CircPnt2dBisec::ThisSolution";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        let r1: f64 = self.circle.radius();
        let dist: f64 = self.point.distance(self.circle.location());
        if dist < self.my_tolerance {
            let axis = Axis2d::new(self.point, Direction2::from_standard(StandardDirection2::X));
            let biscirpnt1 = Circle2::from_x_axis(axis, r1 / 2.0, true)?;
            return Ok(bisector(BCirc::new(biscirpnt1)));
        }
        let (xpoint, ypoint) = (self.point.x(), self.point.y());
        let centre: Point2 = self.circle.location();
        let (xcencir, ycencir) = (centre.x(), centre.y());
        let center = Point2::new((xpoint + xcencir) / 2.0, (ypoint + ycencir) / 2.0);
        let majax = Axis2d::new(center, Direction2::new(xpoint - xcencir, ypoint - ycencir)?);
        match self.theposition {
            // The point is inside the circle.
            -1 => {
                let minor: f64 = (r1 * r1 - dist * dist).sqrt() / 2.0;
                let ellipse = Ellipse2::from_x_axis(majax, r1 / 2.0, minor, true)?;
                Ok(bisector(BElips::new(ellipse)))
            }
            // The point is on the circle: the line through the point and the centre.
            0 => Ok(bisector(BLine::new(self.radial_line()?))),
            // The point is outside: the two main branches of a hyperbola.
            _ => {
                let d1: f64 = (dist * dist - r1 * r1).sqrt() / 2.0;
                let biscirpnt1 = Hyperbola2::from_x_axis(majax, r1 / 2.0, d1, true)?;
                let branch = if index == 1 {
                    biscirpnt1
                } else {
                    biscirpnt1.other_branch()
                };
                Ok(bisector(BHyper::new(branch)))
            }
        }
    }

    // The line through the point towards the centre (away from it for an indirect
    // circle).
    fn radial_line(&self) -> Result<Line2, Failure> {
        let (xpoint, ypoint) = (self.point.x(), self.point.y());
        let centre: Point2 = self.circle.location();
        let dirsol = if self.circle.is_direct() {
            Direction2::new(centre.x() - xpoint, centre.y() - ypoint)?
        } else {
            Direction2::new(xpoint - centre.x(), ypoint - centre.y())?
        };
        Ok(Line2::new(self.point, dirsol))
    }
}
