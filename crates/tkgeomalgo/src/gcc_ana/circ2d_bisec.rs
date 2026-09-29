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
// Rust translation of GccAna_Circ2dBisec.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Circ2dBisec: the bisecting curves of two circles.
use super::bisector;
use super::solutions::{check_solution, not_done};
use crate::gcc_int::{BCirc, BElips, BHyper, BLine, Bisec};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{
    Axis2d, Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Point2, StandardDirection2,
};

/// GccAna_Circ2dBisec: the loci equidistant from two circles: circles, ellipses,
/// hyperbolas and lines depending on the relative position of the circles (the
/// larger circle is taken first). The solutions are computed when they are read.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Circ2dBisec;
/// use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Point2};
/// let at = |x: f64, r: f64| {
///     let axis = Axis2d::new(Point2::new(x, 0.0), Direction2::new(1.0, 0.0).unwrap());
///     Circle2::from_x_axis(axis, r, true).unwrap()
/// };
/// let bisec = Circ2dBisec::new(at(0.0, 1.0), at(5.0, 2.0));
/// assert_eq!(bisec.nb_solutions()?, 4);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2dBisec {
    well_done: bool,
    nbr_sol: i32,
    intersection: i32,
    sameradius: bool,
    circle1: Circle2,
    circle2: Circle2,
}

// The frame of the solutions: the middle of the centres, the X direction, the
// direction joining the centres and its normal.
struct BisecFrame {
    pcen: Point2,
    dircen: Direction2,
    medcen: Direction2,
    dist: f64,
    r1: f64,
    r2: f64,
}

impl Circ2dBisec {
    /// GccAna_Circ2dBisec(Circ1, Circ2): classify the circles with the tolerance
    /// Precision::Confusion; `Circ2dBisec::new(circ1, circ2)`.
    pub fn new(circ1: Circle2, circ2: Circle2) -> Self {
        let tol: f64 = precision::confusion();
        let sameradius: bool = (circ1.radius() - circ2.radius()).abs() <= tol;
        let (circle1, circle2) = if circ1.radius() < circ2.radius() {
            (circ2, circ1)
        } else {
            (circ1, circ2)
        };
        let (r1, r2) = (circle1.radius(), circle2.radius());
        let dist: f64 = circle2.location().distance(circle1.location());
        let (intersection, nbr_sol) = classify(r1, r2, dist, sameradius, tol);
        Self {
            well_done: true,
            nbr_sol,
            intersection,
            sameradius,
            circle1,
            circle2,
        }
    }

    /// IsDone: true; `bisec.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions: 0 to 4; `bisec.nb_solutions()?`.
    pub fn nb_solutions(&self) -> Result<i32, Failure> {
        if !self.well_done {
            return Err(not_done("GccAna_Circ2dBisec::NbSolutions"));
        }
        Ok(self.nbr_sol)
    }

