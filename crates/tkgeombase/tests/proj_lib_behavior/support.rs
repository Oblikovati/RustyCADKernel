// Copyright (c) 1993-1999 Matra Datavision
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

//! Geometry builders shared by the ProjLib behaviour tests.
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine,
    GeomParabola, GeomPlane, GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point3};

pub fn kind<T>(result: Result<T, Failure>) -> &'static str {
    match result {
        Ok(_) => "ok",
        Err(failure) => failure.exception_type(),
    }
}

pub fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

pub fn ax2(origin: Point3, normal: Direction3, x: Direction3) -> Axis2 {
    Axis2::with_x_direction(origin, normal, x).unwrap()
}

pub fn xoy() -> Axis3 {
    Axis3::new(
        Point3::new(0.0, 0.0, 0.0),
        dir(0.0, 0.0, 1.0),
        dir(1.0, 0.0, 0.0),
    )
    .unwrap()
}

pub fn surface(geom: impl GeomSurface + Transient + RuntimeTyped) -> Handle<dyn Adaptor3dSurface> {
    let handle: Handle<dyn GeomSurface> = Handle::new(geom).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&handle).unwrap()).cast()
}

pub fn curve(geom: impl GeomCurve + Transient + RuntimeTyped) -> Handle<dyn Adaptor3dCurve> {
    let handle: Handle<dyn GeomCurve> = Handle::new(geom).cast();
    Handle::new(GeomAdaptorCurve::with_curve(&handle).unwrap()).cast()
}

pub fn ranged(
    geom: impl GeomCurve + Transient + RuntimeTyped,
    first: f64,
    last: f64,
) -> Handle<dyn Adaptor3dCurve> {
    let handle: Handle<dyn GeomCurve> = Handle::new(geom).cast();
    Handle::new(GeomAdaptorCurve::with_range(&handle, first, last).unwrap()).cast()
}

pub fn plane() -> Handle<dyn Adaptor3dSurface> {
    surface(GeomPlane::new(xoy()))
}

pub fn cylinder(radius: f64) -> Handle<dyn Adaptor3dSurface> {
    surface(GeomCylindricalSurface::new(xoy(), radius).unwrap())
}

pub fn sphere(radius: f64) -> Handle<dyn Adaptor3dSurface> {
    surface(GeomSphericalSurface::new(xoy(), radius).unwrap())
}

/// The cone of XOY with the given semi-angle and the reference radius 1.
pub fn cone(semi_angle: f64) -> Handle<dyn Adaptor3dSurface> {
    surface(GeomConicalSurface::new(xoy(), semi_angle, 1.0).unwrap())
}

/// The torus of XOY with the major radius 5 and the minor radius 1.
pub fn torus() -> Handle<dyn Adaptor3dSurface> {
    surface(GeomToroidalSurface::new(xoy(), 5.0, 1.0).unwrap())
}

/// The extrusion along Z of the circle of radius 2.5 in XOY.
pub fn extrusion() -> Handle<dyn Adaptor3dSurface> {
    let circle: Handle<dyn GeomCurve> = Handle::new(horizontal_circle(0.0, 2.5)).cast();
    surface(GeomSurfaceOfLinearExtrusion::new(&circle, dir(0.0, 0.0, 1.0)).unwrap())
}

/// The flat Bezier patch `[0, 2] x [0, 2]` of XOY.
pub fn flat_patch() -> Handle<dyn Adaptor3dSurface> {
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            poles[(i, j)] = Point3::new(f64::from(i - 1), f64::from(j - 1), 0.0);
        }
    }
    surface(GeomBezierSurface::new(&poles).unwrap())
}

pub fn horizontal_circle(z: f64, radius: f64) -> GeomCircle {
    let frame = ax2(
        Point3::new(0.0, 0.0, z),
        dir(0.0, 0.0, 1.0),
        dir(1.0, 0.0, 0.0),
    );
    GeomCircle::new(frame, radius).unwrap()
}

pub fn line(origin: Point3, direction: Direction3) -> GeomLine {
    GeomLine::from_point(origin, direction)
}

/// A parabola in the XZ plane whose axis (X direction) is Z.
pub fn vertical_parabola() -> GeomParabola {
    let frame = ax2(
        Point3::new(0.0, 0.0, 0.0),
        dir(0.0, 1.0, 0.0),
        dir(0.0, 0.0, 1.0),
    );
    GeomParabola::new(frame, 0.5).unwrap()
}
