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

//! Behavior of IntPatch_WLineTool, IntPatch_LineConstructor and the grid members of
//! IntPatch_PrmPrmIntersection beyond the numeric probe, which compares them with native
//! OCCT bit for bit.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::*;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

fn axis(z: Direction3) -> Axis3 {
    Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), z).unwrap()
}

fn bounded_plane(normal: Direction3, bounds: (f64, f64, f64, f64)) -> Adaptor3dSurfaceHandle {
    let s: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(axis(normal))).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&s, bounds).unwrap()).cast()
}

fn cylinder(z: Direction3) -> Adaptor3dSurfaceHandle {
    let s: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(axis(z), 1.0).unwrap()).cast();
    Handle::new(
        GeomAdaptorSurface::with_bounds(&s, (0.0, 2.0 * std::f64::consts::PI, -5.0, 5.0)).unwrap(),
    )
    .cast()
}

fn z() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}

fn y() -> Direction3 {
    Direction3::new(0.0, 1.0, 0.0).unwrap()
}

/// A walking line of `n` points on the x axis at `x0 + i step`, with its end vertices.
fn straight(n: i32, x0: f64, step: f64) -> Handle<IntPatchWLine> {
    let v1: f64 = 5.0;
    let points = Handle::new(IntSurfLineOn2S::new(None));
    for i in 0..n {
        let x = x0 + f64::from(i) * step;
        let mut p = IntSurfPntOn2S::new();
        p.set_value_all(Point3::new(x, 0.0, 0.0), x, v1, x, 0.0);
        points.add(&p);
    }
    let line = Handle::new(IntPatchWLine::new_undecided(&points, false));
    for (i, x) in [(1, x0), (n, x0 + f64::from(n - 1) * step)] {
        let mut v = IntPatchPoint::new();
        v.set_value_tangent(Point3::new(x, 0.0, 0.0), 1.0e-7, false);
        v.set_parameters(x, v1, x, 0.0);
        v.set_parameter(f64::from(i));
        line.add_vertex(&v, false);
    }
    line
}

/// Six points of the unit cylinder of axis z at the angles `a0..a1`, height 0.5.
fn arc(a0: f64, a1: f64) -> Handle<IntPatchWLine> {
    let points = Handle::new(IntSurfLineOn2S::new(None));
    for i in 0..6 {
        let a = a0 + (a1 - a0) * f64::from(i) / 5.0;
        let mut p = IntSurfPntOn2S::new();
        p.set_value_all(Point3::new(a.cos(), a.sin(), 0.5), a, 0.5, 0.5 * a, 0.25);
        points.add(&p);
    }
    Handle::new(IntPatchWLine::new_undecided(&points, false))
}

#[test]
fn purge_of_two_equal_points_is_null() {
    let (s1, s2) = (
        bounded_plane(z(), (0.0, 4.0, 0.0, 10.0)),
        bounded_plane(y(), (-5.0, 5.0, -5.0, 5.0)),
    );
    let (d1, d2) = (
        Adaptor3dTopolTool::with_surface(&s1).unwrap(),
        Adaptor3dTopolTool::with_surface(&s2).unwrap(),
    );
    let line = straight(2, 0.5, 0.0);
    let purged = IntPatchWLineTool::compute_purged_wline(&line, &s1, &s2, &d1, &d2).unwrap();
    assert!(purged.is_null());
}

#[test]
fn purge_of_a_long_straight_line_keeps_its_ends() {
    let (s1, s2) = (
        bounded_plane(z(), (0.0, 4.0, 0.0, 10.0)),
        bounded_plane(y(), (-5.0, 5.0, -5.0, 5.0)),
    );
    let (d1, d2) = (
        Adaptor3dTopolTool::with_surface(&s1).unwrap(),
        Adaptor3dTopolTool::with_surface(&s2).unwrap(),
    );
    let line = straight(40, 0.5, 0.05);
    let purged = IntPatchWLineTool::compute_purged_wline(&line, &s1, &s2, &d1, &d2).unwrap();
    assert!(purged.is_some());
    // The collinear points are in the tubes of their neighbours: few points are left, and
    // the vertices stay on the ends.
    let nb = purged.nb_pnts().unwrap();
    assert!((2..40).contains(&nb));
    assert_eq!(purged.vertex(1).unwrap().parameter_on_line(), 1.0);
    assert_eq!(purged.vertex(2).unwrap().parameter_on_line(), f64::from(nb));
}

