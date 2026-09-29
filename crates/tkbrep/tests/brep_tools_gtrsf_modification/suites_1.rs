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

// The last seven declarations of BRepTools_GTrsfModification_Test.cxx.

#[test]
fn BRepTools_GTrsfModificationTest__Polygon3D_RepresentationLocationTransformsToEdgeFrame() {
    let mut aNodes: Array1<'static, Point3> = Array1::with_bounds(1, 2).unwrap();
    aNodes.set_value(1, p(1.0, 2.0, 3.0)).unwrap();
    aNodes.set_value(2, p(4.0, 5.0, 6.0)).unwrap();
    let mut aParameters: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aParameters.set_value(1, 0.25).unwrap();
    aParameters.set_value(2, 0.75).unwrap();
    let aPolygon = Handle::new(Polygon3D::from_nodes_and_parameters(&aNodes, &aParameters).unwrap());
    aPolygon.get().unwrap().set_deflection(0.5);

    let aRepresentationTrsf = translation(10.0, 0.0, 0.0);
    let anEdgeTrsf = translation(0.0, 20.0, 0.0);
    let aBuilder = BRepBuilder::default();
    let mut anEdge = Edge::default();
    aBuilder.make_edge(&mut anEdge).unwrap();
    aBuilder
        .update_edge_polygon3d(&anEdge, &aPolygon, &Location::new(aRepresentationTrsf))
        .unwrap();
    anEdge.set_location(&Location::new(anEdgeTrsf), false).unwrap();

    let aGTrsf = makeGTrsf(gp_Mat([2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0]));
    let aModification = GTrsfModification::new(aGTrsf);
    let mut aResult = Handle::<Polygon3D>::default();
    assert!(aModification.new_polygon(&anEdge, &mut aResult).unwrap());
    assert!(!aResult.is_null());

    let (aSourcePolygon, aSourceLocation) = BRepTool::polygon3d(&anEdge).unwrap();
    assert!(!aSourcePolygon.is_null());
    let aResultPolygon = aResult.get().unwrap();
    let aSourceNodes = aPolygon.get().unwrap().nodes();
    for anIndex in 1..=aResultPolygon.nb_nodes() {
        let anExpected =
            aSourceNodes.value(anIndex).unwrap().transformed(aSourceLocation.transformation());
        let anExpected = transforms(&aGTrsf, anExpected);
        let anActual = aResultPolygon
            .nodes()
            .value(anIndex)
            .unwrap()
            .transformed(anEdge.location().transformation());
        expectPointNear(anActual, anExpected);
        expect_double_eq(
            *aResultPolygon.parameters().unwrap().value(anIndex).unwrap(),
            *aParameters.value(anIndex).unwrap(),
        );
    }
    expect_double_eq(aResultPolygon.deflection(), 2.0);
    expect_double_eq(aPolygon.get().unwrap().deflection(), 0.5);
}

#[test]
fn BRepTools_GTrsfModificationTest__MissingPolygonRepresentationsReturnFalse() {
    let aBuilder = BRepBuilder::default();
    let mut anEdge = Edge::default();
    let mut aFace = Face::default();
    aBuilder.make_edge(&mut anEdge).unwrap();
    let aTriangulation = makeTriangulation(false);
    aBuilder.make_face_with_triangulation(&mut aFace, &aTriangulation).unwrap();

    let aModification = GTrsfModification::new(GeneralTransform3::default());
    let mut aPolygon3d = Handle::new(Polygon3D::new(2, false).unwrap());
    let mut aPolygonOnTriangulation = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    assert!(!aModification.new_polygon(&anEdge, &mut aPolygon3d).unwrap());
    assert!(aPolygon3d.is_null());
    assert!(!aModification
        .new_polygon_on_triangulation(&anEdge, &aFace, &mut aPolygonOnTriangulation)
        .unwrap());
    assert!(aPolygonOnTriangulation.is_null());
}

