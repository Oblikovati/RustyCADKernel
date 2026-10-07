//! BRepAlgoAPI_Fuse_Test cases LargePrismWithSmallPrismFromFrontTopLevel_I1 .. LargePrismWithOblongPrismRightSideX_J3.
#![allow(non_snake_case)]

use super::*;

// Test bfuse_simple/I1: Large prism with small prism from front (top level)
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromFrontTopLevel_I1() {
    // Create large prism base from vertices: (3,3,0) to (8,9,4)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 4.0));

    // Create small prism from front at top level
    let aSmallPts = [
        Point3::new(3.0, 3.0, 4.0),
        Point3::new(4.0, 3.0, 4.0),
        Point3::new(4.0, 3.0, 5.0),
        Point3::new(3.0, 3.0, 5.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/I2: Large prism with small prism from left (top level)
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromLeftTopLevel_I2() {
    // Create large prism base from vertices: (3,3,0) to (8,9,4)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 4.0));

    // Create small prism from left at top level
    let aSmallPts = [
        Point3::new(3.0, 3.0, 4.0),
        Point3::new(3.0, 4.0, 4.0),
        Point3::new(3.0, 4.0, 5.0),
        Point3::new(3.0, 3.0, 5.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/I3: Large prism with small prism from top (above)
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromTopAbove_I3() {
    // Create large prism base from vertices: (3,3,0) to (8,9,4)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 4.0));

    // Create small prism from top (above the base)
    let aSmallPts = [
        Point3::new(3.0, 3.0, 5.0),
        Point3::new(4.0, 3.0, 5.0),
        Point3::new(4.0, 4.0, 5.0),
        Point3::new(3.0, 4.0, 5.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 0.0, -1.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/I4: Large prism with small prism from back (top level)
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromBackTopLevel_I4() {
    // Create large prism base from vertices: (3,3,0) to (8,9,4)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 4.0));

    // Create small prism from back at top level
    let aSmallPts = [
        Point3::new(3.0, 4.0, 4.0),
        Point3::new(4.0, 4.0, 4.0),
        Point3::new(4.0, 4.0, 5.0),
        Point3::new(3.0, 4.0, 5.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/I5: Large prism with small prism from right (top level)
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromRightTopLevel_I5() {
    // Create large prism base from vertices: (3,3,0) to (8,9,4)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 4.0));

    // Create small prism from right at top level
    let aSmallPts = [
        Point3::new(4.0, 3.0, 4.0),
        Point3::new(4.0, 4.0, 4.0),
        Point3::new(4.0, 4.0, 5.0),
        Point3::new(4.0, 3.0, 5.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(-1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/I6: Large prism with oblong prism (profile-based)
#[test]
fn BFuseSimpleTest__LargePrismWithOblongPrismProfile_I6() {
    // Create large prism base (5 units high)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 5.0));

    // Create oblong profile: O 4 3 0  P 0 1 0 1 0 0  D -1 0  C 1 180  X 2  C 1 180  W
    let aOps = [
        ProfileOperation::new(ProfileCmd::O, &[4.0, 3.0, 0.0]),
        ProfileOperation::new(ProfileCmd::P, &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::D, &[-1.0, 0.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::X, &[2.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::W, &[]),
    ];
    let aProfileFace = CreateProfileFromOperations(&aOps);
    let aOblongPrism = CreatePrism(&aProfileFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aOblongPrism);
    ValidateResult(&aResult, 180.283, -1.0, false);
}

// Test bfuse_simple/I7: Large prism with oblong prism (Y direction profile)
#[test]
fn BFuseSimpleTest__LargePrismWithOblongPrismYDirection_I7() {
    // Create large prism base (5 units high)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 5.0));

    // Create oblong profile: O 3 3 3  P 0 1 0 1 0 0  D 0 -1  C 1 180  Y 2  C 1 180  W
    let aOps = [
        ProfileOperation::new(ProfileCmd::O, &[3.0, 3.0, 3.0]),
        ProfileOperation::new(ProfileCmd::P, &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::D, &[0.0, -1.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::Y, &[2.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::W, &[]),
    ];
    let aProfileFace = CreateProfileFromOperations(&aOps);
    let aOblongPrism = CreatePrism(&aProfileFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aOblongPrism);
    ValidateResult(&aResult, 180.283, -1.0, false);
}

// Test bfuse_simple/I8: Large prism with oblong prism (X direction profile)
#[test]
fn BFuseSimpleTest__LargePrismWithOblongPrismXDirection_I8() {
    // Create large prism base (5 units high)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 5.0));

    // Create oblong profile: O 5 3 0  P 0 1 0 1 0 0  D -1 0  C 1 180  X 2  C 1 180  W
    let aOps = [
        ProfileOperation::new(ProfileCmd::O, &[5.0, 3.0, 0.0]),
        ProfileOperation::new(ProfileCmd::P, &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::D, &[-1.0, 0.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::X, &[2.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::W, &[]),
    ];
    let aProfileFace = CreateProfileFromOperations(&aOps);
    let aOblongPrism = CreatePrism(&aProfileFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aOblongPrism);
    ValidateResult(&aResult, 180.283, -1.0, false);
}

// Test bfuse_simple/I9: Large prism with oblong prism (Y direction profile)
#[test]
fn BFuseSimpleTest__LargePrismWithOblongPrismYDirectionFinal_I9() {
    // Create large prism base (5 units high)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 5.0));

    // Create oblong profile: O 6 3 3  P 0 1 0 1 0 0  D 0 -1  C 1 180  Y 2  C 1 180  W
    let aOps = [
        ProfileOperation::new(ProfileCmd::O, &[6.0, 3.0, 3.0]),
        ProfileOperation::new(ProfileCmd::P, &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0]),
        ProfileOperation::new(ProfileCmd::D, &[0.0, -1.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::Y, &[2.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::W, &[]),
    ];
    let aProfileFace = CreateProfileFromOperations(&aOps);
    let aOblongPrism = CreatePrism(&aProfileFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aOblongPrism);
    ValidateResult(&aResult, 180.283, -1.0, false);
}

// Test bfuse_simple/J1: Large prism with oblong prism (right side face profile)
#[test]
fn BFuseSimpleTest__LargePrismWithOblongPrismRightSide_J1() {
    // Create large prism base (5 units high)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 5.0));

    // Create oblong profile: O 8 4 0  P -1 0 0 0 1 0  D -1 0  C 1 180  X 2  C 1 180  W
    let aOps = [
        ProfileOperation::new(ProfileCmd::O, &[8.0, 4.0, 0.0]),
        ProfileOperation::new(ProfileCmd::P, &[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
        ProfileOperation::new(ProfileCmd::D, &[-1.0, 0.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::X, &[2.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::W, &[]),
    ];
    let aProfileFace = CreateProfileFromOperations(&aOps);
    let aOblongPrism = CreatePrism(&aProfileFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aOblongPrism);
    ValidateResult(&aResult, 180.283, -1.0, false);
}

// Test bfuse_simple/J2: Large prism with oblong prism (right side face profile Y direction)
#[test]
fn BFuseSimpleTest__LargePrismWithOblongPrismRightSideY_J2() {
    // Create large prism base (5 units high)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 5.0));

    // Create oblong profile: O 8 3 3  P -1 0 0 0 1 0  D 0 -1  C 1 180  Y 2  C 1 180  W
    let aOps = [
        ProfileOperation::new(ProfileCmd::O, &[8.0, 3.0, 3.0]),
        ProfileOperation::new(ProfileCmd::P, &[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
        ProfileOperation::new(ProfileCmd::D, &[0.0, -1.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::Y, &[2.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::W, &[]),
    ];
    let aProfileFace = CreateProfileFromOperations(&aOps);
    let aOblongPrism = CreatePrism(&aProfileFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aOblongPrism);
    ValidateResult(&aResult, 180.283, -1.0, false);
}

// Test bfuse_simple/J3: Large prism with oblong prism (right side face profile X direction)
#[test]
fn BFuseSimpleTest__LargePrismWithOblongPrismRightSideX_J3() {
    // Create large prism base (5 units high)
    let aBasePts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 9.0, 0.0),
        Point3::new(3.0, 9.0, 0.0),
    ];
    let aBaseWire = CreatePolygonWire(&aBasePts, true);
    let aBaseFace = CreateFaceFromWire(&aBaseWire);
    let aBasePrism = CreatePrism(&aBaseFace, Vector3::new(0.0, 0.0, 5.0));

    // Create oblong profile: O 8 6 0  P -1 0 0 0 1 0  D -1 0  C 1 180  X 2  C 1 180  W
    let aOps = [
        ProfileOperation::new(ProfileCmd::O, &[8.0, 6.0, 0.0]),
        ProfileOperation::new(ProfileCmd::P, &[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
        ProfileOperation::new(ProfileCmd::D, &[-1.0, 0.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::X, &[2.0]),
        ProfileOperation::new(ProfileCmd::C, &[1.0, 180.0]),
        ProfileOperation::new(ProfileCmd::W, &[]),
    ];
    let aProfileFace = CreateProfileFromOperations(&aOps);
    let aOblongPrism = CreatePrism(&aProfileFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aOblongPrism);
    ValidateResult(&aResult, 180.283, -1.0, false);
}