    /// ThisSolution(Index): the bisecting curve `index`; a null handle for the
    /// index/configuration pairs that no native branch builds;
    /// `bisec.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Handle<dyn Bisec>, Failure> {
        let member = "GccAna_Circ2dBisec::ThisSolution";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        let frame = self.frame()?;
        match self.intersection {
            0 | 1 => self.nested(&frame, index),
            2 => self.secant(&frame, index),
            3 => self.touching_outside(&frame, index),
            _ => self.apart(&frame, index),
        }
    }

    fn frame(&self) -> Result<BisecFrame, Failure> {
        let tol: f64 = 1.0e-14;
        let (c1, c2) = (self.circle1.location(), self.circle2.location());
        let dist: f64 = c1.distance(c2);
        let pcen = Point2::new((c1.x() + c2.x()) / 2.0, (c1.y() + c2.y()) / 2.0);
        // gp_Dir2d default-constructs to +X; SetCoord only for distinct centres.
        let x = Direction2::from_standard(StandardDirection2::X);
        let (mut dircen, mut medcen) = (x, x);
        if dist > tol {
            dircen = Direction2::new(c2.x() - c1.x(), c2.y() - c1.y())?;
            medcen = Direction2::new(c2.y() - c1.y(), c1.x() - c2.x())?;
        }
        Ok(BisecFrame {
            pcen,
            dircen,
            medcen,
            dist,
            r1: self.circle1.radius(),
            r2: self.circle2.radius(),
        })
    }

    // intersection 0 (circle 2 strictly inside circle 1) and 1 (inside, tangent).
    fn nested(&self, f: &BisecFrame, index: i32) -> Result<Handle<dyn Bisec>, Failure> {
        let tol: f64 = 1.0e-14;
        let (r1, r2, acenx) = (f.r1, f.r2, x_axis(f.pcen));
        if self.nbr_sol == 1 && self.intersection == 0 {
            // Unreachable natively (intersection 0 always has 2 solutions); kept.
            let radius = if index == 1 {
                (r1 + r2) / 2.0
            } else {
                (r1 - r2) / 2.0
            };
            return circle(acenx, radius);
        }
        if self.nbr_sol == 2 && self.intersection == 1 {
            return match index {
                1 => ellipse(f, (r1 + r2) / 2.0, sum_minor(f)),
                2 => Ok(bisector(BLine::new(Line2::new(
                    self.circle1.location(),
                    f.dircen,
                )))),
                _ => Ok(Handle::default()),
            };
        }
        if !(self.nbr_sol == 2 && self.intersection == 0) {
            return Ok(Handle::default());
        }
        let (c1, c2) = (self.circle1.location(), self.circle2.location());
        let concentric = (c2.x() - c1.x()).abs() < tol && (c2.y() - c1.y()).abs() < tol;
        match (index, concentric) {
            (1, true) => circle(acenx, (r1 + r2) / 2.0),
            (1, false) => ellipse(f, (r1 + r2) / 2.0, sum_minor(f)),
            (2, true) => circle(Axis2d::new(f.pcen, f.dircen), (r1 - r2) / 2.0),
            (2, false) => ellipse(f, (r1 - r2) / 2.0, difference_minor(f)),
            _ => Ok(Handle::default()),
        }
    }

    // intersection 2: secant circles.
    fn secant(&self, f: &BisecFrame, index: i32) -> Result<Handle<dyn Bisec>, Failure> {
        let (r1, r2, dist) = (f.r1, f.r2, f.dist);
        if self.sameradius {
            return match index {
                1 => line(f.pcen, f.medcen),
                2 => ellipse(f, r1, (r1 * r1 - dist * dist / 4.0).sqrt()),
                _ => Ok(Handle::default()),
            };
        }
        match index {
            1 => hyperbola(f, (r1 - r2) / 2.0, difference_focal(f), false),
            2 => hyperbola(f, (r1 - r2) / 2.0, difference_focal(f), true),
            3 => ellipse(f, (r1 + r2) / 2.0, sum_minor(f)),
            _ => Ok(Handle::default()),
        }
    }

    // intersection 3: circles tangent outside each other.
    fn touching_outside(&self, f: &BisecFrame, index: i32) -> Result<Handle<dyn Bisec>, Failure> {
        let (r1, r2) = (f.r1, f.r2);
        match (index, self.sameradius) {
            (1, _) => line(f.pcen, f.dircen),
            (2, true) => line(f.pcen, f.medcen),
            (2, false) => hyperbola(f, (r1 - r2) / 2.0, difference_focal(f), false),
            (3, false) => hyperbola(f, (r1 - r2) / 2.0, difference_focal(f), true),
            _ => Ok(Handle::default()),
        }
    }

    // intersection 4: circles outside each other.
    fn apart(&self, f: &BisecFrame, index: i32) -> Result<Handle<dyn Bisec>, Failure> {
        let (r1, r2, dist) = (f.r1, f.r2, f.dist);
        if self.sameradius {
            let minor: f64 = (dist * dist - 4.0 * r1 * r1).sqrt() / 2.0;
            return match index {
                1 => line(f.pcen, f.medcen),
                2 => hyperbola(f, r1, minor, false),
                3 => hyperbola(f, r1, minor, true),
                _ => Ok(Handle::default()),
            };
        }
        let sum_focal: f64 = (dist * dist - (r1 + r2) * (r1 + r2)).sqrt() / 2.0;
        match index {
            1 => hyperbola(f, (r1 - r2) / 2.0, difference_focal(f), false),
            2 => hyperbola(f, (r1 - r2) / 2.0, difference_focal(f), true),
            3 => hyperbola(f, (r1 + r2) / 2.0, sum_focal, false),
            4 => hyperbola(f, (r1 + r2) / 2.0, sum_focal, true),
            _ => Ok(Handle::default()),
        }
    }
}

