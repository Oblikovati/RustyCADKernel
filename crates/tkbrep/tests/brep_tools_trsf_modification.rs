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

//! Direct translation of BRepTools_TrsfModification_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, TrsfModification};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{
    Axis2d, Direction2, Direction3, Plane3, Point2, Point3, Transform3, Vector3,
};
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;

fn makeTriangulation(theHasNormals: bool) -> Handle<Triangulation> {
    let aTriangulation = Triangulation::with_sizes(3, 1, false, theHasNormals).unwrap();
    aTriangulation
        .set_node(1, Point3::new(0.0, 0.0, 0.0))
        .unwrap();
    aTriangulation
        .set_node(2, Point3::new(1.0, 0.0, 0.0))
        .unwrap();
    aTriangulation
        .set_node(3, Point3::new(0.0, 1.0, 1.0))
        .unwrap();
    aTriangulation
        .set_triangle(1, Triangle::new(1, 2, 3))
        .unwrap();
    aTriangulation.set_deflection(0.25);
    if theHasNormals {
        let aNormal = Direction3::new(0.0, -1.0, 1.0).unwrap();
        for anIndex in 1..=3 {
            aTriangulation.set_normal(anIndex, aNormal).unwrap();
        }
    }
    Handle::new(aTriangulation)
}

fn makeScale(theScale: f64) -> Transform3 {
    let mut aTrsf = Transform3::default();
    aTrsf
        .set_scale(Point3::new(0.0, 0.0, 0.0), theScale)
        .unwrap();
    aTrsf
}

fn expectPointNear(theActual: Point3, theExpected: Point3) {
    assert!(theActual.distance(&theExpected) < 1.0e-12);
}

fn plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::from_pln(Plane3::default())).cast()
}

#[test]
fn BRepTools_TrsfModificationTest__CopyMeshFlag_SkipsCachedMeshOnGeometricFace() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aPlane = plane();
    let aTriangulation = makeTriangulation(false);
    aBuilder
        .make_face_on_surface(&mut aFace, &aPlane, 0.1)
        .unwrap();
    aBuilder
        .update_face_triangulation(&aFace, &aTriangulation, true)
        .unwrap();

    let aModification = TrsfModification::new(makeScale(2.0));
    let mut aResult = makeTriangulation(false);
    let aSentinel = aResult.clone();
    assert!(
        !aModification
            .new_triangulation(&aFace, &mut aResult)
            .unwrap()
    );
    assert!(aResult == aSentinel);

    aModification.set_copy_mesh(true);
    assert!(
        aModification
            .new_triangulation(&aFace, &mut aResult)
            .unwrap()
    );
    let aResultMesh = aResult.get().unwrap();
    expectPointNear(aResultMesh.node(2).unwrap(), Point3::new(2.0, 0.0, 0.0));
    assert_eq!(aResultMesh.deflection(), 0.5);
    expectPointNear(
        aTriangulation.get().unwrap().node(2).unwrap(),
        Point3::new(1.0, 0.0, 0.0),
    );
}