#[test]
fn BRepTools_GTrsfModificationTest__Curve2dAndVertexParameterPreserveParametrization() {
    let aSurface: Handle<dyn GeomSurface> = bezierSurface([
        p(0.0, 0.0, 0.0),
        p(1.0, 0.0, 0.0),
        p(0.0, 1.0, 0.0),
        p(1.0, 1.0, 0.0),
    ])
    .cast();
    let aCurve = bezierCurve(&[p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0)]);

    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let mut anEdge = Edge::default();
    let mut aVertex = Vertex::default();
    aBuilder.make_face_on_surface(&mut aFace, &aSurface, 0.05).unwrap();
    let aStored: Handle<dyn GeomCurve> = aCurve.clone().cast();
    aBuilder.make_edge_with_curve(&mut anEdge, &aStored, 0.05).unwrap();
    aBuilder.range(&anEdge, 0.2, 0.8, false).unwrap();
    let aLine = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    )));
    let aPcurve: Handle<dyn Geom2dCurve> = aLine.clone().cast();
    aBuilder.update_edge_pcurve_on_face(&anEdge, &aPcurve, &aFace, 0.05).unwrap();
    aBuilder.range_on_face(&anEdge, &aFace, 0.2, 0.8).unwrap();
    let aPoint = aCurve.get().unwrap().value(0.25).unwrap();
    aBuilder.make_vertex_at(&mut aVertex, aPoint, 0.02).unwrap();
    let mut anEdgeShape = anEdge.clone().into();
    aBuilder.add(&mut anEdgeShape, &aVertex.clone().into()).unwrap();
    aBuilder.update_vertex_on_edge(&aVertex, 0.25, &anEdge, 0.02).unwrap();

    let aSource = BRepTool::curve_on_face(&anEdge, &aFace).unwrap();
    assert!(!aSource.pcurve.is_null());
    let (aSourceFirst, aSourceLast) = (aSource.first, aSource.last);

    let aGTrsf = makeGTrsf(gp_Mat([2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 1.0]));
    let aModification = GTrsfModification::new(aGTrsf);
    let mut aResultCurve = Handle::<dyn Geom2dCurve>::default();
    let mut aTolerance = 0.0;
    assert!(aModification
        .new_curve2d(
            &anEdge,
            &aFace,
            &Edge::default(),
            &Face::default(),
            &mut aResultCurve,
            &mut aTolerance,
        )
        .unwrap());
    assert!(!aResultCurve.is_null());
    let aResult = aResultCurve.get().unwrap();
    expect_double_eq(aResult.first_parameter(), aSourceFirst);
    expect_double_eq(aResult.last_parameter(), aSourceLast);
    let aSourceValue = aLine.get().unwrap().value(0.4).unwrap();
    assert!(aResult.value(0.4).unwrap().distance(aSourceValue) < 1.0e-12);
    expect_double_eq(aTolerance, 0.15);

    let mut aParameter = 0.0;
    assert!(aModification
        .new_parameter(&aVertex, &anEdge, &mut aParameter, &mut aTolerance)
        .unwrap());
    expect_double_eq(aParameter, 0.25);
    expect_double_eq(aTolerance, 0.06);
}

#[test]
fn BRepTools_GTrsfModificationTest__Modifier_LocatedSurfaceAndMeshStayCoincident() {
    let aSurface = bezierSurface([
        p(0.0, 0.0, 0.0),
        p(1.0, 0.0, 0.0),
        p(0.0, 1.0, 1.0),
        p(1.0, 1.0, 1.0),
    ]);
    let aTriangulation = makeTriangulation(false);

    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aStored: Handle<dyn GeomSurface> = aSurface.clone().cast();
    aBuilder.make_face_on_surface(&mut aFace, &aStored, 0.1).unwrap();
    aBuilder.update_face_triangulation(&aFace, &aTriangulation, true).unwrap();
    let aLocationTrsf = translation(10.0, 20.0, 30.0);
    aFace.set_location(&Location::new(aLocationTrsf), false).unwrap();

    let aGTrsf = GeneralTransform3::new(
        gp_Mat([2.0, 0.5, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0]),
        Xyz::new(5.0, 6.0, 7.0),
    );
    let aModification: Handle<dyn BRepToolsModification> =
        Handle::new(GTrsfModification::new(aGTrsf)).cast();
    let aModifier = Modifier::with_modification(&aFace.clone().into(), &aModification).unwrap();
    assert!(aModifier.is_done());
    let aResultFace = topods::face(&aModifier.modified_shape(&aFace.clone().into()).unwrap()).unwrap();
    assert!(!aResultFace.is_null());

    let (aResultSurface, aResultSurfaceLocation) =
        BRepTool::surface_with_location(&aResultFace).unwrap();
    assert!(!aResultSurface.is_null());
    let anExpectedSurface =
        aSurface.get().unwrap().value(0.25, 0.75).unwrap().transformed(aLocationTrsf);
    let anExpectedSurface = transforms(&aGTrsf, anExpectedSurface);
    expectPointNear(
        aResultSurface
            .get()
            .unwrap()
            .value(0.25, 0.75)
            .unwrap()
            .transformed(aResultSurfaceLocation.transformation()),
        anExpectedSurface,
    );

    let (aResultTriangulation, aResultMeshLocation) =
        BRepTool::triangulation(&aResultFace, Default::default()).unwrap();
    assert!(!aResultTriangulation.is_null());
    let aSourceMesh = aTriangulation.get().unwrap();
    for anIndex in 1..=aSourceMesh.nb_nodes() {
        let anExpectedNode = aSourceMesh.node(anIndex).unwrap().transformed(aLocationTrsf);
        let anExpectedNode = transforms(&aGTrsf, anExpectedNode);
        expectPointNear(
            aResultTriangulation
                .get()
                .unwrap()
                .node(anIndex)
                .unwrap()
                .transformed(aResultMeshLocation.transformation()),
            anExpectedNode,
        );
    }
}

