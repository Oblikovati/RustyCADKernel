// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomBndLib_Plane.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Plane`: the box of a planar patch, opened for infinite parameters.
use super::live;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{self, Plane3, Point3};

/// `GeomBndLib_Plane`: the box of a `Geom_Plane` patch.
pub struct Plane {
    geom: Handle<GeomPlane>,
}

// `BaryCenter`: the patch centre, 10 inside the finite end of a half-infinite range and
// 0 for a doubly infinite one.
fn bary_centre(plane: &Plane3, [u_min, u_max, v_min, v_max]: [f64; 4]) -> Point3 {
    let centre =
        |min: f64, max: f64| match (precision::is_infinite(min), precision::is_infinite(max)) {
            (true, true) => 0.0,
            (true, false) => max - 10.0,
            (false, true) => min + 10.0,
            (false, false) => (min + max) / 2.0,
        };
    elslib::value(centre(u_min, u_max), centre(v_min, v_max), plane)
}

// `TreatInfinitePlane`: a plane normal to a global axis (within RealEpsilon) is its
// centre with the two in-plane axes opened; any other plane is the whole space (and,
// as natively, skips the final Enlarge).
fn treat_infinite_plane(
    plane: &Plane3,
    patch: [f64; 4],
    tolerance: f64,
    bounds: &mut BoundingBox3,
) {
    let normal = plane.axis().direction();
    let angular: f64 = real_epsilon();
    let location: Point3 = bary_centre(plane, patch);
    let normal_axis: Option<usize> = [gp::dx(), gp::dy(), gp::dz()]
        .iter()
        .position(|&axis| normal.is_parallel(axis, angular));
    let Some(normal_axis) = normal_axis else {
        bounds.set_whole();
        return;
    };
    bounds.add_point(location);
    let openers: [[fn(&mut BoundingBox3); 2]; 3] = [
        [BoundingBox3::open_xmin, BoundingBox3::open_xmax],
        [BoundingBox3::open_ymin, BoundingBox3::open_ymax],
        [BoundingBox3::open_zmin, BoundingBox3::open_zmax],
    ];
    for (axis, [open_min, open_max]) in openers.into_iter().enumerate() {
        if axis != normal_axis {
            open_min(bounds);
            open_max(bounds);
        }
    }
    bounds.enlarge(tolerance);
}

impl Plane {
    /// The evaluator of `plane`; `Plane::new(handle)`.
    pub fn new(plane: Handle<GeomPlane>) -> Self {
        Self { geom: plane }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomPlane> {
        &self.geom
    }
    /// `Box(Tol)`: the (infinite) bounds of the plane; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = live(&self.geom, "GeomBndLib_Plane")?.bounds();
        self.bounding_box_range(u1, u2, v1, v2, tolerance)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: the four corners, or the infinite treatment
    /// when any parameter is infinite; `evaluator.bounding_box_range(0.0, 1.0, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let plane: Plane3 = live(&self.geom, "GeomBndLib_Plane")?.pln();
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        let patch: [f64; 4] = [u_min, u_max, v_min, v_max];
        if patch.iter().any(|&p| precision::is_infinite(p)) {
            treat_infinite_plane(&plane, patch, tolerance, &mut bounds);
            return Ok(bounds);
        }
        for (u, v) in [
            (u_min, v_min),
            (u_min, v_max),
            (u_max, v_min),
            (u_max, v_max),
        ] {
            bounds.add_point(elslib::value(u, v, &plane));
        }
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: the exact corner box;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.bounding_box_range(u_min, u_max, v_min, v_max, tolerance)
    }
    /// `BoxOptimal(Tol)`: `Box(Tol)`; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        self.bounding_box(tolerance)
    }
}
