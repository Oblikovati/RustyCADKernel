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

//! Direct translation of IntPatch_Polyhedron_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::IntPatchPolyhedron;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

fn an_axis() -> Axis3 {
    Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

fn make_plane() -> Adaptor3dSurfaceHandle {
    let a_plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(an_axis())).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&a_plane, (-1.0, 1.0, -1.0, 1.0)).unwrap()).cast()
}

fn make_sphere() -> Adaptor3dSurfaceHandle {
    let a_sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(an_axis(), 1.0).unwrap()).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&a_sphere).unwrap()).cast()
}

// Basic construction with auto-computed subdivision.
#[test]
fn IntPatch_Polyhedron__DefaultConstructor_ProducesValidMesh() {
    let a_surf = make_sphere();
    let a_poly = IntPatchPolyhedron::new(&a_surf).unwrap();

    let (nb_u, nb_v) = a_poly.size();
    assert!(nb_u > 0);
    assert!(nb_v > 0);
    assert!(a_poly.nb_triangles() > 0);
    assert!(a_poly.nb_points() > 0);
}

// Regression test for bug #51: zero nbu/nbv must not cause division-by-zero.
// Constructor clamps to minimum 1.
#[test]
fn IntPatch_Polyhedron__ZeroSubdivision_ClampedToMinimum() {
    let a_surf = make_plane();

    let a_poly = IntPatchPolyhedron::with_subdivisions(&a_surf, 0, 0).unwrap();
    let (nb_u, nb_v) = a_poly.size();
    assert!(nb_u >= 1);
    assert!(nb_v >= 1);
    assert!(a_poly.nb_triangles() > 0);
}

// Small subdivision values should work normally.
#[test]
fn IntPatch_Polyhedron__SmallSubdivision_ProducesValidMesh() {
    let a_surf = make_plane();
    let a_poly = IntPatchPolyhedron::with_subdivisions(&a_surf, 2, 2).unwrap();

    let (nb_u, nb_v) = a_poly.size();
    assert_eq!(nb_u, 2);
    assert_eq!(nb_v, 2);
    assert_eq!(a_poly.nb_triangles(), 2 * 2 * 2);
}

// Regression test for bug #53: TriConnex must not crash when Pedge == 0.
#[test]
fn IntPatch_Polyhedron__TriConnex_PedgeZero_NoCrash() {
    let a_surf = make_sphere();
    let a_poly = IntPatchPolyhedron::with_subdivisions(&a_surf, 4, 4).unwrap();

    // Get a valid triangle and pivot point.
    let [a_p1, _a_p2, _a_p3] = a_poly.triangle(1);

    // Call TriConnex with Pedge = 0 (unknown edge mode).
    let (a_result, _a_tri_con, _an_other_p) = a_poly.tri_connex(1, a_p1, 0).unwrap();
    // Must not crash; result is a valid triangle index or 0.
    assert!(a_result >= 0);
}

// TriConnex with various Pedge values must not crash.
#[test]
fn IntPatch_Polyhedron__TriConnex_AllVertices_NoCrash() {
    let a_surf = make_sphere();
    let a_poly = IntPatchPolyhedron::with_subdivisions(&a_surf, 3, 3).unwrap();

    let [a_p1, a_p2, a_p3] = a_poly.triangle(1);

    // Call TriConnex with each vertex as Pedge and with Pedge=0.
    a_poly.tri_connex(1, a_p1, 0).unwrap();
    a_poly.tri_connex(1, a_p1, a_p2).unwrap();
    a_poly.tri_connex(1, a_p1, a_p3).unwrap();
    a_poly.tri_connex(1, a_p2, a_p3).unwrap();
    // All calls must complete without crash.
}
