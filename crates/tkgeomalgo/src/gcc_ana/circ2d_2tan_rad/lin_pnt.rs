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
// Rust translation of GccAna_Circ2d2TanRad_3.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to a line and through a point: the
//! intersections of the parallels to the line at the radius with the circle of
//! the radius around the point.
//!
//! Native quirk kept: the single solution of a point just beyond the diameter
//! (within the tolerance) leaves the qualifier of the line unset (0 here,
//! uninitialized natively).
use super::{Circ2d2TanRad, NAME};
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::{Position, QualifiedLin, bad_qualifier};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

impl Circ2d2TanRad {
    /// GccAna_Circ2d2TanRad(Qualified1, Point2, Radius, Tolerance): circles of
    /// radius `radius` tangent to a qualified line and through a point, at most
    /// two; `Circ2d2TanRad::lin_pnt(l1, point, radius, tol)?`.
    pub fn lin_pnt(
        qualified1: QualifiedLin,
        point2: Point2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        let tol: f64 = tolerance.abs();
        if !(qualified1.is_enclosed() || qualified1.is_outside() || qualified1.is_unqualified()) {
            return Err(bad_qualifier(&format!(
                "{NAME}: line qualifier {:?}; expected enclosed, outside or unqualified",
                qualified1.qualifier()
            )));
        }
        let l1: Line2 = qualified1.qualified();
        let displ1: f64 = l1.distance(point2);
        let (xdir, ydir) = (l1.direction().x(), l1.direction().y());
        let (lxloc, lyloc) = (l1.location().x(), l1.location().y());
        let side: f64 = -ydir * (point2.x() - lxloc) + xdir * (point2.y() - lyloc);
        Self::check_radius(radius)?;
        let (nbsol, cote): (i32, &[f64]) = if displ1 - radius * 2.0 > tol {
            circ.well_done = true;
            (0, &[])
        } else if qualified1.is_enclosed() || qualified1.is_outside() {
            let refused: bool = if qualified1.is_enclosed() {
                side < 0.0
            } else {
                side > 0.0
            };
            let cote: &[f64] = if qualified1.is_enclosed() {
                &[1.0]
            } else {
                &[-1.0]
            };
            if refused {
                circ.well_done = true;
                (0, &[])
            } else if displ1 - radius * 2.0 > 0.0 {
                (2, cote)
            } else {
                (1, cote)
            }
        } else if displ1 - radius * 2.0 > 0.0 {
            if side > 0.0 {
                (2, &[1.0])
            } else if side < 0.0 {
                (2, &[-1.0])
            } else {
                (0, &[])
            }
        } else {
            (1, &[1.0, -1.0])
        };
        if nbsol == 1 {
            circ.near_point(qualified1, point2, radius, displ1, cote)?;
        } else if nbsol == 2 {
            let center = Point2::from(point2.xy() + cote[0] * radius * Xy::new(-ydir, xdir));
            circ.well_done = true;
            circ.nbr_sol = 1;
            circ.cirsol.set(1, circle_at(center, radius)?)?;
            circ.qualifier2.set(1, Position::NoQualifier)?;
            circ.tangency1.point.set(
                1,
                Point2::from(center.xy() + cote[0] * radius * Xy::new(ydir, -xdir)),
            )?;
            circ.tangency2.point.set(1, point2)?;
        }
        for i in 1..=circ.nbr_sol {
            let solution: Circle2 = circ.cirsol.get(i)?;
            let (p1, p2) = (circ.tangency1.point.get(i)?, circ.tangency2.point.get(i)?);
            circ.tangency1
                .par_sol
                .set(i, elclib::parameter(&solution, p1)?)?;
            circ.tangency1.par_arg.set(i, elclib::parameter(&l1, p1)?)?;
            circ.tangency2
                .par_sol
                .set(i, elclib::parameter(&solution, p2)?)?;
            circ.tangency2.par_arg.set(i, 0.0)?;
        }
        Ok(circ)
    }

    // nbsol 1: a point on the line gives the circles on both sides directly, any
    // other point the intersections (at most two solutions in all).
    fn near_point(
        &mut self,
        qualified1: QualifiedLin,
        point2: Point2,
        radius: f64,
        displ1: f64,
        cote: &[f64],
    ) -> Result<(), Failure> {
        let l1: Line2 = qualified1.qualified();
        let (xdir, ydir) = (l1.direction().x(), l1.direction().y());
        if displ1 < 1.0e-10 {
            for &c in cote {
                let center = Point2::new(
                    point2.x() - c * ydir * radius,
                    point2.y() + c * xdir * radius,
                );
                let index: i32 = self.push_circle(center, radius)?;
                let position = if !qualified1.is_unqualified() {
                    qualified1.qualifier()
                } else if c > 0.0 {
                    Position::Outside
                } else {
                    Position::Enclosed
                };
                self.set_qualifiers(index, position, Position::NoQualifier)?;
                self.tangency1.point.set(index, point2)?;
                self.tangency2.point.set(index, point2)?;
            }
            self.well_done = true;
            return Ok(());
        }
        let cirint: Circle2 = circle_at(point2, radius)?;
        let (lxloc, lyloc) = (l1.location().x(), l1.location().y());
        for &c in cote {
            let origin = Point2::new(lxloc - c * ydir * radius, lyloc + c * xdir * radius);
            let intp = AnaIntersection::line_circle(Line2::new(origin, l1.direction()), cirint)?;
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                let mut i: i32 = 1;
                while i <= intp.nb_points()? && self.nbr_sol < 2 {
                    self.parallel_solution(qualified1, point2, radius, c, intp.point(i)?.value())?;
                    i += 1;
                }
            }
            self.well_done = true;
        }
        Ok(())
    }

    fn parallel_solution(
        &mut self,
        qualified1: QualifiedLin,
        point2: Point2,
        radius: f64,
        c: f64,
        center: Point2,
    ) -> Result<(), Failure> {
        let l1: Line2 = qualified1.qualified();
        let (xdir, ydir) = (l1.direction().x(), l1.direction().y());
        let index: i32 = self.push_circle(center, radius)?;
        let dc1 = Direction2::from_xy(l1.location().xy() - center.xy())?;
        let position = if !qualified1.is_unqualified() {
            qualified1.qualifier()
        } else if dc1.dot(Direction2::new(-ydir, xdir)?) > 0.0 {
            Position::Outside
        } else {
            Position::Enclosed
        };
        self.set_qualifiers(index, position, Position::NoQualifier)?;
        self.tangency1.point.set(
            index,
            Point2::from(center.xy() + c * radius * Xy::new(ydir, -xdir)),
        )?;
        self.tangency2.point.set(index, point2)
    }
}
