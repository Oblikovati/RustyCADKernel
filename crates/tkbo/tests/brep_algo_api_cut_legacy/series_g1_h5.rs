//! BRepAlgoAPI_Cut_Test cases BoxMinusSphere_G1 .. ComplexProfileForwardForwardVariation_H5.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/G1: box - sphere
#[test]
fn BCutSimpleTest__BoxMinusSphere_G1() {
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let aResult = PerformCut(&aBox, &aSphere);
    ValidateResult(&aResult, 5.2146, -1.0, false);
}

// Test bcut_simple/G2: rotated sphere - box
#[test]
fn BCutSimpleTest__RotatedSphereMinusBox_G2() {
    let mut aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    aSphere = RotateShape(
        &aSphere,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        -90.0 * PI / 180.0,
    );
    aSphere = RotateShape(
        &aSphere,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 1.0, 0.0).unwrap(),
        ),
        -45.0 * PI / 180.0,
    );
    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let aResult = PerformCut(&aSphere, &aBox);
    ValidateResult(&aResult, 13.3517, -1.0, false);
}

// Test bcut_simple/G3: box - rotated sphere
#[test]
fn BCutSimpleTest__BoxMinusRotatedSphere_G3() {
    let mut aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    aSphere = RotateShape(
        &aSphere,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        -90.0 * PI / 180.0,
    );
    aSphere = RotateShape(
        &aSphere,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 1.0, 0.0).unwrap(),
        ),
        -45.0 * PI / 180.0,
    );
    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let aResult = PerformCut(&aBox, &aSphere);
    ValidateResult(&aResult, 5.2146, -1.0, false);
}

// Test bcut_simple/G4: sphere - rotated box
#[test]
fn BCutSimpleTest__SphereMinusRotatedBox_G4() {
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    let mut aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox = RotateShape(
        &aBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 1.0),
            Direction3::new(0.0, 1.0, 0.0).unwrap(),
        ),
        90.0 * PI / 180.0,
    );
    let aResult = PerformCut(&aSphere, &aBox);
    ValidateResult(&aResult, 13.3517, -1.0, false);
}

// Test bcut_simple/G5: rotated box - sphere
#[test]
fn BCutSimpleTest__RotatedBoxMinusSphere_G5() {
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    let mut aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox = RotateShape(
        &aBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 1.0),
            Direction3::new(0.0, 1.0, 0.0).unwrap(),
        ),
        90.0 * PI / 180.0,
    );
    let aResult = PerformCut(&aBox, &aSphere);
    ValidateResult(&aResult, 5.2146, -1.0, false);
}

// Test bcut_simple/G6: Complex profile/revol operation (pro5363 bug)
#[test]
fn BCutSimpleTest__ComplexProfileRevolOperation_G6() {
    // Create box: "box b 100 100 40"
    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 100.0, 100.0, 40.0);

    // Create exact G6 profile: "profile rev S b_4 F 50 20 Y 50 C 10 180 Y -50 C 10 180"
    let aPlane = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps = [
        ProfileOperation::new(ProfileCmd::F, &[50.0, 20.0]),
        ProfileOperation::new(ProfileCmd::Y, &[50.0]),
        ProfileOperation::new(ProfileCmd::C, &[10.0, 180.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-50.0]),
        ProfileOperation::new(ProfileCmd::C, &[10.0, 180.0]),
    ];
    let aProfile = CreateProfile(&aPlane, &aProfileOps);

    // Create revolution: "revol rev2 rev 0 0 50 0 1 0 360" (around Y-axis at Z=50)
    let aRevolution = CreateRevolution(
        &aProfile,
        Axis1::new(
            Point3::new(0.0, 0.0, 50.0),
            Direction3::new(0.0, 1.0, 0.0).unwrap(),
        ),
        2.0 * PI,
    );

    // Perform boolean cut operation: "bcut result b rev2"
    let aResult = PerformCut(&aBox, &aRevolution);
    ValidateResult(&aResult, 41187.4, -1.0, false);
}

