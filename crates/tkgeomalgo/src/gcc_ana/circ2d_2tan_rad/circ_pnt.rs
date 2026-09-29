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
// Rust translation of GccAna_Circ2d2TanRad_2.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to a circle and through a point: the
//! intersections of a circle concentric to C1 with the circle of the radius around
//! the point, or a single circle on the line from C1 to the point.
//!
//! Native quirks kept: the direction from C1 to the point is built before the
//! radius test (a point at the centre raises Standard_ConstructionError), and an
//! unqualified circle of the given radius through the point gives C1 itself and
//! then the intersection solutions after it.
use super::circ_circ::ring_tangency;
use super::{Circ2d2TanRad, NAME};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::{circle_at, qualify};
use crate::gcc_ent::{Position, QualifiedCirc};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Point2};

// What the qualifier analysis settled.
enum Plan {
    None,
    /// nbsol < 0: one circle on the line of centres (deport, signe).
    Aligned(f64, f64),
    /// nbsol > 0: the pairs of circles to intersect.
    Rings(Vec<(Circle2, Circle2)>),
}

impl Circ2d2TanRad {
    /// GccAna_Circ2d2TanRad(Qualified1, Point2, Radius, Tolerance): circles of
    /// radius `radius` tangent to a qualified circle and through a point;
    /// `Circ2d2TanRad::circ_pnt(q1, point, radius, tol)?`.
    pub fn circ_pnt(
        qualified1: QualifiedCirc,
        point2: Point2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        let tol: f64 = tolerance.abs();
        check_qualifier(qualified1.qualifier(), NAME)?;
        let c1: Circle2 = qualified1.qualified();
        let dir1 = Direction2::from_xy(point2.xy() - c1.location().xy())?;
        Self::check_radius(radius)?;
        match circ.plan_circ_pnt(qualified1, point2, radius, tol, dir1)? {
            Plan::None => {}
            Plan::Aligned(deport, signe) => {
                circ.aligned(qualified1, point2, radius, tol, (dir1, deport, signe))?
            }
            Plan::Rings(pairs) => {
                for (first, second) in pairs {
                    circ.rings_through_point(qualified1, point2, radius, tol, (first, second))?;
                }
            }
        }
        for i in 1..=circ.nbr_sol {
            let solution: Circle2 = circ.cirsol.get(i)?;
            let (p1, p2) = (circ.tangency1.point.get(i)?, circ.tangency2.point.get(i)?);
            circ.tangency1
                .par_sol
                .set(i, elclib::parameter(&solution, p1)?)?;
            if circ.tangency1.same.get(i)? == 0 {
                circ.tangency1.par_arg.set(i, elclib::parameter(&c1, p1)?)?;
            }
            circ.tangency2
                .par_sol
                .set(i, elclib::parameter(&solution, p2)?)?;
            circ.tangency2.par_arg.set(i, 0.0)?;
        }
        Ok(circ)
    }

    fn plan_circ_pnt(
        &mut self,
        q1: QualifiedCirc,
        point2: Point2,
        radius: f64,
        tol: f64,
        dir1: Direction2,
    ) -> Result<Plan, Failure> {
        let c1: Circle2 = q1.qualified();
        let r1: f64 = c1.radius();
        let distance: f64 = c1.location().distance(point2);
        let dispc1: f64 = c1.distance(point2);
        let concentric = |r: f64| Circle2::from_x_axis(c1.x_axis(), r, true);
        let around = || circle_at(point2, radius);
        if dispc1 - radius * 2.0 > tol {
            self.well_done = true;
            return Ok(Plan::None);
        }
        if q1.is_enclosed() || q1.is_enclosing() {
            let refused: bool = if q1.is_enclosed() {
                (distance - r1 > tol) || (radius - r1 > tol)
            } else {
                (tol < r1 - distance) || (tol < r1 - radius)
            };
            if refused {
                self.well_done = true;
                return Ok(Plan::None);
            }
            if (distance - r1).abs() < tol {
                return Ok(Plan::Aligned(r1 - radius, 1.0));
            }
            return Ok(Plan::Rings(vec![(
                concentric((radius - r1).abs())?,
                around()?,
            )]));
        }
        if q1.is_outside() {
            if tol < r1 - distance {
                self.well_done = true;
                return Ok(Plan::None);
            }
            if ((distance - r1).abs() < tol) || ((dispc1 - radius * 2.0).abs() < tol) {
                return Ok(Plan::Aligned(r1 + radius, -1.0));
            }
            return Ok(Plan::Rings(vec![(concentric(radius + r1)?, around()?)]));
        }
        if (dispc1 - radius * 2.0).abs() < tol {
            self.diametral(c1, point2, radius, tol, (dir1, distance))?;
            return Ok(Plan::None);
        }
        if ((r1 - radius).abs() < tol) && ((distance - r1).abs() < tol) {
            self.cirsol.set(1, c1)?;
            self.set_qualifiers(1, Position::Unqualified, Position::NoQualifier)?;
            self.tangency1.same.set(1, 1)?;
            self.tangency2.point.set(1, point2)?;
            self.well_done = true;
            self.nbr_sol = 1;
            return Ok(Plan::Rings(vec![(concentric(radius + r1)?, around()?)]));
        }
        Ok(Plan::Rings(vec![
            (concentric((radius - r1).abs())?, around()?),
            (concentric(radius + r1)?, around()?),
        ]))
    }

