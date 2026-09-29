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

//! GeomProjLib behaviour: the contracts of every overload and the preserved native
//! quirks (docs/tkgeombase-progress.md, Preserved native behaviour). Bit-exact parity
//! with the native kernel is the numeric oracle's job (`gpl_*` requests of
//! tools/verify_tkgeombase_numeric.py).
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkgeombase::geom_proj_lib as gpl;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point3};

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

fn xoy() -> Axis3 {
    Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap()
}

fn plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(xoy())).cast()
}

fn cylinder() -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(xoy(), 2.0).unwrap()).cast()
}

fn circle(z: f64, radius: f64) -> Handle<dyn GeomCurve> {
    let frame = Axis2::new(Point3::new(0.0, 0.0, z), dir(0.0, 0.0, 1.0)).unwrap();
    Handle::new(GeomCircle::new(frame, radius).unwrap()).cast()
}

fn segment(origin: Point3, direction: Direction3, u1: f64, u2: f64) -> Handle<dyn GeomCurve> {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(origin, direction)).cast();
    Handle::new(GeomTrimmedCurve::new(&line, u1, u2, true, true).unwrap()).cast()
}

fn name<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> &'static str {
    handle.dynamic_type().map_or("null", |kind| kind.name())
}

#[test]
fn curve2d_of_a_circle_on_a_plane_is_a_circle_with_the_tolerance_raised() {
    let (c2d, reached) =
        gpl::curve2d_range_tolerance(&circle(0.0, 1.5), (0.0, 1.0), &plane(), 0.0).unwrap();
    assert_eq!(name(&c2d), "Geom2d_Circle");
    let circle2d: Handle<Geom2dCircle> = c2d.cast();
    assert_eq!(circle2d.get().unwrap().radius(), 1.5);
    // The tolerance is raised to PConfusion, then ProjLib_ProjectedCurve raises it to
    // Confusion, which the exact projection returns.
    assert_eq!(reached, precision::confusion());
    let whole: Handle<dyn Geom2dCurve> = gpl::curve2d(&circle(0.0, 1.5), &plane()).unwrap();
    assert_eq!(name(&whole), "Geom2d_Circle");
}

#[test]
fn curve2d_trims_by_the_3d_trim_and_clamps_a_bounded_result() {
    // The trimmed segment gives a trimmed line over its own trim parameters, whatever
    // the given range; a line result is not periodic but unbounded, so nothing clamps.
    let c = segment(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0), -1.0, 2.0);
    let c2d = gpl::curve2d_range(&c, (0.0, 0.5), &plane()).unwrap();
    assert_eq!(name(&c2d), "Geom2d_TrimmedCurve");
    let trimmed: Handle<Geom2dTrimmedCurve> = c2d.cast();
    let t = trimmed.get().unwrap();
    assert_eq!((t.first_parameter(), t.last_parameter()), (-1.0, 2.0));
    assert_eq!(name(&t.basis_curve()), "Geom2d_Line");
    let _: Handle<Geom2dLine> = t.basis_curve().cast();
}

#[test]
fn curve2d_bounded_overloads_restrict_the_surface() {
    let c = segment(Point3::new(2.0, 0.0, 0.0), dir(0.0, 0.0, 1.0), 0.0, 1.0);
    let bounds = (0.0, 2.0 * std::f64::consts::PI, -1.0, 2.0);
    let c2d = gpl::curve2d_bounds(&c, &cylinder(), bounds).unwrap();
    assert_eq!(name(&c2d), "Geom2d_TrimmedCurve");
    let (with_tol, reached) = gpl::curve2d_bounds_tolerance(&c, &cylinder(), bounds, 1e-3).unwrap();
    assert_eq!(name(&with_tol), "Geom2d_TrimmedCurve");
    assert_eq!(reached, 1e-3);
}

#[test]
fn curve2d_of_null_inputs_is_a_null_object() {
    let null_curve: Handle<dyn GeomCurve> = Handle::default();
    let null_surface: Handle<dyn GeomSurface> = Handle::default();
    let error = gpl::curve2d(&null_curve, &plane()).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    let error = gpl::curve2d_range_tolerance(&circle(0.0, 1.0), (0.0, 1.0), &null_surface, 0.0)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_NullObject");
}

