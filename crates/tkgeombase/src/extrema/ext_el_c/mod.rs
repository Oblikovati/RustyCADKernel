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
// Rust translation of Extrema_ExtElC.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtElC`: the extrema between two elementary 3D curves in closed form
//! (a line and a line or conic, two coplanar circles).
mod line_conics;
mod trig_roots;

use super::pair_result::CurvePairExtrema;
use super::pon::POnCurv;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Circle3, Direction3, Line3, Plane3, Point3, Vector3, Xyz};
use std::ops::Deref;

/// `Extrema_ExtElC`: one constructor per curve pair; the accessors (`IsDone`,
/// `IsParallel`, `NbExt`, `SquareDistance`, `Points`) are those of
/// `CurvePairExtrema`, reached through `Deref`;
/// `ExtElC::lines(line1, line2, 1e-10).nb_ext()? == 1`.
#[derive(Clone, Debug)]
pub struct ExtElC {
    result: CurvePairExtrema<Point3, 6>,
}

impl Default for ExtElC {
    fn default() -> Self {
        Self {
            result: CurvePairExtrema::new("Extrema_ExtElC"),
        }
    }
}

impl Deref for ExtElC {
    type Target = CurvePairExtrema<Point3, 6>;
    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

impl ExtElC {
    /// `Extrema_ExtElC()`: not done; `ExtElC::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtElC(gp_Lin, gp_Lin, AngTol)`: the common perpendicular, solving
    /// `u1 - (D1.D2) u2 = L.D1`, `(D1.D2) u1 - u2 = L.D2`; parallel lines (square sine
    /// below gp::Resolution(), directions parallel within Precision::Angular(), or an
    /// infinite solution) give one extremum without points. The tolerance is unused;
    /// `ExtElC::lines(line1, line2, 1e-10)`.
    pub fn lines(first: Line3, second: Line3, _ang_tol: f64) -> Self {
        let mut extrema: Self = Self::new();
        let Some((u1, u2)) = perpendicular_parameters(&first, &second) else {
            let sq_dist: f64 = second.square_distance(first.location());
            extrema.result.set_parallel(sq_dist);
            return extrema;
        };
        let p1: Point3 = elclib::value(u1, &first);
        let p2: Point3 = elclib::value(u2, &second);
        let sq_dist: f64 = p1.square_distance(&p2);
        extrema
            .result
            .push(sq_dist, POnCurv::new(u1, p1), POnCurv::new(u2, p2));
        extrema.result.done = true;
        extrema
    }
}

// The parameters of the common perpendicular, or None (myIsPar) for parallel lines
// or an infinite solution.
fn perpendicular_parameters(first: &Line3, second: &Line3) -> Option<(f64, f64)> {
    let (d1, d2) = (first.direction(), second.direction());
    let cos_a: f64 = d1.dot(d2);
    let sq_sin_a: f64 = 1.0 - cos_a * cos_a;
    if sq_sin_a < gp::resolution() || d1.is_parallel(d2, precision::angular()) {
        return None;
    }
    let l1l2: Xyz = second.location().xyz() - first.location().xyz();
    let (d1l, d2l) = (d1.xyz().dot(l1l2), d2.xyz().dot(l1l2));
    let u1: f64 = (d1l - cos_a * d2l) / sq_sin_a;
    let u2: f64 = (cos_a * d1l - d2l) / sq_sin_a;
    if precision::is_infinite(u1) || precision::is_infinite(u2) {
        return None;
    }
    Some((u1, u2))
}

impl ExtElC {
    /// `Extrema_ExtElC(gp_Circ, gp_Circ)`: only coplanar circles (parallel axes within
    /// Precision::Angular(), centre within Precision::Confusion() of the first plane);
    /// concentric circles are parallel with the square radius difference; otherwise
    /// the four points on the line of centres, plus the one or two intersection
    /// points (square distance 0) when the circles are neither apart nor nested;
    /// `ExtElC::circles(circle1, circle2)?`.
    pub fn circles(first: Circle3, second: Circle3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let (tol_d, tol_a) = (precision::confusion(), precision::angular());
        let (pc1, dc1) = (first.location(), first.axis().direction());
        let (pc2, dc2) = (second.location(), second.axis().direction());
        let sq_offset: f64 = Plane3::new(pc1, dc1)?.square_distance(pc2);
        if !(dc1.is_parallel(dc2, tol_a) && sq_offset < tol_d * tol_d) {
            return Ok(extrema);
        }
        if pc1.square_distance(&pc2) < tol_d * tol_d {
            let dr: f64 = first.radius() - second.radius();
            extrema.result.set_parallel(dr * dr);
            return Ok(extrema);
        }
        extrema.result.done = true;
        // j1/j2: the larger circle comes first in the computation.
        let swapped: bool = second.radius() > first.radius();
        let (c1, c2) = if swapped {
            (second, first)
        } else {
            (first, second)
        };
        extrema.coplanar_circles(&c1, &c2, dc1, swapped)?;
        Ok(extrema)
    }