    // An unqualified circle at a diameter from the point: the circle between them.
    fn diametral(
        &mut self,
        c1: Circle2,
        point2: Point2,
        radius: f64,
        tol: f64,
        (dir1, distance): (Direction2, f64),
    ) -> Result<(), Failure> {
        self.well_done = true;
        let center = Point2::from(c1.location().xy() + (distance - radius) * dir1.xy());
        self.cirsol.set(1, circle_at(center, radius)?)?;
        let q1 = if (center.distance(c1.location()) - c1.radius()).abs() < tol {
            Position::Enclosed
        } else {
            Position::Outside
        };
        self.set_qualifiers(1, q1, Position::NoQualifier)?;
        self.tangency1
            .point
            .set(1, Point2::from(center.xy() - radius * dir1.xy()))?;
        self.tangency2.point.set(1, point2)?;
        self.nbr_sol = 1;
        Ok(())
    }

    // nbsol < 0: the circle at `deport` from C1 towards the point.
    fn aligned(
        &mut self,
        q1: QualifiedCirc,
        point2: Point2,
        radius: f64,
        tol: f64,
        (dir1, deport, signe): (Direction2, f64, f64),
    ) -> Result<(), Failure> {
        let c1: Circle2 = q1.qualified();
        let center = Point2::from(c1.location().xy() + deport * dir1.xy());
        self.cirsol.set(1, circle_at(center, radius)?)?;
        self.set_qualifiers(1, q1.qualifier(), Position::NoQualifier)?;
        if deport.abs() <= tol && (radius - c1.radius()).abs() <= tol {
            self.tangency1.same.set(1, 1)?;
        } else {
            self.tangency1
                .point
                .set(1, Point2::from(center.xy() + signe * radius * dir1.xy()))?;
        }
        self.tangency2.point.set(1, point2)?;
        self.well_done = true;
        self.nbr_sol = 1;
        Ok(())
    }

    fn rings_through_point(
        &mut self,
        q1: QualifiedCirc,
        point2: Point2,
        radius: f64,
        tol: f64,
        (first, second): (Circle2, Circle2),
    ) -> Result<(), Failure> {
        let c1: Circle2 = q1.qualified();
        let intp = AnaIntersection::circles(first, second)?;
        if !intp.is_done() {
            return Ok(());
        }
        if !intp.is_empty()? {
            for i in 1..=intp.nb_points()? {
                let center: Point2 = intp.point(i)?.value();
                let index: i32 = self.push_circle(center, radius)?;
                let distcc1: f64 = c1.location().distance(center);
                let position = qualify(q1.qualifier(), distcc1, radius, c1.radius(), tol, false);
                self.set_qualifiers(index, position, Position::NoQualifier)?;
                let dir1 = Direction2::from_xy(center.xy() - c1.location().xy())?;
                let p1: Point2 = ring_tangency(center, c1.location(), c1.radius(), radius, dir1);
                self.tangency1.point.set(index, p1)?;
                self.tangency2.point.set(index, point2)?;
            }
        }
        self.well_done = true;
        Ok(())
    }
}