// The relative position of the circles (intersection code) and the number of
// bisectors, as the native constructor sets them.
fn classify(r1: f64, r2: f64, dist: f64, sameradius: bool, tol: f64) -> (i32, i32) {
    if r1 - dist - r2 > tol {
        return (0, 2);
    }
    if (r1 - dist - r2).abs() <= tol {
        return (1, if sameradius { 0 } else { 2 });
    }
    if (dist + r2 - r1 > tol) && (r1 - dist + r2 > tol) {
        return (2, if sameradius { 2 } else { 3 });
    }
    if (r1 - dist + r2).abs() <= tol {
        return (3, if sameradius { 2 } else { 3 });
    }
    (4, if sameradius { 3 } else { 4 })
}

// std::sqrt((R1 * R1 + R2 * R2 - dist * dist) / 4. + R1 * R2 / 2.)
fn sum_minor(f: &BisecFrame) -> f64 {
    ((f.r1 * f.r1 + f.r2 * f.r2 - f.dist * f.dist) / 4.0 + f.r1 * f.r2 / 2.0).sqrt()
}

// std::sqrt((R1 * R1 + R2 * R2 - dist * dist) / 4. - R1 * R2 / 2.)
fn difference_minor(f: &BisecFrame) -> f64 {
    ((f.r1 * f.r1 + f.r2 * f.r2 - f.dist * f.dist) / 4.0 - f.r1 * f.r2 / 2.0).sqrt()
}

// std::sqrt(dist * dist - (R1 - R2) * (R1 - R2)) / 2.0
fn difference_focal(f: &BisecFrame) -> f64 {
    (f.dist * f.dist - (f.r1 - f.r2) * (f.r1 - f.r2)).sqrt() / 2.0
}

fn x_axis(pcen: Point2) -> Axis2d {
    Axis2d::new(pcen, Direction2::from_standard(StandardDirection2::X))
}

fn circle(axis: Axis2d, radius: f64) -> Result<Handle<dyn Bisec>, Failure> {
    Ok(bisector(BCirc::new(Circle2::from_x_axis(
        axis, radius, true,
    )?)))
}

fn line(location: Point2, direction: Direction2) -> Result<Handle<dyn Bisec>, Failure> {
    Ok(bisector(BLine::new(Line2::new(location, direction))))
}

// gp_Elips2d(acencen, major, minor).
fn ellipse(f: &BisecFrame, major: f64, minor: f64) -> Result<Handle<dyn Bisec>, Failure> {
    let acencen = Axis2d::new(f.pcen, f.dircen);
    Ok(bisector(BElips::new(Ellipse2::from_x_axis(
        acencen, major, minor, true,
    )?)))
}

// gp_Hypr2d(acencen, major, minor), or its other branch.
fn hyperbola(
    f: &BisecFrame,
    major: f64,
    minor: f64,
    other_branch: bool,
) -> Result<Handle<dyn Bisec>, Failure> {
    let acencen = Axis2d::new(f.pcen, f.dircen);
    let h1 = Hyperbola2::from_x_axis(acencen, major, minor, true)?;
    let branch = if other_branch { h1.other_branch() } else { h1 };
    Ok(bisector(BHyper::new(branch)))
}
