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

//! Direct translation of IntPatch_PolyhedronBVH_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchBVHTraversal, IntPatchInterferencePolyhedron, IntPatchPolyhedron,
    IntPatchPolyhedronBVH, IntPatchPolyhedronTool,
};
use rustycad_tkmath::bvh::BvhSet;
use rustycad_tkmath::gp::{Axis3, Direction3, Plane3, Point3};
use std::f64::consts::PI;

/// The fixture `IntPatch_PolyhedronBVHTest`: a sphere and a cylinder.
struct IntPatchPolyhedronBVHTest {
    my_sphere_adaptor: Adaptor3dSurfaceHandle,
    my_cyl_adaptor: Adaptor3dSurfaceHandle,
}

fn z_axis(x: f64) -> Axis3 {
    Axis3::from_normal(
        Point3::new(x, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

impl IntPatchPolyhedronBVHTest {
    fn set_up() -> Self {
        // Create a sphere surface
        let a_sphere_surf: Handle<dyn GeomSurface> =
            Handle::new(GeomSphericalSurface::new(z_axis(0.0), 1.0).unwrap()).cast();
        let my_sphere_adaptor: Adaptor3dSurfaceHandle =
            Handle::new(GeomAdaptorSurface::with_surface(&a_sphere_surf).unwrap()).cast();
        // Create a cylinder surface
        let a_cyl_surf: Handle<dyn GeomSurface> =
            Handle::new(GeomCylindricalSurface::new(z_axis(0.5), 0.8).unwrap()).cast();
        let my_cyl_adaptor: Adaptor3dSurfaceHandle = Handle::new(
            GeomAdaptorSurface::with_bounds(&a_cyl_surf, (0.0, 2.0 * PI, -1.0, 1.0)).unwrap(),
        )
        .cast();
        Self {
            my_sphere_adaptor,
            my_cyl_adaptor,
        }
    }
}

// Test that PolyhedronBVH can be constructed and has correct size
#[test]
fn IntPatch_PolyhedronBVHTest__Construction() {
    let f = IntPatchPolyhedronBVHTest::set_up();
    let (a_nb_u, a_nb_v) = (10, 10);

    let a_poly =
        IntPatchPolyhedron::with_subdivisions(&f.my_sphere_adaptor, a_nb_u, a_nb_v).unwrap();
    let a_bvh = IntPatchPolyhedronBVH::with_polyhedron(&a_poly).unwrap();

    assert!(a_bvh.is_initialized());
    assert!(a_bvh.size() > 0);
    assert!(a_bvh.size() <= IntPatchPolyhedronTool::nb_triangles(&a_poly));
}

// Test that Box() returns valid bounding boxes
#[test]
fn IntPatch_PolyhedronBVHTest__Box() {
    let f = IntPatchPolyhedronBVHTest::set_up();
    let a_poly = IntPatchPolyhedron::with_subdivisions(&f.my_sphere_adaptor, 5, 5).unwrap();
    let a_bvh = IntPatchPolyhedronBVH::with_polyhedron(&a_poly).unwrap();

    // Check that all boxes are valid
    for i in 0..a_bvh.size() {
        let a_box = a_bvh.element_box(i).unwrap();
        assert!(a_box.is_valid(), "Box {i} is not valid");
    }
}

// Test that Center() returns valid centroid coordinates
#[test]
fn IntPatch_PolyhedronBVHTest__Center() {
    let f = IntPatchPolyhedronBVHTest::set_up();
    let a_poly = IntPatchPolyhedron::with_subdivisions(&f.my_sphere_adaptor, 5, 5).unwrap();
    let a_bvh = IntPatchPolyhedronBVH::with_polyhedron(&a_poly).unwrap();

    // Check that centroids are within the overall bounding box
    let a_bounding = IntPatchPolyhedronTool::bounding(&a_poly);
    let [a_xmin, a_ymin, a_zmin, a_xmax, a_ymax, a_zmax] = a_bounding.get().unwrap();

    for i in 0..a_bvh.size() {
        let a_cx = a_bvh.center(i, 0).unwrap();
        let a_cy = a_bvh.center(i, 1).unwrap();
        let a_cz = a_bvh.center(i, 2).unwrap();

        assert!(
            a_cx >= a_xmin - 1e-10,
            "Center X of triangle {i} is below min"
        );
        assert!(
            a_cx <= a_xmax + 1e-10,
            "Center X of triangle {i} is above max"
        );
        assert!(
            a_cy >= a_ymin - 1e-10,
            "Center Y of triangle {i} is below min"
        );
        assert!(
            a_cy <= a_ymax + 1e-10,
            "Center Y of triangle {i} is above max"
        );
        assert!(
            a_cz >= a_zmin - 1e-10,
            "Center Z of triangle {i} is below min"
        );
        assert!(
            a_cz <= a_zmax + 1e-10,
            "Center Z of triangle {i} is above max"
        );
    }
}

// Test that OriginalIndex() returns valid 1-based indices
#[test]
fn IntPatch_PolyhedronBVHTest__OriginalIndex() {
    let f = IntPatchPolyhedronBVHTest::set_up();
    let a_poly = IntPatchPolyhedron::with_subdivisions(&f.my_sphere_adaptor, 5, 5).unwrap();
    let mut a_bvh = IntPatchPolyhedronBVH::with_polyhedron(&a_poly).unwrap();

    let a_nb_tri = a_bvh.size();
    let a_nb_poly_tri = IntPatchPolyhedronTool::nb_triangles(&a_poly);

    // Before BVH build, indices should be sequential
    for i in 0..a_nb_tri {
        let an_orig_idx = a_bvh.original_index(i);
        assert!(an_orig_idx >= 1, "Original index should be >= 1");
        assert!(
            an_orig_idx <= a_nb_poly_tri,
            "Original index should be <= NbTriangles"
        );
    }

    // Force BVH build
    a_bvh.built_bvh().unwrap();

    // After BVH build, each original index should appear exactly once
    let mut a_used = vec![false; (a_nb_poly_tri + 1) as usize];
    for i in 0..a_nb_tri {
        let an_orig_idx = a_bvh.original_index(i) as usize;
        assert!(
            !a_used[an_orig_idx],
            "Original index {an_orig_idx} used more than once"
        );
        a_used[an_orig_idx] = true;
    }
}

// Test BVH traversal finds overlapping triangles
#[test]
fn IntPatch_PolyhedronBVHTest__Traversal() {
    let f = IntPatchPolyhedronBVHTest::set_up();
    let a_poly1 = IntPatchPolyhedron::with_subdivisions(&f.my_sphere_adaptor, 10, 10).unwrap();
    let a_poly2 = IntPatchPolyhedron::with_subdivisions(&f.my_cyl_adaptor, 10, 10).unwrap();

    let mut a_set1 = IntPatchPolyhedronBVH::with_polyhedron(&a_poly1).unwrap();
    let mut a_set2 = IntPatchPolyhedronBVH::with_polyhedron(&a_poly2).unwrap();

    let mut a_traversal = IntPatchBVHTraversal::new();
    let a_nb_pairs = a_traversal
        .perform(&mut a_set1, &mut a_set2, false)
        .unwrap();

    // The sphere and cylinder should have some overlapping triangles
    assert!(a_nb_pairs > 0, "Expected some overlapping triangle pairs");
    assert_eq!(a_nb_pairs, a_traversal.pairs().len() as i32);

    // Verify all pairs have valid indices
    for a_pair in a_traversal.pairs() {
        assert!(a_pair.first >= 1);
        assert!(a_pair.first <= IntPatchPolyhedronTool::nb_triangles(&a_poly1));
        assert!(a_pair.second >= 1);
        assert!(a_pair.second <= IntPatchPolyhedronTool::nb_triangles(&a_poly2));
    }
}

// Test self-interference mode
#[test]
fn IntPatch_PolyhedronBVHTest__SelfInterference() {
    let f = IntPatchPolyhedronBVHTest::set_up();
    let a_poly = IntPatchPolyhedron::with_subdivisions(&f.my_sphere_adaptor, 5, 5).unwrap();
    let mut a_set = IntPatchPolyhedronBVH::with_polyhedron(&a_poly).unwrap();

    let mut a_traversal = IntPatchBVHTraversal::new();
    a_traversal.perform_self(&mut a_set, true).unwrap(); // self-interference mode

    // In self-interference mode, First < Second for all pairs
    for a_pair in a_traversal.pairs() {
        assert!(
            a_pair.first < a_pair.second,
            "Self-interference should have First < Second"
        );
    }
}

// Test that IntPatch_InterferencePolyhedron works with BVH
#[test]
fn IntPatch_PolyhedronBVHTest__InterferencePolyhedron() {
    let f = IntPatchPolyhedronBVHTest::set_up();
    let a_poly1 = IntPatchPolyhedron::with_subdivisions(&f.my_sphere_adaptor, 10, 10).unwrap();
    let a_poly2 = IntPatchPolyhedron::with_subdivisions(&f.my_cyl_adaptor, 10, 10).unwrap();

    // Create interference and check it completes without error
    let an_interf = IntPatchInterferencePolyhedron::between(&a_poly1, &a_poly2).unwrap();

    // The result should have some section points or lines
    // (exact number depends on geometry, just verify it runs)
    let a_nb_points = an_interf.nb_section_points();
    let a_nb_lines = an_interf.nb_section_lines();
    let a_nb_zones = an_interf.nb_tangent_zones();

    // At least some intersection should be found
    assert!(
        a_nb_points > 0 || a_nb_lines > 0 || a_nb_zones > 0,
        "Expected some intersection results"
    );
}

// Test with non-overlapping surfaces
#[test]
fn IntPatch_PolyhedronBVHTest__NoOverlap() {
    let f = IntPatchPolyhedronBVHTest::set_up();
    // Create a plane far from the sphere
    let a_plane = Plane3::new(
        Point3::new(10.0, 10.0, 10.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let a_plane_surf: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::new(a_plane.position())).cast();
    let a_plane_adaptor: Adaptor3dSurfaceHandle = Handle::new(
        GeomAdaptorSurface::with_bounds(&a_plane_surf, (-1.0, 1.0, -1.0, 1.0)).unwrap(),
    )
    .cast();

    let a_poly1 = IntPatchPolyhedron::with_subdivisions(&f.my_sphere_adaptor, 5, 5).unwrap();
    let a_poly2 = IntPatchPolyhedron::with_subdivisions(&a_plane_adaptor, 5, 5).unwrap();

    let mut a_set1 = IntPatchPolyhedronBVH::with_polyhedron(&a_poly1).unwrap();
    let mut a_set2 = IntPatchPolyhedronBVH::with_polyhedron(&a_poly2).unwrap();

    let mut a_traversal = IntPatchBVHTraversal::new();
    let a_nb_pairs = a_traversal
        .perform(&mut a_set1, &mut a_set2, false)
        .unwrap();

    // Far-away surfaces should have no overlapping boxes
    assert_eq!(
        a_nb_pairs, 0,
        "Expected no overlapping triangle pairs for distant surfaces"
    );
}