// Test bcut_simple/G7: Box minus translated box (buc40054 bug)
#[test]
fn BCutSimpleTest__BoxMinusTranslatedBox_G7() {
    let aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 3.0, 3.0, 3.0);
    let aBox2 = CreateBox(Point3::new(0.0, 3.0, 0.0), 1.0, 1.0, 1.0);
    let aResult = PerformCut(&aBox1, &aBox2);
    ValidateResult(&aResult, 54.0, -1.0, false);
}

// Test bcut_simple/G8: Box minus prism (BUC40188 bug)
#[test]
fn BCutSimpleTest__BoxMinusPrism_G8() {
    // Create box mb (-0.5, -0.5, -0.5, 1, 1, 1)
    let aBoxMb = CreateBox(Point3::new(-0.5, -0.5, -0.5), 1.0, 1.0, 1.0);

    // Get face mb_1 (approximated as first face)
    let aFace = GetFaceByIndex(&aBoxMb, 1);

    // Create semi-infinite prism (simulated with very large prism: prism pryz mb_1 1 0 0 SemiInf)
    // Semi-infinite prism approximated as large prism extending 1000 units in X direction
    let aSemiInfPrism = CreatePrism(&aFace, Vector3::new(1000.0, 0.0, 0.0));

    // Create box ab (0, -1, -1, 2, 2, 2)
    let aBoxAb = CreateBox(Point3::new(0.0, -1.0, -1.0), 2.0, 2.0, 2.0);

    // Perform boolean cut operation: bcut result ab pryz
    let aResult = PerformCut(&aBoxAb, &aSemiInfPrism);
    ValidateResult(&aResult, 30.0, -1.0, false);
}

// Test bcut_simple/G9: Complex cylinder/cone fuse minus translated cylinder
#[test]
fn BCutSimpleTest__ComplexCylinderConeOperation_G9() {
    // Create cylinder and cone, fuse them
    let aCylinder = CreateCylinder(9.0, 3.0);
    let aCone = CreateCone(7.0, 6.0, 4.0);

    let aFuseOp = BRepAlgoAPIFuse::from_shapes(&aCylinder, &aCone).unwrap();
    assert!(aFuseOp.is_done(), "Fuse operation failed");
    let aBody = aFuseOp.shape().clone();

    // Create small cylinder and translate it
    let mut aSmallCylinder = CreateCylinder(1.0, 9.0);
    aSmallCylinder = TranslateShape(&aSmallCylinder, Vector3::new(5.0, 0.0, -2.0));

    let aResult = PerformCut(&aBody, &aSmallCylinder);
    ValidateResult(&aResult, 727.481, -1.0, false);
}

// Test bcut_simple/H1: Complex custom polygon prism minus box (MFA bug collection)
#[test]
fn BCutSimpleTest__ComplexPolygonPrismMinusBox_H1() {
    // Create custom polygon from H1 test vertices: v1(0,0,0) v2(1,0,0) v3(1,3,0) v4(2,3,0) v5(2,0,0)
    // v6(3,0,0) v7(3,5,0) v8(0,5,0)
    let aPoints = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 3.0, 0.0),
        Point3::new(2.0, 3.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(3.0, 5.0, 0.0),
        Point3::new(0.0, 5.0, 0.0),
    ];

    // Create wire and extrude to solid (prism sol p 0 0 2)
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);
    let aSolid = CreatePrism(&aFace, Vector3::new(0.0, 0.0, 2.0));

    // Create box: box b -1 2 1 5 1 3 (corner at (-1,2,1), dimensions 5x1x3)
    let aBox = CreateBox(Point3::new(-1.0, 2.0, 1.0), 5.0, 1.0, 3.0);

    // Perform boolean cut: bcut result sol b
    let aResult = PerformCut(&aSolid, &aBox);
    ValidateResult(&aResult, 68.0, -1.0, false);
}

