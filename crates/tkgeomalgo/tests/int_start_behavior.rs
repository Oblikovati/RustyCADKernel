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

//! Behavior of the IntStart templates beyond the numeric probe, which compares them with
//! native OCCT bit for bit. The instances use the probe ports of the IntPatch classes
//! (IntPatch is not ported yet), shared with the numeric probe.
#[allow(dead_code)]
#[path = "../examples/ga_probe/int_start_tools.rs"]
mod int_start_tools;

use int_start_tools::{
    ProbeArcFunction, ProbeDomain, ProbeHInterTool, ProbePathPoint, ProbeSeenTool, ProbeSegment,
    SEEN, SeenTable,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_imp::{HSurfaceTool, IntImpZerImpFunc};
use rustycad_tkgeomalgo::int_start::{
    SearchInside, SearchInsideTypes, SearchOnBoundaries, SearchOnBoundariesTypes, ThePathPoint,
};
use rustycad_tkgeomalgo::int_surf::{IntSurfQuadric, IntSurfQuadricTool};
use rustycad_tkmath::gp::{Axis3, Cylinder3, Direction3, Point3, Sphere3};

struct PatchTypes;
impl SearchOnBoundariesTypes for PatchTypes {
    type TopolTool = ProbeDomain;
    type SOBTool = ProbeHInterTool;
    type Function = ProbeArcFunction;
    type PathPoint = ProbePathPoint;
    type Segment = ProbeSegment;
}

struct SeenTypes;
impl SearchOnBoundariesTypes for SeenTypes {
    type TopolTool = ProbeDomain;
    type SOBTool = ProbeSeenTool;
    type Function = ProbeArcFunction;
    type PathPoint = ProbePathPoint;
    type Segment = ProbeSegment;
}

struct InsideTypes;
impl SearchInsideTypes for InsideTypes {
    type SurfaceTool = HSurfaceTool;
    type TopolTool = Adaptor3dTopolTool;
    type Function = IntImpZerImpFunc<HSurfaceTool, IntSurfQuadricTool>;
}

fn frame() -> Axis3 {
    Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

/// The square [-1, 1] x [-1, 1] of the plane z = 0.
fn square() -> Adaptor3dSurfaceHandle {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame())).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&plane, (-1.0, 1.0, -1.0, 1.0)).unwrap()).cast()
}

fn sphere(radius: f64) -> IntSurfQuadric {
    IntSurfQuadric::from_sphere(&Sphere3::new(frame(), radius).unwrap())
}

fn arc_function(s: &Adaptor3dSurfaceHandle, q: &IntSurfQuadric) -> ProbeArcFunction {
    let mut f = ProbeArcFunction::default();
    f.set_quadric(q);
    f.set_surface(s);
    f
}

fn domain(s: &Adaptor3dSurfaceHandle) -> ProbeDomain {
    ProbeDomain {
        tool: Adaptor3dTopolTool::with_surface(s).unwrap(),
        edge: None,
    }
}

#[test]
fn search_on_boundaries_reads_nothing_before_perform() {
    let search = SearchOnBoundaries::<PatchTypes>::new();
    assert!(!search.is_done());
    for failure in [
        search.nb_points().err(),
        search.nb_segments().err(),
        search.all_arc_solution().err(),
        search.point(1).err(),
    ] {
        assert_eq!(failure.unwrap().exception_type(), "StdFail_NotDone");
    }
}