#[test]
fn project_on_a_plane_keeps_the_3d_trim_parameters() {
    let c = segment(Point3::new(0.0, 0.0, 3.0), dir(1.0, 0.0, 0.0), -1.0, 2.0);
    let projected = gpl::project(&c, &plane()).unwrap();
    assert_eq!(name(&projected), "Geom_TrimmedCurve");
    let t: Handle<GeomTrimmedCurve> = projected.cast();
    let t = t.get().unwrap();
    assert_eq!((t.first_parameter(), t.last_parameter()), (-1.0, 2.0));
    assert_eq!(t.value(0.0).unwrap(), Point3::new(0.0, 0.0, 0.0));
}

#[test]
fn project_on_a_cylinder_is_an_approximated_bspline() {
    let projected = gpl::project(&circle(1.0, 2.5), &cylinder()).unwrap();
    assert_eq!(name(&projected), "Geom_BSplineCurve");
    let spline: Handle<GeomBSplineCurve> = projected.cast();
    let point = spline.get().unwrap().value(0.0).unwrap();
    assert!((point.distance(&Point3::new(2.0, 0.0, 1.0))) < 1e-4);
}

#[test]
fn project_on_plane_keeps_the_parametrization_when_asked() {
    let geom_plane: Handle<GeomPlane> = Handle::new(GeomPlane::new(xoy()));
    let c = segment(Point3::new(0.0, 0.0, 1.0), dir(1.0, 0.0, 1.0), 0.0, 2.0);
    let kept = gpl::project_on_plane(&c, &geom_plane, dir(0.0, 0.0, 1.0), true).unwrap();
    let free = gpl::project_on_plane(&c, &geom_plane, dir(0.0, 0.0, 1.0), false).unwrap();
    assert_eq!(name(&kept), "Geom_TrimmedCurve");
    assert_eq!(name(&free), "Geom_TrimmedCurve");
    let kept_curve: &dyn GeomCurve = kept.get().unwrap();
    let free_curve: &dyn GeomCurve = free.get().unwrap();
    assert_eq!(
        (kept_curve.first_parameter(), kept_curve.last_parameter()),
        (0.0, 2.0)
    );
    // Without it the projected line is parametrized by its arc length.
    assert!((free_curve.last_parameter() - 2.0 * 0.5_f64.sqrt()).abs() < 1e-12);
}

#[test]
fn project_on_plane_refuses_a_trimmed_line_along_a_direction_in_the_plane() {
    // Rust deviation: the NaN segment of the line never returns natively (the trim).
    let geom_plane: Handle<GeomPlane> = Handle::new(GeomPlane::new(xoy()));
    let c = segment(Point3::new(0.0, 0.0, 1.0), dir(0.0, 0.0, 1.0), 0.0, 2.0);
    let error = gpl::project_on_plane(&c, &geom_plane, dir(1.0, 0.0, 0.0), false)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let null_plane: Handle<GeomPlane> = Handle::default();
    let error = gpl::project_on_plane(&c, &null_plane, dir(0.0, 0.0, 1.0), false)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_NullObject");
}

#[test]
fn curve2d_refuses_the_zero_length_segment_of_a_line_on_a_sphere_or_a_cylinder() {
    // Rust deviation (regression): the window scan of ProjLib_ComputeApprox has a zero
    // step on `[1, 1]`, so the native call never returns.
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(xoy(), 2.0).unwrap()).cast();
    let origin: Point3 = Point3::new(3.0, 0.0, 0.0);
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(origin, dir(0.6, 0.8, 0.0))).cast();
    for surface in [sphere, cylinder()] {
        let error = gpl::curve2d_range(&line, (1.0, 1.0), &surface)
            .err()
            .unwrap();
        assert_eq!(error.exception_type(), "Standard_DomainError");
        assert!(error.message().contains("the curve range [1, 1]"));
    }
}
