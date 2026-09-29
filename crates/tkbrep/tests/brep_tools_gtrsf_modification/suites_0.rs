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

// The first six declarations of BRepTools_GTrsfModification_Test.cxx.

#[test]
fn BRepTools_GTrsfModificationTest__Triangulation_ReflectionPreservesWindingAndNormals() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aSource = makeTriangulation(true);
    aSource.get().unwrap().update_cached_min_max();
    assert!(aSource.get().unwrap().has_cached_min_max());
    aBuilder.make_face_with_triangulation(&mut aFace, &aSource).unwrap();

    let aGTrsf = makeGTrsf(gp_Mat([-2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0]));
    let aModification = GTrsfModification::new(aGTrsf);
    let mut aResult = Handle::<Triangulation>::default();
    assert!(aModification.new_triangulation(&aFace, &mut aResult).unwrap());
    assert!(!aResult.is_null());
    let aResultMesh = aResult.get().unwrap();
    let aSourceMesh = aSource.get().unwrap();
    assert!(!aResultMesh.has_cached_min_max());
    assert!(aSourceMesh.has_cached_min_max());

    let (aNode1, aNode2, aNode3) = aResultMesh.triangle(1).unwrap().get();
    assert_eq!(aNode1, 1);
    assert_eq!(aNode2, 3);
    assert_eq!(aNode3, 2);

    let anExpectedNormal = Direction3::new(0.0, -1.0 / 3.0, 1.0 / 4.0).unwrap();
    for anIndex in 1..=3 {
        assert!(aResultMesh.normal(anIndex).unwrap().angle(&anExpectedNormal) < 1.0e-6);
    }
    expect_double_eq(aResultMesh.deflection(), 1.0);
    expect_double_eq(aSourceMesh.deflection(), 0.25);
    expectPointNear(aSourceMesh.node(2).unwrap(), p(1.0, 0.0, 0.0));
}

#[test]
fn BRepTools_GTrsfModificationTest__Triangulation_NegativeUniformScalePreservesOrientation() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aSource = makeTriangulation(true);
    aBuilder.make_face_with_triangulation(&mut aFace, &aSource).unwrap();

    let mut aScale = Transform3::default();
    aScale.set_scale(p(0.0, 0.0, 0.0), -2.0).unwrap();
    let aModification = GTrsfModification::new(GeneralTransform3::from(aScale));
    let mut aResult = Handle::<Triangulation>::default();
    assert!(aModification.new_triangulation(&aFace, &mut aResult).unwrap());
    let aResultMesh = aResult.get().unwrap();

    let (aNode1, aNode2, aNode3) = aResultMesh.triangle(1).unwrap().get();
    assert_eq!(aNode1, 1);
    assert_eq!(aNode2, 3);
    assert_eq!(aNode3, 2);

    let anExpectedNormal = Direction3::new(0.0, 1.0, -1.0).unwrap();
    assert!(aResultMesh.normal(1).unwrap().angle(&anExpectedNormal) < 1.0e-6);
    expect_double_eq(aResultMesh.deflection(), 0.5);
}

#[test]
fn BRepTools_GTrsfModificationTest__Triangulation_NormalTransformationIsScaleInvariant() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aSource = makeTriangulation(true);
    aBuilder.make_face_with_triangulation(&mut aFace, &aSource).unwrap();

    let aScales = [precision::p_confusion(), precision::infinite()];
    for aScale in aScales {
        let aModification = GTrsfModification::new(makeGTrsf(gp_Mat([
            aScale, 0.0, 0.0, 0.0, aScale, 0.0, 0.0, 0.0, aScale,
        ])));
        let mut aResult = Handle::<Triangulation>::default();
        assert!(aModification.new_triangulation(&aFace, &mut aResult).unwrap());
        let aSourceNormal = aSource.get().unwrap().normal(1).unwrap();
        assert!(aResult.get().unwrap().normal(1).unwrap().angle(&aSourceNormal) < 1.0e-6);
    }
}

