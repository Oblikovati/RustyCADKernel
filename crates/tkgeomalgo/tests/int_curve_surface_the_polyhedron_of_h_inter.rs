// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Direct translation of IntCurveSurface_ThePolyhedronOfHInter_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_curve_surface::ThePolyhedronOfHInter;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

fn make_test_plane() -> Adaptor3dSurfaceHandle {
    let an_axis = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let a_plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(an_axis)).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&a_plane, (-1.0, 1.0, -1.0, 1.0)).unwrap()).cast()
}

fn params(values: &[f64]) -> Array1<'static, f64> {
    let mut array: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, &value) in (1..).zip(values) {
        array[i] = value;
    }
    array
}

// Regression test for bug #55: singularity flags must be initialized to false.
#[test]
fn IntCurveSurface_ThePolyhedronOfHInter__SingularityFlags_InitializedFalse() {
    let a_surf = make_test_plane();
    let a_poly = ThePolyhedronOfHInter::new(&a_surf, 3, 3, -1.0, -1.0, 1.0, 1.0).unwrap();

    assert!(!a_poly.has_u_min_singularity());
    assert!(!a_poly.has_u_max_singularity());
    assert!(!a_poly.has_v_min_singularity());
    assert!(!a_poly.has_v_max_singularity());
}

// Singularity setters should update the flags.
#[test]
fn IntCurveSurface_ThePolyhedronOfHInter__SingularitySetters_UpdateFlags() {
    let a_surf = make_test_plane();
    let mut a_poly = ThePolyhedronOfHInter::new(&a_surf, 3, 3, -1.0, -1.0, 1.0, 1.0).unwrap();

    a_poly.u_min_singularity(true);
    assert!(a_poly.has_u_min_singularity());
    assert!(!a_poly.has_u_max_singularity());

    a_poly.v_max_singularity(true);
    assert!(a_poly.has_v_max_singularity());
    assert!(!a_poly.has_v_min_singularity());
}

// Basic construction produces valid polyhedron.
#[test]
fn IntCurveSurface_ThePolyhedronOfHInter__BasicConstruction_ValidMesh() {
    let a_surf = make_test_plane();
    let a_poly = ThePolyhedronOfHInter::new(&a_surf, 4, 4, -1.0, -1.0, 1.0, 1.0).unwrap();

    let (nb_u, nb_v) = a_poly.size();
    assert_eq!(nb_u, 4);
    assert_eq!(nb_v, 4);
    assert_eq!(a_poly.nb_triangles(), 4 * 4 * 2);
    assert_eq!(a_poly.nb_points(), (4 + 1) * (4 + 1));
}

// Regression test for bug #58: parameter-array constructor with minimum-size arrays
// must not cause division by zero (arrays of length 2 give nbdelta = 1).
#[test]
fn IntCurveSurface_ThePolyhedronOfHInter__ParamArrayConstructor_MinimumSize() {
    let a_surf = make_test_plane();

    let a_upars = params(&[-1.0, 1.0]);
    let a_vpars = params(&[-1.0, 1.0]);

    let a_poly = ThePolyhedronOfHInter::with_parameters(&a_surf, &a_upars, &a_vpars).unwrap();

    let (nb_u, nb_v) = a_poly.size();
    assert!(nb_u >= 1);
    assert!(nb_v >= 1);
    assert!(a_poly.nb_triangles() > 0);
}

// Regression test for constructor validation added for invalid explicit parameter arrays.
#[test]
fn IntCurveSurface_ThePolyhedronOfHInter__ParamArrayConstructor_RejectsSingleValueArrays() {
    let a_surf = make_test_plane();

    let a_upars = params(&[0.0]);
    let a_vpars = params(&[-1.0, 1.0]);

    let error = ThePolyhedronOfHInter::with_parameters(&a_surf, &a_upars, &a_vpars)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");

    let a_valid_upars = params(&[-1.0, 1.0]);
    let a_single_vpars = params(&[0.0]);

    let error = ThePolyhedronOfHInter::with_parameters(&a_surf, &a_valid_upars, &a_single_vpars)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

// PlaneEquation for a valid triangle must produce finite results.
#[test]
fn IntCurveSurface_ThePolyhedronOfHInter__PlaneEquation_FiniteResults() {
    let a_surf = make_test_plane();
    let a_poly = ThePolyhedronOfHInter::new(&a_surf, 3, 3, -1.0, -1.0, 1.0, 1.0).unwrap();

    let (a_normal, a_polar_dist) = a_poly.plane_equation(1).unwrap();

    assert!(a_normal.x().is_finite());
    assert!(a_normal.y().is_finite());
    assert!(a_normal.z().is_finite());
    assert!(a_polar_dist.is_finite());
}