#[test]
fn search_on_boundaries_finds_the_circle_crossings_of_the_square() {
    let s = square();
    let (mut f, d) = (arc_function(&s, &sphere(1.2)), domain(&s));
    let mut search = SearchOnBoundaries::<PatchTypes>::new();
    search.perform(&mut f, &d, 1.0e-7, 1.0e-6, false).unwrap();
    // The circle of radius 1.2 crosses each side of the square twice.
    assert_eq!(search.nb_points().unwrap(), 8);
    assert_eq!(search.nb_segments().unwrap(), 0);
    assert!(!search.all_arc_solution().unwrap());
    for i in 1..=8 {
        let p: &ProbePathPoint = search.point(i).unwrap();
        assert!(p.is_new());
        assert!((p.value().distance(&Point3::new(0.0, 0.0, 0.0)) - 1.2).abs() < 1.0e-9);
        assert_eq!(
            p.vertex().err().unwrap().exception_type(),
            "Standard_DomainError"
        );
    }
    assert_eq!(
        search.point(9).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        search.point(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn search_on_boundaries_keeps_the_vertices_on_the_quadric() {
    let s = square();
    // The sphere of radius sqrt(2) passes through the four corners of the square.
    let (mut f, d) = (arc_function(&s, &sphere(2.0_f64.sqrt())), domain(&s));
    let mut search = SearchOnBoundaries::<PatchTypes>::new();
    search.perform(&mut f, &d, 1.0e-7, 1.0e-6, false).unwrap();
    let vertices = (1..=search.nb_points().unwrap())
        .filter(|&i| !search.point(i).unwrap().is_new())
        .count();
    assert!(
        vertices >= 4,
        "{vertices} vertex points; expected the four corners"
    );
}

#[test]
fn search_on_boundaries_takes_the_known_points_of_seen_arcs() {
    SEEN.with(|t| {
        *t.borrow_mut() = SeenTable {
            pts: vec![
                [0.0, 0.0, 0.0, 1.0e-6, 0.5, 0.0],
                [1.0, 0.0, 0.0, 1.0e-6, 0.25, 1.0],
            ],
            segs: vec![(1, 2)],
            all: true,
        }
    });
    let s = square();
    let (mut f, d) = (arc_function(&s, &sphere(1.2)), domain(&s));
    let mut search = SearchOnBoundaries::<SeenTypes>::new();
    search.perform(&mut f, &d, 1.0e-7, 1.0e-6, false).unwrap();
    // Two known points and one segment per restriction of the square.
    assert_eq!(search.nb_points().unwrap(), 8);
    assert_eq!(search.nb_segments().unwrap(), 4);
    assert!(search.all_arc_solution().unwrap());
    assert!(!search.point(2).unwrap().is_new());
    let segment: &ProbeSegment = search.segment(4).unwrap();
    assert_eq!(segment.first.as_ref().unwrap().parameter(), 0.5);
    assert_eq!(segment.last.as_ref().unwrap().parameter(), 0.25);
}

#[test]
fn search_inside_finds_points_of_the_circle_inside_the_square() {
    let s = square();
    let mut f =
        IntImpZerImpFunc::<HSurfaceTool, IntSurfQuadricTool>::with_surfaces(&s, &sphere(0.8));
    f.set_tolerance(1.0e-6);
    let d = Adaptor3dTopolTool::with_surface(&s).unwrap();
    let search = SearchInside::<InsideTypes>::with_domain(&mut f, &s, &d, 1.0e-6).unwrap();
    let n = search.nb_points().unwrap();
    assert!(n > 0);
    for i in 1..=n {
        let p = search.value(i).unwrap();
        assert!((p.value().distance(&Point3::new(0.0, 0.0, 0.0)) - 0.8).abs() < 1.0e-5);
    }
    assert_eq!(
        search.value(n + 1).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn search_inside_from_a_start_point_outside_the_bounds_finds_nothing() {
    let s = square();
    let q = IntSurfQuadric::from_cylinder(&Cylinder3::new(frame(), 0.5).unwrap());
    let mut f = IntImpZerImpFunc::<HSurfaceTool, IntSurfQuadricTool>::with_surfaces(&s, &q);
    f.set_tolerance(1.0e-6);
    let mut search = SearchInside::<InsideTypes>::new();
    assert_eq!(
        search.nb_points().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    search.perform_from(&mut f, &s, 1.5, 0.0).unwrap();
    assert!(search.is_done());
    assert_eq!(search.nb_points().unwrap(), 0);
    search.perform_from(&mut f, &s, 0.4, 0.1).unwrap();
    assert_eq!(search.nb_points().unwrap(), 1);
}
