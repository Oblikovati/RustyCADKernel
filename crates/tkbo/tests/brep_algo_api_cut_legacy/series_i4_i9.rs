//! BRepAlgoAPI_Cut_Test cases ComplexProfileForwardForwardVariation3_I4 .. ComplexProfileReversedReversedVariation2_I9.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/I4: Complex profile operation FORWARD FORWARD (variation 2)
#[test]
fn BCutSimpleTest__ComplexProfileForwardForwardVariation3_I4() {
    // Create profile p1: same as I3 "profile p1 o 0 0 40 f 0 50 x 150 y 100 x -150"
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

    // Create profile p2: same as I3 "profile p2 o 0 0 50 f 25 25 y 100 x 75 y -100"
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

    // Create profile p3: I4 "profile p3 p 0 0 -1 1 0 0 o 0 0 50 f 50 -75 y -100 x 75 y 100" with
    // prism height -3
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -3.0));

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 53000.0, -1.0, false);
}

// Test bcut_simple/I5: Complex profile operation FORWARD REVERSED (variation 2)
#[test]
fn BCutSimpleTest__ComplexProfileForwardReversedVariation2_I5() {
    // Create profile p1: same as I3/I4 "profile p1 o 0 0 40 f 0 50 x 150 y 100 x -150"
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

    // Create profile p2: same as I3/I4 "profile p2 o 0 0 50 f 25 25 y 100 x 75 y -100"
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

    // Create profile p3: I5 "profile p3 o 0 0 50 f 50 75 x 75 y 100 x -75" (similar to H6/I1)
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
    ValidateResult(&aResult, 52000.0, -1.0, false);
}

// Test bcut_simple/I6: Complex profile operation FORWARD REVERSED (variation 3)
#[test]
fn BCutSimpleTest__ComplexProfileForwardReversedVariation3_I6() {
    // Same profiles as I5 but different p3 - "profile p3 o 0 0 50 f 50 75 y 100 x 75 y -100"
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
    ValidateResult(&aResult, 52000.0, -1.0, false);
}

// Test bcut_simple/I7: Complex profile operation REVERSED FORWARD (variation 2)
#[test]
fn BCutSimpleTest__ComplexProfileReversedForwardVariation2_I7() {
    // Create profile p1: "profile p1 p 0 0 -1 1 0 0 o 0 0 40 f 0 -50 y -100 x 150 y 100"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::F, &[0.0, -50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
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

// Test bcut_simple/I8: Complex profile operation REVERSED FORWARD (variation 3)
#[test]
fn BCutSimpleTest__ComplexProfileReversedForwardVariation3_I8() {
    // Create profile p1: "profile p1 p 0 0 -1 1 0 0 o 0 0 40 f 0 -50 y -100 x 150 y 100"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::F, &[0.0, -50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
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

    // Create profile p3: "profile p3 p 0 0 -1 1 0 0 o 0 0 50 f 50 -75 y -100 x 75 y 100"
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
    ValidateResult(&aResult, 52000.0, -1.0, false);
}

// Test bcut_simple/I9: Complex profile operation REVERSED REVERSED (variation 2)
#[test]
fn BCutSimpleTest__ComplexProfileReversedReversedVariation2_I9() {
    // Create profile p1: "profile p1 p 0 0 -1 1 0 0 o 0 0 40 f 0 -50 y -100 x 150 y 100"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 40.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::F, &[0.0, -50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
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

    // Create profile p3: "profile p3 o 0 0 50 f 50 75 x 75 y 100 x -75"
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
    ValidateResult(&aResult, 52000.0, -1.0, false);
}
