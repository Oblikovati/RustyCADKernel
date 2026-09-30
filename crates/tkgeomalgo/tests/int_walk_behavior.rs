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

//! Behavior of the IntWalk port beyond the numeric probe, which compares the marching
//! with native OCCT bit for bit.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::{IntPatchTheIWalking, IntPatchTheSurfFunction};
use rustycad_tkgeomalgo::int_surf::{IntSurfInteriorPoint, IntSurfQuadric};
use rustycad_tkgeomalgo::int_walk::*;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Sphere3, Vector2};
use rustycad_tkmath::math::{Matrix, Vector};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, PI};

fn adapted(
    surface: Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_bounds(&surface, bounds).unwrap()).cast()
}

fn frame(origin: (f64, f64, f64), normal: (f64, f64, f64)) -> Axis3 {
    let p = Point3::new(origin.0, origin.1, origin.2);
    Axis3::from_normal(p, Direction3::new(normal.0, normal.1, normal.2).unwrap()).unwrap()
}

fn plane(origin: (f64, f64, f64), normal: (f64, f64, f64)) -> Adaptor3dSurfaceHandle {
    adapted(
        Handle::new(GeomPlane::new(frame(origin, normal))).cast(),
        (-2.0, 2.0, -2.0, 2.0),
    )
}

fn sphere(center: (f64, f64, f64), radius: f64) -> Adaptor3dSurfaceHandle {
    let s = GeomSphericalSurface::new(frame(center, (0.0, 0.0, 1.0)), radius).unwrap();
    adapted(
        Handle::new(s).cast(),
        (0.0, 2.0 * PI, -FRAC_PI_2, FRAC_PI_2),
    )
}

fn start(values: [f64; 4]) -> Array1<'static, f64> {
    let mut a: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    for (i, v) in values.into_iter().enumerate() {
        a[i as i32 + 1] = v;
    }
    a
}

#[test]
fn pwalking_marches_a_closed_circle_on_both_surfaces() {
    // The plane y = 0.5 cuts a closed circle off the unit sphere inside both parameter spaces.
    let (s1, s2) = (
        plane((0.0, 0.5, 0.0), (0.0, 1.0, 0.0)),
        sphere((0.0, 0.0, 0.0), 1.0),
    );
    let mut walk = IntWalkPWalking::new(&s1, &s2, 1e-6, 1e-8, 0.01, 0.1).unwrap();
    walk.perform(&start([0.866, 0.0, FRAC_PI_2, FRAC_PI_3]))
        .unwrap();
    assert!(walk.is_done());
    assert!(walk.is_closed().unwrap());
    let n = walk.nb_points().unwrap();
    assert!(n > 10);
    assert_eq!(walk.value(1).unwrap(), walk.value(n).unwrap());
    for i in 1..=n {
        let p = walk.value(i).unwrap().value();
        assert!((p.y() - 0.5).abs() < 1e-6);
        assert!((p.x() * p.x() + p.y() * p.y() + p.z() * p.z() - 1.0).abs() < 1e-6);
    }
}

#[test]
fn pwalking_reports_its_errors() {
    let (s1, s2) = (
        plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0)),
        sphere((0.0, 0.0, 0.3), 1.0),
    );
    let mut walk = IntWalkPWalking::new(&s1, &s2, 1e-6, 1e-8, 0.01, 0.1).unwrap();
    // No line before a Perform: a null handle natively.
    assert_eq!(
        walk.nb_points().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        walk.max_step(4).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(walk.max_step(3).unwrap() > 0.0);
    walk.perform(&start([0.9539, 0.0, 0.0, -0.3047])).unwrap();
    let n = walk.nb_points().unwrap();
    assert_eq!(
        walk.value(n + 1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    walk.remove_a_point(n + 10).unwrap();
    assert_eq!(walk.nb_points().unwrap(), n - 1);
}

#[test]
fn pwalking_stops_on_tangent_surfaces() {
    // The plane z = 1.3 touches the sphere at its pole: the first point is tangent.
    let (s1, s2) = (
        sphere((0.0, 0.0, 0.3), 1.0),
        plane((0.0, 0.0, 1.3), (0.0, 0.0, 1.0)),
    );
    let mut walk = IntWalkPWalking::new(&s1, &s2, 1e-6, 1e-8, 0.01, 0.1).unwrap();
    walk.perform(&start([0.0, FRAC_PI_2, 0.0, 0.0])).unwrap();
    assert!(!walk.is_done());
    assert_eq!(
        walk.nb_points().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    let first = walk
        .perform_first_point(&start([0.0, 1.5, 0.0, 0.0]))
        .unwrap();
    assert!(first.is_some());
}

#[test]
fn pwalking_minimizations_skip_analytic_surfaces() {
    let (s1, s2) = (
        plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0)),
        sphere((0.0, 0.0, 0.3), 1.0),
    );
    let walk = IntWalkPWalking::new(&s1, &s2, 1e-6, 1e-8, 0.01, 0.1).unwrap();
    let mut init = [0.9, 0.1, 0.1, -0.3];
    assert!(
        walk.distance_minimize_by_gradient(&s1, &s2, &mut init, None)
            .unwrap()
    );
    assert_eq!(init, [0.9, 0.1, 0.1, -0.3]);
}

#[test]
fn iwalking_closes_a_circle_from_an_interior_point() {
    // The sphere of radius 0.8 centred at z = 0.3 cuts the plane z = 0 along a circle.
    let square = Handle::new(GeomPlane::new(frame((0.0, 0.0, 0.0), (0.0, 0.0, 1.0))));
    let patch = adapted(square.cast(), (-1.0, 1.0, -1.0, 1.0));
    let ball = Sphere3::new(frame((0.0, 0.0, 0.3), (0.0, 0.0, 1.0)), 0.8).unwrap();
    let quadric = IntSurfQuadric::from_sphere(&ball);
    let mut func = IntPatchTheSurfFunction::with_surfaces(&patch, &quadric);
    func.set_tolerance(1e-7);
    let r: f64 = (0.64f64 - 0.09).sqrt();
    // The interior point carries the tangents of the function there.
    let mut x = Vector::with_bounds(1, 2).unwrap();
    let (mut f, mut d) = (
        Vector::with_bounds(1, 1).unwrap(),
        Matrix::with_bounds(1, 1, 1, 2).unwrap(),
    );
    (x[1], x[2]) = (r, 0.0);
    func.values(&x, &mut f, &mut d).unwrap();
    let d2 = func.direction2d().unwrap();
    let start = IntSurfInteriorPoint::with_value(
        func.point(),
        r,
        0.0,
        func.direction3d().unwrap(),
        Vector2::new(d2.x(), d2.y()),
    );
    let mut walk = IntPatchTheIWalking::new(1e-6, 0.01, 0.1, false).unwrap();
    assert_eq!(
        walk.nb_lines().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    walk.perform(&[], &[start], &mut func, &patch, false)
        .unwrap();
    assert_eq!(walk.nb_lines().unwrap(), 1);
    assert_eq!(walk.nb_single_pnts().unwrap(), 0);
    let line = walk.value(1).unwrap();
    let n = line.nb_points();
    assert!(n > 10);
    for i in 1..=n {
        let p = line.value(i).unwrap().value();
        assert!((p.x().hypot(p.y()) - r).abs() < 1e-5);
    }
    let error = walk.value(2).err().map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("Standard_OutOfRange"));
}