// Test bcut_simple/H2: Complex custom polygon prism minus box (pro13306 bug)
#[test]
fn BCutSimpleTest__ComplexPolygonPrismMinusBox_H2() {
    // H2 is identical to H1 - same polygon and same box
    // Create custom polygon from H2 test vertices (same as H1): v1(0,0,0) v2(1,0,0) v3(1,3,0)
    // v4(2,3,0) v5(2,0,0) v6(3,0,0) v7(3,5,0) v8(0,5,0)
    let aPoints = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 3.0, 0.0),
        Point3::new(2.0, 3.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(3.0, 5.0, 0.0),
        Point3::new(0.0, 5.0, 0.0),
    ];

    // Create wire and extrude to solid (prism sol p 0 0 2)
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);
    let aSolid = CreatePrism(&aFace, Vector3::new(0.0, 0.0, 2.0));

    // Create box: box b -1 2 1 5 1 3 (corner at (-1,2,1), dimensions 5x1x3)
    let aBox = CreateBox(Point3::new(-1.0, 2.0, 1.0), 5.0, 1.0, 3.0);

    // Perform boolean cut: bcut result sol b
    let aResult = PerformCut(&aSolid, &aBox);
    ValidateResult(&aResult, 68.0, -1.0, false);
}

// Test bcut_simple/H3: Complex cylinder/cone fuse minus translated cylinder (pro13307 bug)
#[test]
fn BCutSimpleTest__ComplexCylinderConeOperationPro13307_H3() {
    // This is similar to G9 but with a different bug reference
    // Create cylinder and cone, fuse them
    let aCylinder = CreateCylinder(9.0, 3.0);
    let aCone = CreateCone(7.0, 6.0, 4.0);

    let aFuseOp = BRepAlgoAPIFuse::from_shapes(&aCylinder, &aCone).unwrap();
    assert!(aFuseOp.is_done(), "Fuse operation failed");
    let aBody = aFuseOp.shape().clone();

    // Create small cylinder and translate it (x direction as per comment)
    let mut aSmallCylinder = CreateCylinder(1.0, 9.0);
    aSmallCylinder = TranslateShape(&aSmallCylinder, Vector3::new(5.0, 0.0, -2.0));

    let aResult = PerformCut(&aBody, &aSmallCylinder);
    ValidateResult(&aResult, 727.481, -1.0, false);
}

// Test bcut_simple/H4: Complex profile operation FORWARD FORWARD
#[test]
fn BCutSimpleTest__ComplexProfileForwardForward_H4() {
    // Create profile p1: "profile p1 o 0 0 40 x 150 y 200 x -150"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0]),
        ProfileOperation::new(ProfileCmd::X, &[-150.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: "profile p2 o 0 0 50 f 25 25 y 100 x 75 y -100"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 50.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, 25.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    // First cut operation: bcut po1 pr1 pr2
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: "profile p3 p 0 0 -1 1 0 0 o 0 0 50 f 50 -75 x 75 y -100 x -75"
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    // Final cut operation: bcut result po1 pr3
    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 98000.0, -1.0, false);
}

// Test bcut_simple/H5: Complex profile operation FORWARD FORWARD (variation)
#[test]
fn BCutSimpleTest__ComplexProfileForwardForwardVariation_H5() {
    // H5 has same profiles as H4, but p3 has different sequence: "f 50 -75 y -100 x 75 y 100"

    // Create profile p1: same as H4 "profile p1 o 0 0 40 x 150 y 200 x -150"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0]),
        ProfileOperation::new(ProfileCmd::X, &[-150.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: same as H4 "profile p2 o 0 0 50 f 25 25 y 100 x 75 y -100"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 50.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, 25.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: H5 variation "profile p3 p 0 0 -1 1 0 0 o 0 0 50 f 50 -75 y -100 x 75 y 100"
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 98000.0, -1.0, false);
}