#[test]
fn BRepTools_TrsfModificationTest__Polygon3D_DistinctRepresentationLocation() {
    let mut aNodes: Array1<'static, Point3> = Array1::with_bounds(1, 2).unwrap();
    aNodes.set_value(1, Point3::new(1.0, 2.0, 3.0)).unwrap();
    aNodes.set_value(2, Point3::new(4.0, 5.0, 6.0)).unwrap();
    let aPolygon = Handle::new(Polygon3D::from_nodes(&aNodes));

    let mut aRepresentationTrsf = Transform3::default();
    aRepresentationTrsf.set_translation(Vector3::new(10.0, 0.0, 0.0));
    let mut anEdgeTrsf = Transform3::default();
    anEdgeTrsf.set_translation(Vector3::new(0.0, 20.0, 0.0));
    let aBuilder = BRepBuilder::default();
    let mut anEdge = Edge::default();
    aBuilder.make_edge(&mut anEdge).unwrap();
    aBuilder
        .update_edge_polygon3d(&anEdge, &aPolygon, &Location::new(aRepresentationTrsf))
        .unwrap();
    anEdge
        .set_location(&Location::new(anEdgeTrsf), false)
        .unwrap();

    let aModification = TrsfModification::new(makeScale(2.0));
    aModification.set_copy_mesh(true);
    let mut aResult = Handle::<Polygon3D>::default();
    assert!(aModification.new_polygon(&anEdge, &mut aResult).unwrap());

    let (aSourcePolygon, aSourceLocation) = BRepTool::polygon3d(&anEdge).unwrap();
    assert!(!aSourcePolygon.is_null());
    let aResultPolygon = aResult.get().unwrap();
    for anIndex in 1..=aResultPolygon.nb_nodes() {
        let mut anExpected = aPolygon
            .get()
            .unwrap()
            .nodes()
            .value(anIndex)
            .unwrap()
            .transformed(aSourceLocation.transformation());
        anExpected.transform(makeScale(2.0));
        let anActual = aResultPolygon
            .nodes()
            .value(anIndex)
            .unwrap()
            .transformed(anEdge.location().transformation());
        expectPointNear(anActual, anExpected);
    }
}

#[test]
fn BRepTools_TrsfModificationTest__ParameterlessPolygonOnTriangulationDoesNotThrow() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let mut anEdge = Edge::default();
    let aPlane = plane();
    let aTriangulation = makeTriangulation(false);
    aBuilder
        .make_face_on_surface(&mut aFace, &aPlane, 0.1)
        .unwrap();
    aBuilder
        .update_face_triangulation(&aFace, &aTriangulation, true)
        .unwrap();

    let mut aNodes: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aNodes.set_value(1, 1).unwrap();
    aNodes.set_value(2, 2).unwrap();
    let aPolygon = Handle::new(PolygonOnTriangulation::from_nodes(&aNodes));
    aPolygon.get().unwrap().set_deflection(0.25);
    aBuilder
        .make_edge_on_triangulation(
            &mut anEdge,
            &aPolygon,
            &aTriangulation,
            &Location::default(),
        )
        .unwrap();
    let aPcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    )))
    .cast();
    aBuilder
        .update_edge_pcurve_on_face(&anEdge, &aPcurve, &aFace, 0.1)
        .unwrap();
    aBuilder.range_on_face(&anEdge, &aFace, 0.0, 1.0).unwrap();

    let aModification = TrsfModification::new(makeScale(2.0));
    aModification.set_copy_mesh(true);
    let mut aResult = Handle::<PolygonOnTriangulation>::default();
    assert!(
        aModification
            .new_polygon_on_triangulation(&anEdge, &aFace, &mut aResult)
            .unwrap()
    );
    let aResultPolygon = aResult.get().unwrap();
    assert!(!aResultPolygon.has_parameters());
    assert_eq!(aResultPolygon.deflection(), 0.5);
}

#[test]
fn BRepTools_TrsfModificationTest__NegativeScaleUpdatesWindingNormalsAndDeflection() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aTriangulation = makeTriangulation(true);
    aBuilder
        .make_face_with_triangulation(&mut aFace, &aTriangulation)
        .unwrap();

    let aModification = TrsfModification::new(makeScale(-2.0));
    aModification.set_copy_mesh(true);
    let mut aResult = Handle::<Triangulation>::default();
    assert!(
        aModification
            .new_triangulation(&aFace, &mut aResult)
            .unwrap()
    );
    let aResultMesh = aResult.get().unwrap();
    let (aNode1, aNode2, aNode3) = aResultMesh.triangle(1).unwrap().get();
    assert_eq!(aNode1, 1);
    assert_eq!(aNode2, 3);
    assert_eq!(aNode3, 2);
    assert!(
        aResultMesh
            .normal(1)
            .unwrap()
            .angle(&Direction3::new(0.0, 1.0, -1.0).unwrap())
            < 1.0e-6
    );
    assert_eq!(aResultMesh.deflection(), 0.5);
}
