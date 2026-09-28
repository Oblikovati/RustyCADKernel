// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of the file-local AxeOperator class and RefineDir of
// IntAna_QuadQuadGeo.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! AxeOperator: the relative position of two axes (parallel, coplanar,
//! intersecting) and their distance. The native Coplanar() and Normal()
//! members are unused by IntAna_QuadQuadGeo and are not ported.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Point3, Vector3, Xyz};

/// The relative position of two axes, computed on construction.
pub(super) struct AxeOperator {
    ptintersect: Point3,
    axe1: Axis1,
    axe2: Axis1,
    thedistance: f64,
    theparallel: bool,
    thecoplanar: bool,
    eps_distance: f64,
}

/// Det33: the determinant of the 3x3 matrix given by rows.
fn det33(row1: [f64; 3], row2: [f64; 3], row3: [f64; 3]) -> f64 {
    let [a11, a12, a13] = row1;
    let [a21, a22, a23] = row2;
    let [a31, a32, a33] = row3;
    a11 * (a22 * a33 - a32 * a23) - a21 * (a12 * a33 - a32 * a13) + a31 * (a12 * a23 - a22 * a13)
}

impl AxeOperator {
    /// AxeOperator(A1, A2) with the native default tolerances (1e-14 for
    /// distances, the angular precision for parallelism).
    pub fn new(a1: Axis1, a2: Axis1) -> Result<Self, Failure> {
        Self::with_tolerances(a1, a2, 1.0e-14, precision::angular())
    }

    /// AxeOperator(A1, A2, theEpsDistance, theEpsAxesPara): parallel within
    /// `eps_axes_para` (on the refined directions), coplanar when the axes are
    /// closer than `eps_distance` and their mixed product vanishes, and the
    /// intersection point of
    /// coplanar, non-parallel axes (the origin otherwise).
    pub fn with_tolerances(
        a1: Axis1,
        a2: Axis1,
        eps_distance: f64,
        eps_axes_para: f64,
    ) -> Result<Self, Failure> {
        let (mut v1, mut v2) = (a1.direction(), a2.direction());
        let (p1, p2) = (a1.location(), a2.location());
        refine_dir(&mut v1)?;
        refine_dir(&mut v2)?;
        let parallel: bool = v1.is_parallel(v2, eps_axes_para);
        let perp: Xyz = a1.direction().xyz().crossed(a2.direction().xyz());
        let distance: f64 = if parallel {
            Line3::from_axis(a1).distance(a2.location())
        } else {
            Vector3::from(perp.normalized()?)
                .dot(Vector3::between(p1, p2))
                .abs()
        };
        let mut coplanar: bool = false;
        if distance < eps_distance {
            let offset = [p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z()];
            let d33: f64 = det33(*v1.coordinates(), *v2.coordinates(), offset);
            coplanar = d33.abs() <= eps_distance;
        }
        let ptintersect: Point3 = if coplanar && !parallel {
            intersection_point(v1, v2, p1, p2)
        } else {
            // To avoid FPE.
            Point3::new(0.0, 0.0, 0.0)
        };
        Ok(Self {
            ptintersect,
            axe1: a1,
            axe2: a2,
            thedistance: distance,
            theparallel: parallel,
            thecoplanar: coplanar,
            eps_distance,
        })
    }

    /// Distance(dist, Param1, Param2): the signed distance of the axes along
    /// `N = D1 ^ D2` and the parameters of the common perpendicular, solving
    /// `P1 D1 + P2 D2 + d N = O1O2`; zeros when the determinant vanishes (the
    /// native outputs stay unassigned there).
    pub fn distance_parameters(&self) -> Result<(f64, f64, f64), Failure> {
        let o1o2 = Vector3::between(self.axe1.location(), self.axe2.location());
        let (u1, u2) = (self.axe1.direction(), self.axe2.direction());
        let n: Direction3 = u1.crossed(u2)?;
        let (o, uu, vv, nn) = (
            [o1o2.x(), o1o2.y(), o1o2.z()],
            *u1.coordinates(),
            *u2.coordinates(),
            *n.coordinates(),
        );
        let column = |a: [f64; 3], b: [f64; 3], c: [f64; 3]| {
            det33([a[0], b[0], c[0]], [a[1], b[1], c[1]], [a[2], b[2], c[2]])
        };
        let d: f64 = column(uu, vv, nn);
        if d == 0.0 {
            return Ok((0.0, 0.0, 0.0));
        }
        let dist: f64 = column(uu, vv, o) / d;
        let param1: f64 = column(o, vv, nn) / (-d);
        let param2: f64 = column(uu, o, nn) / d;
        Ok((dist, param1, param2))
    }

    pub fn pt_intersect(&self) -> Point3 {
        self.ptintersect
    }

    pub fn same(&self) -> bool {
        self.theparallel && (self.thedistance < self.eps_distance)
    }

    pub fn distance(&self) -> f64 {
        self.thedistance
    }

    pub fn intersect(&self) -> bool {
        self.thecoplanar && !self.theparallel
    }

    pub fn parallel(&self) -> bool {
        self.theparallel
    }
}

/// The intersection of coplanar axes, solved on the largest 2x2 minor.
fn intersection_point(v1: Direction3, v2: Direction3, p1: Point3, p2: Point3) -> Point3 {
    let (smx, smy, smz) = (p2.x() - p1.x(), p2.y() - p1.y(), p2.z() - p1.z());
    let det1: f64 = v1.y() * v2.x() - v1.x() * v2.y();
    let det2: f64 = v1.z() * v2.y() - v1.y() * v2.z();
    let det3: f64 = v1.z() * v2.x() - v1.x() * v2.z();
    let a: f64 = if (det1 != 0.0) && ((det1.abs() >= det2.abs()) && (det1.abs() >= det3.abs())) {
        (smy * v2.x() - smx * v2.y()) / det1
    } else if (det2 != 0.0) && ((det2.abs() >= det1.abs()) && (det2.abs() >= det3.abs())) {
        (smz * v2.y() - smy * v2.z()) / det2
    } else {
        (smz * v2.x() - smx * v2.z()) / det3
    };
    Point3::new(
        p1.x() + a * v1.x(),
        p1.y() + a * v1.y(),
        p1.z() + a * v1.z(),
    )
}

/// RefineDir: a direction with one coordinate exactly ±1 and another non-null
/// has its coordinate within RealEpsilon() of ±1 snapped to the axis.
fn refine_dir(direction: &mut Direction3) -> Result<(), Failure> {
    let mut c: [f64; 3] = *direction.coordinates();
    let units: usize = c.iter().filter(|&&v| v == 1. || v == -1.).count();
    let others: usize = c
        .iter()
        .filter(|&&v| !(v == 1. || v == -1.) && v != 0.)
        .count();
    if units == 0 || others == 0 {
        return Ok(());
    }
    let (r1, r2) = (1. - f64::EPSILON, 1. + f64::EPSILON);
    for k in 0..3 {
        let positive: bool = c[k] > 0.;
        let magnitude: f64 = if positive { c[k] } else { -c[k] };
        if magnitude > r1 && magnitude < r2 {
            c[k] = if positive { 1. } else { -1. };
            c[(k + 1) % 3] = 0.;
            c[(k + 2) % 3] = 0.;
            break;
        }
    }
    direction.set_coordinates(c[0], c[1], c[2])
}