    // The solutions of two coplanar, not concentric circles, `c1` the larger.
    fn coplanar_circles(
        &mut self,
        c1: &Circle3,
        c2: &Circle3,
        dc1: Direction3,
        swapped: bool,
    ) -> Result<(), Failure> {
        let (r1, r2) = (c1.radius(), c2.radius());
        let (pc1, pc2) = (c1.location(), c2.location());
        let d12: f64 = pc1.distance(&pc2);
        let dir12: Direction3 = Direction3::from_vector(Vector3::between(pc1, pc2))?;
        let axis_point = |center: Point3, r: f64| Point3::from(center.xyz() + dir12.xyz() * r);
        let (p11, p12) = (axis_point(pc1, -r1), axis_point(pc1, r1));
        let (p21, p22) = (axis_point(pc2, -r2), axis_point(pc2, r2));
        let (t11, t12) = (elclib::parameter(c1, p11)?, elclib::parameter(c1, p12)?);
        let (t21, t22) = (elclib::parameter(c2, p21)?, elclib::parameter(c2, p22)?);
        let pairs = [
            ((t11, p11), (t21, p21)),
            ((t11, p11), (t22, p22)),
            ((t12, p12), (t21, p21)),
            ((t12, p12), (t22, p22)),
        ];
        for ((ta, pa), (tb, pb)) in pairs {
            self.push_ordered(pa.square_distance(&pb), (ta, pa), (tb, pb), swapped);
        }
        let tol_d: f64 = precision::confusion();
        let outside: bool = d12 > (r1 + r2 + tol_d);
        let inside: bool = d12 < (r1 - r2 - tol_d);
        if !outside && !inside {
            self.intersection_points(c1, c2, (dc1, dir12), d12, swapped)?;
        }
        Ok(())
    }

    // `myPoint[k][j1]` gets the point on the larger circle, `[j2]` the other one.
    fn push_ordered(
        &mut self,
        sq_dist: f64,
        (ta, pa): (f64, Point3),
        (tb, pb): (f64, Point3),
        swapped: bool,
    ) {
        let (on_large, on_small) = (POnCurv::new(ta, pa), POnCurv::new(tb, pb));
        if swapped {
            self.result.push(sq_dist, on_small, on_large);
        } else {
            self.result.push(sq_dist, on_large, on_small);
        }
    }

    // The intersection points (see pkv/900/L4): one always, a second when the two are
    // more than Precision::Confusion() apart.
    fn intersection_points(
        &mut self,
        c1: &Circle3,
        c2: &Circle3,
        (dc1, dir12): (Direction3, Direction3),
        d12: f64,
        swapped: bool,
    ) -> Result<(), Failure> {
        let (r1, r2) = (c1.radius(), c2.radius());
        let alpha: f64 = 0.5 * (r1 * r1 - r2 * r2 + d12 * d12) / d12;
        let mut val: f64 = r1 * r1 - alpha * alpha;
        if val < 0. {
            // see pkv/900/L4 for details
            val = -val;
        }
        let beta: f64 = val.sqrt();
        let pt: Xyz = c1.location().xyz() + dir12.xyz() * alpha;
        let dlt: Direction3 = dc1.crossed(dir12)?;
        let pl1: Point3 = Point3::from(pt + dlt.xyz() * beta);
        let pl2: Point3 = Point3::from(pt - dlt.xyz() * beta);
        let tol_d: f64 = precision::confusion();
        let two: bool = pl1.square_distance(&pl2) > tol_d * tol_d;
        for point in if two { vec![pl1, pl2] } else { vec![pl1] } {
            let (ta, tb) = (elclib::parameter(c1, point)?, elclib::parameter(c2, point)?);
            self.push_ordered(0., (ta, point), (tb, point), swapped);
        }
        Ok(())
    }
}
