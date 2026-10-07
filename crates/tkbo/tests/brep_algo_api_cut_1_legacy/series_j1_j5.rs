//! BRepAlgoAPI_Cut_Test_1 cases ComplexProfileReversedReversedVariation3_J1 .. ComplexProfileForwardForwardForwardVariation_J5.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/J1: Complex profile operation REVERSED REVERSED (variation 3)
#[test]
fn BCutSimpleTest1__ComplexProfileReversedReversedVariation3_J1() {
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

    // Create profile p3: "profile p3 o 0 0 50 f 50 75 y 100 x 75 y -100" with prism height -3
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, 75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -3.0));

    let aResult = PerformCut(&aIntermediate, &aPrism3);
    ValidateResult(&aResult, 53000.0, -1.0, false);
}

// Test bcut_simple/J2: Complex profile operation with multiple cuts (FORWARD FORWARD FORWARD)
#[test]
fn BCutSimpleTest1__ComplexMultiCutProfileForwardForwardForward_J2() {
    // Create base prism
    let aPrism1 = CreateRectangularPrism(Point3::new(0.0, 0.0, 40.0), 175.0, 250.0, -40.0);

    // Create first tool prism
    let aPrism2 = CreateRectangularPrism(Point3::new(25.0, 25.0, 50.0), 50.0, 75.0, -30.0);

    // First cut operation
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate1 = aCutOp1.shape().clone();

    // Create second tool prism
    let aPrism3 = CreateRectangularPrism(Point3::new(100.0, 150.0, 50.0), 50.0, 75.0, -30.0);

    // Second cut operation
    let aCutOp2 = BRepAlgoAPICut::from_shapes(&aIntermediate1, &aPrism3).unwrap();
    assert!(aCutOp2.is_done(), "Second cut operation failed");
    let aIntermediate2 = aCutOp2.shape().clone();

    // Create third tool prism
    let aPrism4 = CreateRectangularPrism(Point3::new(50.0, -75.0, 50.0), 75.0, 100.0, -30.0);

    // Final cut operation
    let aResult = PerformCut(&aIntermediate2, &aPrism4);
    ValidateResult(&aResult, 134500.0, -1.0, false);
}

