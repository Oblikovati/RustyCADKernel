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
// Rust translation of GC_MakeTrimmedCone and GC_MakeTrimmedCylinder (.hxx/.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GC_MakeTrimmedCone and GC_MakeTrimmedCylinder: cones and cylinders trimmed to
//! a full turn in U and a height in V.
use super::{MakeConicalSurface, MakeCylindricalSurface};
use crate::gce::{ErrorType, Root};
use crate::line_extrema::line_projection;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkmath::gp::{Axis1, Circle3, Direction3, Line3, Point3};
use std::f64::consts::PI;

/// `Geom_RectangularTrimmedSurface(S, 0, 2 PI, 0, V2, true, true)`.
fn full_turn(
    surface: Handle<dyn GeomSurface>,
    v2: f64,
) -> Result<GeomRectangularTrimmedSurface, Failure> {
    GeomRectangularTrimmedSurface::new(&surface, (0.0, 2.0 * PI), (0.0, v2), (true, true))
}

/// GC_MakeTrimmedCone: a conical surface trimmed to `[0, 2 PI] x [0, D]`, where D
/// is the slant length between the two sections.
///
/// `let cone = MakeTrimmedCone::from_points_radii(p1, p2, 1.0, 2.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeTrimmedCone {
    the_result: Handle<GeomRectangularTrimmedSurface>,
    the_error: ErrorType,
}
gc_result!(
    MakeTrimmedCone,
    GeomRectangularTrimmedSurface,
    "GC_MakeTrimmedCone"
);

impl MakeTrimmedCone {
    /// GC_MakeTrimmedCone(P1, P2, P3, P4): the cone of GC_MakeConicalSurface(P1, P2,
    /// P3, P4) trimmed to the slant distance between the projections of P3 and P4
    /// on the axis P1P2 (`Extrema_ExtPElC` with tolerance 1e-7 on `[-2e100, 2e100]`).
    /// `MakeTrimmedCone::through_points(p1, p2, p3, p4)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3, p4: Point3) -> Result<Self, Failure> {
        let cone = MakeConicalSurface::through_points(p1, p2, p3, p4)?;
        if cone.status() != ErrorType::Done {
            return Ok(Self::failed(cone.status()));
        }
        let axis = Line3::new(p1, Direction3::from_xyz(p2.xyz() - p1.xyz())?);
        let p5: Point3 = line_projection(p3, axis, 1.0e-7, (-2.0e100, 2.0e100))?;
        let p6: Point3 = line_projection(p4, axis, 1.0e-7, (-2.0e100, 2.0e100))?;
        let surface = cone.value()?;
        let slant: f64 = p6.distance(&p5) / surface.semi_angle().cos();
        Ok(Self::built(full_turn(surface.cast(), slant)?))
    }

    /// GC_MakeTrimmedCone(P1, P2, R1, R2): the cone of GC_MakeConicalSurface(P1, P2,
    /// R1, R2) trimmed to the slant length `|P1 P2| / cos(SemiAngle)`.
    /// `MakeTrimmedCone::from_points_radii(p1, p2, 1.0, 2.0)?`.
    pub fn from_points_radii(p1: Point3, p2: Point3, r1: f64, r2: f64) -> Result<Self, Failure> {
        let cone = MakeConicalSurface::from_points_radii(p1, p2, r1, r2)?;
        if cone.status() != ErrorType::Done {
            return Ok(Self::failed(cone.status()));
        }
        let surface = cone.value()?;
        let slant: f64 = p2.distance(&p1) / surface.semi_angle().cos();
        Ok(Self::built(full_turn(surface.cast(), slant)?))
    }
}

/// GC_MakeTrimmedCylinder: a cylindrical surface trimmed to `[0, 2 PI] x [0, H]`.
///
/// `let cylinder = MakeTrimmedCylinder::from_axis(axis, 1.0, 2.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeTrimmedCylinder {
    the_result: Handle<GeomRectangularTrimmedSurface>,
    the_error: ErrorType,
}
gc_result!(
    MakeTrimmedCylinder,
    GeomRectangularTrimmedSurface,
    "GC_MakeTrimmedCylinder"
);

impl MakeTrimmedCylinder {
    /// The status of the cylindrical surface, and its trim to `height` when done.
    fn trimmed(cylinder: MakeCylindricalSurface, height: f64) -> Result<Self, Failure> {
        if cylinder.status() != ErrorType::Done {
            return Ok(Self::failed(cylinder.status()));
        }
        Ok(Self::built(full_turn(cylinder.value()?.cast(), height)?))
    }

    /// GC_MakeTrimmedCylinder(P1, P2, P3): the cylinder of axis P1P2 through P3,
    /// trimmed to the height `|P1 P2|`. `MakeTrimmedCylinder::through_points(a, b, c)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3) -> Result<Self, Failure> {
        Self::trimmed(
            MakeCylindricalSurface::through_points(p1, p2, p3)?,
            p2.distance(&p1),
        )
    }

    /// GC_MakeTrimmedCylinder(Circ, Height): the cylinder of `circle` trimmed to
    /// `height`. `MakeTrimmedCylinder::from_circ(circle, 2.0)?`.
    pub fn from_circ(circle: Circle3, height: f64) -> Result<Self, Failure> {
        Self::trimmed(MakeCylindricalSurface::from_circ(circle)?, height)
    }

    /// GC_MakeTrimmedCylinder(A1, Radius, Height): the cylinder of axis `axis`
    /// trimmed to `height`. `MakeTrimmedCylinder::from_axis(axis, 1.0, 2.0)?`.
    pub fn from_axis(axis: Axis1, radius: f64, height: f64) -> Result<Self, Failure> {
        Self::trimmed(MakeCylindricalSurface::from_axis(axis, radius)?, height)
    }
}
