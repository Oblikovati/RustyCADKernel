//! BRepAlgoAPI_Fuse_Test cases ComplexProfileWithScaling_E1 .. BoxWithPrismFromBottomExternal_F3.
#![allow(non_snake_case)]

use super::*;

// Test bfuse_simple/E1: Complex profile with scaling (pro7637 bug)
#[test]
fn BFuseSimpleTest__ComplexProfileWithScaling_E1() {
    let SCALE = 100.0;

    // Create first profile: "profile f1 c 50*SCALE 180 x -100*SCALE c 50*SCALE 180"
    let aPlane1 = Plane3::from_position(Axis3::from_axis2(
        Axis2::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        )
        .unwrap(),
    ));
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::C, &[50.0 * SCALE, 180.0]),
        ProfileOperation::new(ProfileCmd::X, &[-100.0 * SCALE]),
        ProfileOperation::new(ProfileCmd::C, &[50.0 * SCALE, 180.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, 30.0 * SCALE));

    // Create second profile: "profile f2 x 300*SCALE y 200*SCALE x -300*SCALE; ttranslate f2
    // -200*SCALE -50*SCALE 0"
    let aPlane2 = Plane3::from_position(Axis3::from_axis2(
        Axis2::new(
            Point3::new(-200.0 * SCALE, -50.0 * SCALE, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        )
        .unwrap(),
    ));
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::X, &[300.0 * SCALE]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0 * SCALE]),
        ProfileOperation::new(ProfileCmd::X, &[-300.0 * SCALE]),
        ProfileOperation::new(ProfileCmd::Y, &[-200.0 * SCALE]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -50.0 * SCALE));

    let aResult = PerformFuse(&aPrism2, &aPrism1);
    ValidateResult(&aResult, 1.85425e+09, -1.0, false);
}

// Test bfuse_simple/E2: Adjacent boxes (buc40054 bug)
#[test]
fn BFuseSimpleTest__AdjacentBoxes_E2() {
    let aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 3.0, 3.0, 3.0);
    let aBox2 = CreateBox(Point3::new(0.0, 3.0, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 58.0, -1.0, false);
}

// Test bfuse_simple/E3: Complex vertex/edge/wire construction with revolution
#[test]
fn BFuseSimpleTest__ComplexVertexEdgeWireRevolution_E3() {
    // Create profile points for the wire
    let aPoints = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(9.0, 0.0, 0.0),
        Point3::new(9.0, 0.0, 3.0),
        Point3::new(6.25, 0.0, 3.0),
        Point3::new(6.0, 0.0, 4.0),
        Point3::new(0.0, 0.0, 4.0),
    ];

    // Create wire from points
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create revolution: "revol cyla pa 0 0 0 0 0 1 360"
    let aRevolution = CreateRevolution(
        &aFace,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        2.0 * PI,
    );

    // Create cylinder: "pcylinder cylb 1 9; ttranslate cylb 5 0 -2"
    let mut aCylinder = CreateCylinder(1.0, 9.0);
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(5.0, 0.0, -2.0));
    aCylinder.move_by(&Location::new(aTrsf), false).unwrap();

    let aResult = PerformFuse(&aRevolution, &aCylinder);
    ValidateResult(&aResult, 740.048, -1.0, false);
}

// Test bfuse_simple/E4: Cylinder with complex wire revolution
#[test]
fn BFuseSimpleTest__CylinderWithComplexWireRevolution_E4() {
    // Create cylinder: "pcylinder cyl 3 5"
    let aCylinder = CreateCylinder(3.0, 5.0);

    // Create profile points for rectangular profile
    let aPoints = [
        Point3::new(0.0, 3.0, 2.0),
        Point3::new(0.0, 4.0, 2.0),
        Point3::new(0.0, 4.0, 3.0),
        Point3::new(0.0, 3.0, 3.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create revolution: "revol ring f 0 0 0 0 0 1 269"
    let aRing = CreateRevolution(
        &aFace,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        269.0 * PI / 180.0,
    );

    let aResult = PerformFuse(&aCylinder, &aRing);
    ValidateResult(&aResult, 190.356, -1.0, false);
}

// Test bfuse_simple/E5: Box with prism from vertex/edge construction
#[test]
fn BFuseSimpleTest__BoxWithPrismFromVertexEdge_E5() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for rectangular prism
    let aPoints = [
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(4.0, 2.0, 0.0),
        Point3::new(4.0, 3.0, 0.0),
        Point3::new(3.0, 3.0, 0.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 0 1"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 0.0, 1.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/E6: Box with prism from front
#[test]
fn BFuseSimpleTest__BoxWithPrismFromFront_E6() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for front prism
    let aPoints = [
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(4.0, 2.0, 0.0),
        Point3::new(4.0, 2.0, 1.0),
        Point3::new(3.0, 2.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 1 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 1.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/E7: Box with prism from left
#[test]
fn BFuseSimpleTest__BoxWithPrismFromLeft_E7() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for left prism
    let aPoints = [
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(3.0, 3.0, 1.0),
        Point3::new(3.0, 2.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 1 0 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/E8: Box with prism from top
#[test]
fn BFuseSimpleTest__BoxWithPrismFromTop_E8() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for top prism
    let aPoints = [
        Point3::new(3.0, 2.0, 1.0),
        Point3::new(4.0, 2.0, 1.0),
        Point3::new(4.0, 3.0, 1.0),
        Point3::new(3.0, 3.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 0 -1"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 0.0, -1.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/E9: Box with prism from back
#[test]
fn BFuseSimpleTest__BoxWithPrismFromBack_E9() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for back prism
    let aPoints = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(4.0, 3.0, 0.0),
        Point3::new(4.0, 3.0, 1.0),
        Point3::new(3.0, 3.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 -1 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/F1: Box with prism from right
#[test]
fn BFuseSimpleTest__BoxWithPrismFromRight_F1() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for right prism
    let aPoints = [
        Point3::new(4.0, 2.0, 0.0),
        Point3::new(4.0, 3.0, 0.0),
        Point3::new(4.0, 3.0, 1.0),
        Point3::new(4.0, 2.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f -1 0 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(-1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/F2: Box with prism from bottom (different position)
#[test]
fn BFuseSimpleTest__BoxWithPrismFromBottomDifferent_F2() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for bottom prism (different position)
    let aPoints = [
        Point3::new(4.0, 3.0, 0.0),
        Point3::new(4.0, 2.0, 0.0),
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(3.0, 3.0, 0.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 0 1"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 0.0, 1.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/F3: Box with prism from bottom (external position)
#[test]
fn BFuseSimpleTest__BoxWithPrismFromBottomExternal_F3() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for bottom prism (external position)
    let aPoints = [
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(9.0, 3.0, 0.0),
        Point3::new(9.0, 4.0, 0.0),
        Point3::new(8.0, 4.0, 0.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 0 1"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 0.0, 1.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}