// Test bcut_simple/J3: Complex profile operation with translation (FORWARD REVERSED FORWARD)
#[test]
fn BCutSimpleTest1__ComplexProfileWithTranslationForwardReversedForward_J3() {
    // Create profile p1: "profile p1 o 0 0 40 x 175 y 250 x -175"
    let aPrism1 = CreateRectangularPrism(Point3::new(0.0, 0.0, 40.0), 175.0, 250.0, -40.0);

    // Create profile p2: "profile p2 o 0 0 50 f 25 25 y 75 x 50 y -75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 50.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, 25.0]),
        ProfileOperation::new(ProfileCmd::Y, &[75.0]),
        ProfileOperation::new(ProfileCmd::X, &[50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    // First cut operation: "bcut po1 pr1 pr2"
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate1 = aCutOp1.shape().clone();

    // Create profile p3: "profile p3 p 0 0 -1 1 0 0 o 0 0 50 f 100 -150 y -75 x 50 y 75"
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[100.0, -150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-75.0]),
        ProfileOperation::new(ProfileCmd::X, &[50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    // Second cut operation: "bcut po2 po1 pr3"
    let aCutOp2 = BRepAlgoAPICut::from_shapes(&aIntermediate1, &aPrism3).unwrap();
    assert!(aCutOp2.is_done(), "Second cut operation failed");
    let aIntermediate2 = aCutOp2.shape().clone();

    // Create profile p4: "profile p4 p 0 0 -1 1 0 0 o 0 0 50 f 50 -75 x 75 y -100 x -75"
    let aProfileOps4 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile4 = CreateProfile(&aPlane2, &aProfileOps4);
    let mut aPrism4 = CreatePrism(&aProfile4, Vector3::new(0.0, 0.0, -30.0));

    // Apply translation: "ttranslate pr4 -10 0 0"
    aPrism4 = TranslateShape(&aPrism4, Vector3::new(-10.0, 0.0, 0.0));

    // Final cut operation: "bcut result po2 pr4"
    let aResult = PerformCut(&aIntermediate2, &aPrism4);
    ValidateResult(&aResult, 134500.0, -1.0, false);
}

// Test bcut_simple/J4: Complex profile operation (REVERSED REVERSED FORWARD)
#[test]
fn BCutSimpleTest1__ComplexProfileReversedReversedForward_J4() {
    // Create profile p1: "profile p1 o 0 0 40 x 175 y 250 x -175"
    let aPrism1 = CreateRectangularPrism(Point3::new(0.0, 0.0, 40.0), 175.0, 250.0, -40.0);

    // Create profile p2: "profile p2 p 0 0 -1 1 0 0 o 0 0 50 f 25 -25 y -75 x 50 y 75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, -25.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-75.0]),
        ProfileOperation::new(ProfileCmd::X, &[50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    // Create profile p3: "profile p3 p 0 0 -1 1 0 0 o 0 0 50 f 100 -150 y -75 x 50 y 75"
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[100.0, -150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-75.0]),
        ProfileOperation::new(ProfileCmd::X, &[50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    // First cut operation: "bcut po1 pr1 pr2"
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate1 = aCutOp1.shape().clone();

    // Second cut operation: "bcut po2 po1 pr3"
    let aCutOp2 = BRepAlgoAPICut::from_shapes(&aIntermediate1, &aPrism3).unwrap();
    assert!(aCutOp2.is_done(), "Second cut operation failed");
    let aIntermediate2 = aCutOp2.shape().clone();

    // Create profile p4: "profile p4 p 0 0 -1 1 0 0 o 0 0 50 f 50 -75 x 75 y -100 x -75"
    let aProfileOps4 = [
        ProfileOperation::new(ProfileCmd::P, &[0.0, 0.0, -1.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, -75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile4 = CreateProfile(&aPlane2, &aProfileOps4);
    let aPrism4 = CreatePrism(&aProfile4, Vector3::new(0.0, 0.0, -30.0));

    // Final cut operation: "bcut result po2 pr4"
    let aResult = PerformCut(&aIntermediate2, &aPrism4);
    ValidateResult(&aResult, 134500.0, -1.0, false);
}

// Test bcut_simple/J5: Complex profile operation (FORWARD FORWARD FORWARD variation)
#[test]
fn BCutSimpleTest1__ComplexProfileForwardForwardForwardVariation_J5() {
    // Create profile p1: "profile p1 o 0 0 40 x 175 y 250 x -175"
    let aPrism1 = CreateRectangularPrism(Point3::new(0.0, 0.0, 40.0), 175.0, 250.0, -40.0);

    // Create profile p2: "profile p2 o 0 0 50 f 25 25 y 75 x 50 y -75"
    let aPlane2 = Plane3::new(
        Point3::new(0.0, 0.0, 50.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[25.0, 25.0]),
        ProfileOperation::new(ProfileCmd::Y, &[75.0]),
        ProfileOperation::new(ProfileCmd::X, &[50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-75.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, -30.0));

    // Create profile p3: "profile p3 o 0 0 50 f 100 150 y 75 x 50 y -75"
    let aProfileOps3 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[100.0, 150.0]),
        ProfileOperation::new(ProfileCmd::Y, &[75.0]),
        ProfileOperation::new(ProfileCmd::X, &[50.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-75.0]),
    ];
    let aProfile3 = CreateProfile(&aPlane2, &aProfileOps3);
    let aPrism3 = CreatePrism(&aProfile3, Vector3::new(0.0, 0.0, -30.0));

    // First cut operation: "bcut po1 pr1 pr2"
    let aCutOp1 = BRepAlgoAPICut::from_shapes(&aPrism1, &aPrism2).unwrap();
    assert!(aCutOp1.is_done(), "First cut operation failed");
    let aIntermediate1 = aCutOp1.shape().clone();

    // Second cut operation: "bcut po2 po1 pr3"
    let aCutOp2 = BRepAlgoAPICut::from_shapes(&aIntermediate1, &aPrism3).unwrap();
    assert!(aCutOp2.is_done(), "Second cut operation failed");
    let aIntermediate2 = aCutOp2.shape().clone();

    // Create profile p4: "profile p4 o 0 0 50 f 50 75 x 75 y 100 x -75"
    let aProfileOps4 = [
        ProfileOperation::new(ProfileCmd::O, &[0.0, 0.0, 50.0]),
        ProfileOperation::new(ProfileCmd::F, &[50.0, 75.0]),
        ProfileOperation::new(ProfileCmd::X, &[75.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-75.0]),
    ];
    let aProfile4 = CreateProfile(&aPlane2, &aProfileOps4);
    let aPrism4 = CreatePrism(&aProfile4, Vector3::new(0.0, 0.0, -30.0));

    // Final cut operation: "bcut result po2 pr4"
    let aResult = PerformCut(&aIntermediate2, &aPrism4);
    ValidateResult(&aResult, 134500.0, -1.0, false);
}
