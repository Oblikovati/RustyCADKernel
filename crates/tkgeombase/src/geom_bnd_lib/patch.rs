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
// Rust translation of the shared patch bodies of GeomBndLib_Sphere.cxx (computeSphere)
// and BndLib.cxx (ComputeSphere, computeDegeneratedTorus) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sphere and degenerate-torus patch analysis: the six axis-aligned extreme points
//! that lie in the patch, then the four boundary iso circles.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Point3};
use std::f64::consts::PI;

/// A `(u, v)` patch `[u_min, u_max] x [v_min, v_max]`.
#[derive(Clone, Copy, Debug)]
pub struct Patch {
    /// Lower U bound.
    pub u_min: f64,
    /// Upper U bound.
    pub u_max: f64,
    /// Lower V bound.
    pub v_min: f64,
    /// Upper V bound.
    pub v_max: f64,
}

impl Patch {
    /// Whether the point parameters, with `u` brought into `[u_min, u_min + 2Pi)`, lie in
    /// the patch; `patch.contains((0.5, 0.1))`.
    pub fn contains(&self, (u, v): (f64, f64)) -> bool {
        let u: f64 = elclib::in_period(u, self.u_min, self.u_min + 2.0 * PI);
        u >= self.u_min && u <= self.u_max && v >= self.v_min && v <= self.v_max
    }
}

/// Add the axis extremes `centre` moved to `extents = [xmin, xmax, ymin, ymax, zmin,
/// zmax]` one coordinate at a time, in the native order, when `parameters` of the point
/// lie in `patch`; `add_axis_extrema(&mut bounds, centre, extents, &params, patch)`.
pub fn add_axis_extrema(
    bounds: &mut BoundingBox3,
    centre: Point3,
    extents: [f64; 6],
    parameters: &dyn Fn(Point3) -> (f64, f64),
    patch: Patch,
) {
    let mut extreme: Point3 = centre;
    for axis in 0..3 {
        for value in [extents[2 * axis], extents[2 * axis + 1]] {
            set_axis(&mut extreme, axis, value);
            if patch.contains(parameters(extreme)) {
                bounds.add_point(extreme);
            }
        }
        set_axis(&mut extreme, axis, centre.coordinates()[axis]);
    }
}

// `SetX`/`SetY`/`SetZ` by axis index.
fn set_axis(point: &mut Point3, axis: usize, value: f64) {
    match axis {
        0 => point.set_x(value),
        1 => point.set_y(value),
        _ => point.set_z(value),
    }
}

/// The circle arc builder of the patch boundaries (GeomBndLib_Circle::Box added as a
/// box, or BndLib::Add on the accumulated box).
pub type ArcAdder<'f> = dyn Fn(&mut BoundingBox3, &Circle3, f64, f64) -> Result<(), Failure> + 'f;

/// Add the U iso circles at `u_min`, `u_max` over `[v_min, v_max]`, then the V iso
/// circles at `v_min`, `v_max` over `[u_min, u_max]`;
/// `add_iso_circles(&mut bounds, &u_iso, &v_iso, patch, &add_arc)`.
pub fn add_iso_circles(
    bounds: &mut BoundingBox3,
    u_iso: &dyn Fn(f64) -> Circle3,
    v_iso: &dyn Fn(f64) -> Circle3,
    patch: Patch,
    add_arc: &ArcAdder<'_>,
) -> Result<(), Failure> {
    add_arc(bounds, &u_iso(patch.u_min), patch.v_min, patch.v_max)?;
    add_arc(bounds, &u_iso(patch.u_max), patch.v_min, patch.v_max)?;
    add_arc(bounds, &v_iso(patch.v_min), patch.u_min, patch.u_max)?;
    add_arc(bounds, &v_iso(patch.v_max), patch.u_min, patch.u_max)
}
