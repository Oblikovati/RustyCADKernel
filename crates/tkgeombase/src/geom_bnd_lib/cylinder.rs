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
// Rust translation of GeomBndLib_Cylinder and GeomBndLib_Cone at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Cylinder` and `GeomBndLib_Cone`: boxes of the V bands of a cylinder or a
//! cone from their boundary circles, opened along the axis for infinite V.
use super::circle::Circle;
use super::live;
use super::ranges::add_v_range;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomCylindricalSurface, GeomSurface};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Circle3, Cone3, Cylinder3, Vector3};

/// `GeomBndLib_Cylinder`: the box of a `Geom_CylindricalSurface` patch.
pub struct Cylinder {
    geom: Handle<GeomCylindricalSurface>,
}

// `computeCylinder`: the V iso circle at `v1` and its translation to `v2`, each as a
// GeomBndLib_Circle arc box of `[u_min, u_max]`.
fn cylinder_band(
    cylinder: &Cylinder3,
    (u_min, u_max): (f64, f64),
    bounds: &mut BoundingBox3,
    v1: f64,
    v2: f64,
) {
    let mut circle: Circle3 = elslib::cylinder_v_iso(cylinder.position(), cylinder.radius(), v1);
    bounds.add_box(&Circle::circ_arc_box(&circle, u_min, u_max, 0.0));
    circle.translate(Vector3::from(cylinder.axis().direction()) * (v2 - v1));
    bounds.add_box(&Circle::circ_arc_box(&circle, u_min, u_max, 0.0));
}

impl Cylinder {
    /// The evaluator of `cylinder`; `Cylinder::new(handle)`.
    pub fn new(cylinder: Handle<GeomCylindricalSurface>) -> Self {
        Self { geom: cylinder }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomCylindricalSurface> {
        &self.geom
    }
    /// `Box(Tol)`: the surface bounds (infinite in V); `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = live(&self.geom, "GeomBndLib_Cylinder")?.bounds();
        self.bounding_box_range(u1, u2, v1, v2, tolerance)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: the boundary circles of the V band; an
    /// infinite V end uses V = 0 and opens the axis on its side; a doubly infinite V
    /// range keeps the arc at V = 0 for the radial extent;
    /// `evaluator.bounding_box_range(0.0, 6.3, 0.0, 10.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let cylinder: Cylinder3 = live(&self.geom, "GeomBndLib_Cylinder")?.cylinder()?;
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        let mut band = |b: &mut BoundingBox3, v1: f64, v2: f64| {
            cylinder_band(&cylinder, (u_min, u_max), b, v1, v2);
            Ok(())
        };
        let mut whole = |b: &mut BoundingBox3| {
            let circle: Circle3 =
                elslib::cylinder_v_iso(cylinder.position(), cylinder.radius(), 0.0);
            b.add_box(&Circle::circ_arc_box(&circle, u_min, u_max, 0.0));
            Ok(())
        };
        let message = "GeomBndLib_Cylinder::Box - bad parameter";
        let axis = cylinder.axis().direction();
        add_v_range(
            &mut bounds,
            axis,
            (v_min, v_max),
            &mut band,
            &mut whole,
            message,
        )?;
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(0.0, 6.3, 0.0, 10.0, 0.0)?`.
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

/// `GeomBndLib_Cone`: the box of a `Geom_ConicalSurface` patch.
pub struct Cone {
    geom: Handle<GeomConicalSurface>,
}

// One V iso circle of the cone: its arc box, or its centre when the radius is below
// Precision::Confusion() (the apex).
fn add_cone_circle(cone: &Cone3, v: f64, (u_min, u_max): (f64, f64), bounds: &mut BoundingBox3) {
    let circle: Circle3 =
        elslib::cone_v_iso(cone.position(), cone.ref_radius(), cone.semi_angle(), v);
    if circle.radius() > precision::confusion() {
        bounds.add_box(&Circle::circ_arc_box(&circle, u_min, u_max, 0.0));
    } else {
        bounds.add_point(circle.location());
    }
}

impl Cone {
    /// The evaluator of `cone`; `Cone::new(handle)`.
    pub fn new(cone: Handle<GeomConicalSurface>) -> Self {
        Self { geom: cone }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomConicalSurface> {
        &self.geom
    }
    /// `Box(Tol)`: the surface bounds (infinite in V); `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = live(&self.geom, "GeomBndLib_Cone")?.bounds();
        self.bounding_box_range(u1, u2, v1, v2, tolerance)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: the V iso circles at both band ends (the apex
    /// as a point); infinite V as for the cylinder;
    /// `evaluator.bounding_box_range(0.0, 6.3, 0.0, 5.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let cone: Cone3 = live(&self.geom, "GeomBndLib_Cone")?.cone()?;
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        let mut band = |b: &mut BoundingBox3, v1: f64, v2: f64| {
            add_cone_circle(&cone, v1, (u_min, u_max), b);
            add_cone_circle(&cone, v2, (u_min, u_max), b);
            Ok(())
        };
        let mut whole = |b: &mut BoundingBox3| {
            add_cone_circle(&cone, 0.0, (u_min, u_max), b);
            Ok(())
        };
        let message = "GeomBndLib_Cone::Box - bad parameter";
        let axis = cone.axis().direction();
        add_v_range(
            &mut bounds,
            axis,
            (v_min, v_max),
            &mut band,
            &mut whole,
            message,
        )?;
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(0.0, 6.3, 0.0, 5.0, 0.0)?`.
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
