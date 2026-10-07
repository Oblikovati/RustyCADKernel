//! BRepAlgoAPI_Cut_Test cases ComplexProfileForwardReversed_H6 .. ComplexProfileForwardForwardVariation2_I3.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/H6: Complex profile operation FORWARD REVERSED
#[test]
fn BCutSimpleTest__ComplexProfileForwardReversed_H6() {
    // Create profile p1: same as H4/H5 "profile p1 o 0 0 40 x 150 y 200 x -150"
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

    // Create profile p2: same as H4/H5 "profile p2 o 0 0 50 f 25 25 y 100 x 75 y -100"
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

    // Create profile p3: H6 variation "profile p3 o 0 0 50 f 50 75 x 75 y 100 x -75"
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, 75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 98000.0, -1.0, false);
}

// Test bcut_simple/H7: Complex profile operation FORWARD REVERSED (variation)
#[test]
fn BCutSimpleTest__ComplexProfileForwardReversedVariation_H7() {
    // Create profile p1: same as other H tests "profile p1 o 0 0 40 x 150 y 200 x -150"
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

    // Create profile p2: same as other H tests "profile p2 o 0 0 50 f 25 25 y 100 x 75 y -100"
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

    // Create profile p3: H7 variation "profile p3 o 0 0 50 f 50 75 y 100 x 75 y -100"
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, 75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 98000.0, -1.0, false);
}

// Test bcut_simple/H8: Complex profile operation REVERSED FORWARD
#[test]
fn BCutSimpleTest__ComplexProfileReversedForward_H8() {
    // Create profile p1: "profile p1 p 0 0 -1 1 0 0 o 0 0 40 y -200 x 150 y 200"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-200.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: "profile p2 p 0 0 -1 1 0 0 o 0 0 50 f 25 -25 x 75 y -100 x -75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 50.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, -25.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

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

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 98000.0, -1.0, false);
}

// Test bcut_simple/H9: Complex profile operation REVERSED FORWARD (variation)
#[test]
fn BCutSimpleTest__ComplexProfileReversedForwardVariation_H9() {
    // Create profile p1: same as H8 "profile p1 p 0 0 -1 1 0 0 o 0 0 40 y -200 x 150 y 200"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-200.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: same as H8 "profile p2 p 0 0 -1 1 0 0 o 0 0 50 f 25 -25 x 75 y -100 x -75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 50.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, -25.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: H9 variation "profile p3 p 0 0 -1 1 0 0 o 0 0 50 f 50 -75 y -100 x 75 y 100"
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

// Test bcut_simple/I1: Complex profile operation REVERSED REVERSED
#[test]
fn BCutSimpleTest__ComplexProfileReversedReversed_I1() {
    // Create profile p1: same as H8/H9 "profile p1 p 0 0 -1 1 0 0 o 0 0 40 y -200 x 150 y 200"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-200.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: same as H8/H9 "profile p2 p 0 0 -1 1 0 0 o 0 0 50 f 25 -25 x 75 y -100 x
    // -75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 50.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, -25.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: I1 "profile p3 o 0 0 50 f 50 75 x 75 y 100 x -75" (similar to H6)
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, 75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 98000.0, -1.0, false);
}

// Test bcut_simple/I2: Complex profile operation REVERSED REVERSED (variation)
#[test]
fn BCutSimpleTest__ComplexProfileReversedReversedVariation_I2() {
    // Create profile p1: same as I1 "profile p1 p 0 0 -1 1 0 0 o 0 0 40 y -200 x 150 y 200"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-200.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: same as I1 "profile p2 p 0 0 -1 1 0 0 o 0 0 50 f 25 -25 x 75 y -100 x -75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 50.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, -25.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: I2 "profile p3 o 0 0 50 f 50 75 y 100 x 75 y -100" (similar to H7)
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, 75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 98000.0, -1.0, false);
}

// Test bcut_simple/I3: Complex profile operation FORWARD FORWARD (variation)
#[test]
fn BCutSimpleTest__ComplexProfileForwardForwardVariation2_I3() {
    // Create profile p1: "profile p1 o 0 0 40 f 0 50 x 150 y 100 x -150"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::F, &[0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-150.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: same as I1/I2 but different plane "profile p2 o 0 0 50 f 25 25 y 100 x 75 y
    // -100"
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

    // Create profile p3: same as H8 "profile p3 p 0 0 -1 1 0 0 o 0 0 50 f 50 -75 x 75 y -100 x -75"
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 52000.0, -1.0, false);
}
