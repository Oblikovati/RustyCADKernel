//! BRepAlgoAPI_Cut_Test_1 cases SameOrientedProfileForwardObjectReversedTool_K2 .. SameOrientedProfileForwardObjectReversedToolVar_K6.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/K2: SAMEORIENTED profile operation (forward object, reversed tool)
#[test]
fn BCutSimpleTest1__SameOrientedProfileForwardObjectReversedTool_K2() {
    // Create profile p1: "profile p1 o 0 0 40 x 150 y 200 x -150"
    let aPrism1 = CreateRectangularPrism(Point3::new(0.0, 0.0, 40.0), 150.0, 200.0, -40.0);

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

    // First cut operation: "bcut po1 pr1 pr2"
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: "profile p3 p 0 0 -1 1 0 0 o 0 0 -10 f 50 -75 x 75 y -100 x -75"
    let aPlane3 = Plane3::new(
        Point3::new(0.0, 0.0, -1.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, -10.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane3, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, 30.0));

    // Final cut operation: "bcut result po1 pr3"
    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 97000.0, -1.0, false);
}

// Test bcut_simple/K3: SAMEORIENTED profile operation (reversed object + forward tool)
#[test]
fn BCutSimpleTest1__SameOrientedProfileReversedObjectForwardTool_K3() {
    // Create profile p1: "profile p1 p 0 0 -1 1 0 0 o 0 0 40 y -200 x 150 y 200"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-200.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: "profile p2 p 0 0 -1 1 0 0 o 0 0 50 f 25 -25 x 75 y -100 x -75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, -25.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    // First cut operation: "bcut po1 pr1 pr2"
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: "profile p3 o 0 0 -10 f 50 75 y 100 x 75 y -100"
    let aPlane3 = Plane3::new(
        Point3::new(0.0, 0.0, -10.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, -10.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, 75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane3, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, 30.0));

    // Final cut operation: "bcut result po1 pr3"
    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 97000.0, -1.0, false);
}

// Test bcut_simple/K4: SAMEORIENTED profile operation (reversed object + reversed tool)
#[test]
fn BCutSimpleTest1__SameOrientedProfileReversedObjectReversedTool_K4() {
    // Create profile p1: "profile p1 p 0 0 -1 1 0 0 o 0 0 40 y -200 x 150 y 200"
    let aPlane1 = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 40.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-200.0]),
        ProfileOperation::new(ProfileCmd::X, &[150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[200.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, -40.0));

    // Create profile p2: "profile p2 p 0 0 -1 1 0 0 o 0 0 50 f 25 -25 x 75 y -100 x -75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, -25.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    // First cut operation: "bcut po1 pr1 pr2"
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: "profile p3 o 0 0 -10 f 50 75 x 75 y 100 x -75"
    let aPlane3 = Plane3::new(
        Point3::new(0.0, 0.0, -10.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, -10.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, 75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane3, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, 30.0));

    // Final cut operation: "bcut result po1 pr3"
    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 97000.0, -1.0, false);
}

// Test bcut_simple/K5: SAMEORIENTED profile operation (forward + reversed faces, repeated)
#[test]
fn BCutSimpleTest1__SameOrientedProfileForwardReversedRepeated_K5() {
    // Create profile p1: "profile p1 o 0 0 40 x 150 y 200 x -150"
    let aPrism1 = CreateRectangularPrism(Point3::new(0.0, 0.0, 40.0), 150.0, 200.0, -40.0);

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

    // First cut operation: "bcut po1 pr1 pr2"
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: "profile p3 p 0 0 -1 1 0 0 o 0 0 -10 f 50 -75 y -100 x 75 y 100"
    let aPlane3 = Plane3::new(
        Point3::new(0.0, 0.0, -1.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, -10.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane3, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, 30.0));

    // Final cut operation: "bcut result po1 pr3"
    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 97000.0, -1.0, false);
}

// Test bcut_simple/K6: SAMEORIENTED profile operation (forward object + reversed tool, variation)
#[test]
fn BCutSimpleTest1__SameOrientedProfileForwardObjectReversedToolVar_K6() {
    // Create profile p1: "profile p1 o 0 0 40 x 150 y 200 x -150"
    let aPrism1 = CreateRectangularPrism(Point3::new(0.0, 0.0, 40.0), 150.0, 200.0, -40.0);

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

    // First cut operation: "bcut po1 pr1 pr2"
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate = aCutOp1.shape().clone();

    // Create profile p3: "profile p3 p 0 0 -1 1 0 0 o 0 0 -10 f 50 -75 x 75 y -100 x -75"
    let aPlane3 = Plane3::new(
        Point3::new(0.0, 0.0, -1.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, -10.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane3, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, 30.0));

    // Final cut operation: "bcut result po1 pr3"
    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 97000.0, -1.0, false);
}