#[test]
fn BRepTools_GTrsfModificationTest__Triangulation_LocatedFaceUsesLocalCoordinates() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aSource = makeTriangulation(false);
    aBuilder.make_face_with_triangulation(&mut aFace, &aSource).unwrap();

    let aLocationTrsf = translation(10.0, 20.0, 30.0);
    aFace.set_location(&Location::new(aLocationTrsf), false).unwrap();

    let aGTrsf = makeGTrsf(gp_Mat([2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0]));
    let aModification = GTrsfModification::new(aGTrsf);
    let mut aResult = Handle::<Triangulation>::default();
    assert!(aModification.new_triangulation(&aFace, &mut aResult).unwrap());

    let (aFound, aResultLocation) = BRepTool::triangulation(&aFace, Default::default()).unwrap();
    assert!(!aFound.is_null());
    let aSourceMesh = aSource.get().unwrap();
    for anIndex in 1..=aSourceMesh.nb_nodes() {
        let anExpected =
            aSourceMesh.node(anIndex).unwrap().transformed(aResultLocation.transformation());
        let anExpected = transforms(&aGTrsf, anExpected);
        let anActual = aResult
            .get()
            .unwrap()
            .node(anIndex)
            .unwrap()
            .transformed(aResultLocation.transformation());
        assert!(anActual.distance(&anExpected) < 1.0e-12);
    }
}

#[test]
fn BRepTools_GTrsfModificationTest__Tolerance_TracksCurrentTransformation() {
    let aBuilder = BRepBuilder::default();
    let mut aVertex = Vertex::default();
    aBuilder.make_vertex_at(&mut aVertex, p(1.0, 2.0, 3.0), 0.1).unwrap();

    let aModification = GTrsfModification::new(GeneralTransform3::default());
    aModification.set_gtrsf(makeGTrsf(gp_Mat([2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0])));
    let mut aPoint = Point3::default();
    let mut aTolerance = 0.0;
    assert!(aModification.new_point(&aVertex, &mut aPoint, &mut aTolerance).unwrap());
    expect_double_eq(aTolerance, 0.4);
    expectPointNear(aPoint, p(2.0, 6.0, 12.0));

    aModification.set_gtrsf(makeGTrsf(gp_Mat([1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])));
    assert!(aModification.new_point(&aVertex, &mut aPoint, &mut aTolerance).unwrap());
    expect_double_eq(aTolerance, 0.1);
    expectPointNear(aPoint, p(3.0, 2.0, 3.0));

    let aCos = 0.5_f64.sqrt();
    aModification.set_gtrsf(makeGTrsf(gp_Mat([aCos, -aCos, 0.0, aCos, aCos, 0.0, 0.0, 0.0, 1.0])));
    assert!(aModification.new_point(&aVertex, &mut aPoint, &mut aTolerance).unwrap());
    assert!((aTolerance - 0.1).abs() <= 1.0e-14);

    let mut aScale = Transform3::default();
    aScale.set_scale(p(0.0, 0.0, 0.0), 3.0).unwrap();
    aModification.set_gtrsf(GeneralTransform3::from(aScale));
    assert!(aModification.new_point(&aVertex, &mut aPoint, &mut aTolerance).unwrap());
    expect_double_eq(aTolerance, 0.3);
    expectPointNear(aPoint, p(3.0, 6.0, 9.0));
}

#[test]
fn BRepTools_GTrsfModificationTest__PolygonOnTriangulation_PreservesDeflection() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let mut anEdge = Edge::default();
    let aTriangulation = makeTriangulation(false);
    aBuilder.make_face_with_triangulation(&mut aFace, &aTriangulation).unwrap();

    let mut aNodes: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aNodes.set_value(1, 1).unwrap();
    aNodes.set_value(2, 2).unwrap();
    let aPolygon = Handle::new(PolygonOnTriangulation::from_nodes(&aNodes));
    aPolygon.get().unwrap().set_deflection(0.25);
    aBuilder
        .make_edge_on_triangulation(&mut anEdge, &aPolygon, &aTriangulation, &Location::default())
        .unwrap();

    let aGTrsf = makeGTrsf(gp_Mat([2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0]));
    let aModification = GTrsfModification::new(aGTrsf);
    let mut aResult = Handle::<PolygonOnTriangulation>::default();
    assert!(aModification
        .new_polygon_on_triangulation(&anEdge, &aFace, &mut aResult)
        .unwrap());
    assert!(!aResult.is_null());
    expect_double_eq(aResult.get().unwrap().deflection(), 0.25);
}