#[test]
fn BRepTools_GTrsfModificationTest__Triangulation_SingularAffinityHandlesNormals() {
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aSource = makeTriangulation(true);
    aBuilder.make_face_with_triangulation(&mut aFace, &aSource).unwrap();

    let aRankTwoModification =
        GTrsfModification::new(makeGTrsf(gp_Mat([2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0])));
    let mut aResult = Handle::<Triangulation>::default();
    assert!(aRankTwoModification.new_triangulation(&aFace, &mut aResult).unwrap());
    assert!(aResult.get().unwrap().has_normals());
    let aZ = Direction3::new(0.0, 0.0, 1.0).unwrap();
    assert!(aResult.get().unwrap().normal(1).unwrap().angle(&aZ) < 1.0e-6);

    let aRankOneModification =
        GTrsfModification::new(makeGTrsf(gp_Mat([2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])));
    assert!(aRankOneModification.new_triangulation(&aFace, &mut aResult).unwrap());
    assert!(aResult.get().unwrap().has_normals());
    let aSourceNormal = aSource.get().unwrap().normal(1).unwrap();
    assert!(aResult.get().unwrap().normal(1).unwrap().angle(&aSourceNormal) < 1.0e-6);
}

#[test]
fn BRepTools_GTrsfModificationTest__Surface_LocatedBezierTransformsEvaluatedGeometry() {
    let aSurface = bezierSurface([
        p(0.0, 0.0, 0.0),
        p(2.0, 0.0, 1.0),
        p(0.0, 3.0, 1.0),
        p(2.0, 3.0, 2.0),
    ]);

    let aLocationTrsf = translation(4.0, -2.0, 5.0);
    let aLocation = Location::new(aLocationTrsf);
    let aBuilder = BRepBuilder::default();
    let mut aFace = Face::default();
    let aStored: Handle<dyn GeomSurface> = aSurface.clone().cast();
    aBuilder.make_face_on_located_surface(&mut aFace, &aStored, &aLocation, 0.2).unwrap();

    let aGTrsf = GeneralTransform3::new(
        gp_Mat([2.0, 0.5, 0.0, 0.0, 3.0, 0.25, 0.0, 0.0, 4.0]),
        Xyz::new(7.0, 8.0, 9.0),
    );
    let aModification = GTrsfModification::new(aGTrsf);
    let mut aResult = NewSurface {
        rev_wires: true,
        rev_face: true,
        ..NewSurface::default()
    };
    assert!(aModification.new_surface(&aFace, &mut aResult).unwrap());
    assert!(!aResult.surface.is_null());
    assert!(aResult.location.is_identity());
    assert!(!aResult.rev_wires);
    assert!(!aResult.rev_face);

    let anExpected = aSurface.get().unwrap().value(0.35, 0.65).unwrap().transformed(aLocationTrsf);
    let anExpected = transforms(&aGTrsf, anExpected);
    expectPointNear(aResult.surface.get().unwrap().value(0.35, 0.65).unwrap(), anExpected);
    expectPointNear(aSurface.get().unwrap().pole(2, 2).unwrap(), p(2.0, 3.0, 2.0));
    expect_double_eq(aResult.tolerance, 0.8);
}

#[test]
fn BRepTools_GTrsfModificationTest__Curve_LocatedBezierPreservesRangeAndSource() {
    let aCurve = bezierCurve(&[p(0.0, 0.0, 0.0), p(1.0, 2.0, 0.0), p(3.0, 2.0, 1.0)]);

    let aLocationTrsf = translation(-3.0, 4.0, 2.0);
    let aLocation = Location::new(aLocationTrsf);
    let aBuilder = BRepBuilder::default();
    let mut anEdge = Edge::default();
    let aStored: Handle<dyn GeomCurve> = aCurve.clone().cast();
    aBuilder.make_edge_with_located_curve(&mut anEdge, &aStored, &aLocation, 0.1).unwrap();
    aBuilder.range(&anEdge, 0.2, 0.8, false).unwrap();

    let aGTrsf = GeneralTransform3::new(
        gp_Mat([1.0, 0.5, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0]),
        Xyz::new(2.0, -1.0, 5.0),
    );
    let aModification = GTrsfModification::new(aGTrsf);
    let mut aResult = NewCurve::default();
    assert!(aModification.new_curve(&anEdge, &mut aResult).unwrap());
    assert!(!aResult.curve.is_null());
    assert!(aResult.location.is_identity());
    let aResultCurve = aResult.curve.get().unwrap();
    expect_double_eq(aResultCurve.first_parameter(), 0.2);
    expect_double_eq(aResultCurve.last_parameter(), 0.8);

    let anExpected = aCurve.get().unwrap().value(0.4).unwrap().transformed(aLocationTrsf);
    let anExpected = transforms(&aGTrsf, anExpected);
    expectPointNear(aResultCurve.value(0.4).unwrap(), anExpected);
    expectPointNear(aCurve.get().unwrap().pole(2).unwrap(), p(1.0, 2.0, 0.0));
}