#[test]
fn join_needs_two_cylinders() {
    let plane = bounded_plane(z(), (0.0, 4.0, 0.0, 10.0));
    let mut lines: Vec<Handle<dyn IntPatchLine>> =
        vec![straight(3, 0.0, 0.1).cast(), straight(3, 0.2, 0.1).cast()];
    let mut points: Vec<IntPatchPoint> = Vec::new();
    let error =
        IntPatchWLineTool::join_wlines(&mut lines, &mut points, &plane, &plane, 1e-7).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
    // An empty sequence returns before reading the surfaces.
    IntPatchWLineTool::join_wlines(&mut Vec::new(), &mut points, &plane, &plane, 1e-7).unwrap();
}

#[test]
fn join_of_two_lines_sharing_an_end() {
    let (s1, s2) = (
        cylinder(z()),
        cylinder(Direction3::new(1.0, 0.0, 0.0).unwrap()),
    );
    // Two arcs of the first cylinder meeting at the angle 1 (the probe request of the same
    // lines is joined by the native kernel too).
    let mut lines: Vec<Handle<dyn IntPatchLine>> = vec![arc(0.3, 1.0).cast(), arc(1.0, 1.8).cast()];
    let mut points: Vec<IntPatchPoint> = vec![IntPatchPoint::new()];
    IntPatchWLineTool::join_wlines(&mut lines, &mut points, &s1, &s2, 1e-7).unwrap();
    assert_eq!(lines.len(), 1);
    let joined: Handle<IntPatchWLine> = lines[0].cast();
    assert!(joined.nb_pnts().unwrap() >= 11);
    assert_eq!(
        IntPatchWLineTool::MY_MAX_CONCAT_ANGLE,
        std::f64::consts::PI / 6.0
    );
}

#[test]
fn line_constructor_splits_a_walking_line_inside_the_domains() {
    let (s1, s2) = (
        bounded_plane(z(), (0.0, 4.0, 0.0, 10.0)),
        bounded_plane(y(), (-5.0, 5.0, -5.0, 5.0)),
    );
    let (d1, d2) = (
        Adaptor3dTopolTool::with_surface(&s1).unwrap(),
        Adaptor3dTopolTool::with_surface(&s2).unwrap(),
    );
    let line: Handle<dyn IntPatchLine> = straight(10, 0.5, 0.2).cast();
    let mut lc = IntPatchLineConstructor::new(2);
    lc.perform(&[], &line, &s1, &d1, &s2, &d2, 1e-7).unwrap();
    assert_eq!(lc.nb_lines(), 1);
    assert_eq!(lc.line(1).unwrap().arc_type(), IntPatchIType::Walking);
    assert_eq!(
        lc.line(2).err().map(|e| e.exception_type().to_string()),
        Some("Standard_OutOfRange".to_string())
    );
}

#[test]
fn prm_prm_grid_members() {
    let pp = IntPatchPrmPrmIntersection::new();
    let t = pp.grille_integer(5, 3, 1);
    assert_eq!(pp.integer_grille(t), (5, 3, 1));
    assert_eq!(
        (pp.dans_grille(-1), pp.dans_grille(0), pp.dans_grille(128)),
        (0, 1, 0)
    );
    assert_eq!(pp.nb_points_grille(), 128);
    assert_eq!(pp.code_reject([0.0; 3], [1.0; 3], [-1.0, 2.0, 0.5]), 1 | 16);
    let mut map = IntPatchPrmPrmIntersectionT3Bits::new(128);
    pp.remplit_lin([0, 0, 0], [4, 0, 0], &mut map);
    // Each halving sets only the first cell of its segments, so the odd cells between two
    // neighbours are left out, as natively.
    let set: Vec<bool> = (0..=4)
        .map(|x| map.val(pp.grille_integer(x, 0, 0)) != 0)
        .collect();
    assert_eq!(set, vec![true, false, true, false, true]);
    assert!(!pp.is_done());
    assert_eq!(
        pp.nb_lines().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}

#[test]
fn point_depart_of_two_crossing_planes() {
    let (s1, s2) = (
        bounded_plane(z(), (-1.0, 1.0, -1.0, 1.0)),
        bounded_plane(y(), (-1.0, 1.0, -1.0, 1.0)),
    );
    let points = Handle::new(IntSurfLineOn2S::new(None));
    IntPatchPrmPrmIntersection::new()
        .point_depart(&points, &s1, (1, 1), &s2, (1, 1))
        .unwrap();
    assert!(points.nb_points() > 0);
    // The start points are near the common line (the x axis).
    let p = points.value(1).unwrap().value();
    assert!(p.y().abs() < 0.2 && p.z().abs() < 0.2);
}
