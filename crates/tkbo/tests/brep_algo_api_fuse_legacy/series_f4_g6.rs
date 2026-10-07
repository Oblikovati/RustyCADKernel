//! BRepAlgoAPI_Fuse_Test cases BoxWithPrismFromFrontExternal_F4 .. LargePrismWithSmallPrismFromBottom_G6.
#![allow(non_snake_case)]

use super::*;

// Test bfuse_simple/F4: Box with prism from front (external position)
#[test]
fn BFuseSimpleTest__BoxWithPrismFromFrontExternal_F4() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for front prism (external position)
    let aPoints = [
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(9.0, 3.0, 0.0),
        Point3::new(9.0, 3.0, 1.0),
        Point3::new(8.0, 3.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 1 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 1.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/F5: Box with prism from left (external position)
#[test]
fn BFuseSimpleTest__BoxWithPrismFromLeftExternal_F5() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for left prism (external position)
    let aPoints = [
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 4.0, 0.0),
        Point3::new(8.0, 4.0, 1.0),
        Point3::new(8.0, 3.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 1 0 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/F6: Box with prism from top (external position)
#[test]
fn BFuseSimpleTest__BoxWithPrismFromTopExternal_F6() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for top prism (external position)
    let aPoints = [
        Point3::new(8.0, 3.0, 1.0),
        Point3::new(9.0, 3.0, 1.0),
        Point3::new(9.0, 4.0, 1.0),
        Point3::new(8.0, 4.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 0 -1"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 0.0, -1.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/F7: Box with prism from back (external position)
#[test]
fn BFuseSimpleTest__BoxWithPrismFromBackExternal_F7() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for back prism (external position)
    let aPoints = [
        Point3::new(8.0, 4.0, 0.0),
        Point3::new(9.0, 4.0, 0.0),
        Point3::new(9.0, 4.0, 1.0),
        Point3::new(8.0, 4.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 -1 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/F8: Box with prism from right (external position)
#[test]
fn BFuseSimpleTest__BoxWithPrismFromRightExternal_F8() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for right prism (external position)
    let aPoints = [
        Point3::new(9.0, 3.0, 0.0),
        Point3::new(9.0, 4.0, 0.0),
        Point3::new(9.0, 4.0, 1.0),
        Point3::new(9.0, 3.0, 1.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f -1 0 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(-1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/F9: Box with prism from bottom (top position)
#[test]
fn BFuseSimpleTest__BoxWithPrismFromBottomTopPosition_F9() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for bottom prism at top position
    let aPoints = [
        Point3::new(3.0, 3.0, 4.0),
        Point3::new(4.0, 3.0, 4.0),
        Point3::new(4.0, 4.0, 4.0),
        Point3::new(3.0, 4.0, 4.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 0 1"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 0.0, 1.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/G1: Box with prism from front at top level
#[test]
fn BFuseSimpleTest__BoxWithPrismFromFrontTopLevel_G1() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for front prism at top level
    let aPoints = [
        Point3::new(3.0, 3.0, 4.0),
        Point3::new(4.0, 3.0, 4.0),
        Point3::new(4.0, 3.0, 5.0),
        Point3::new(3.0, 3.0, 5.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 1 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 1.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/G2: Box with prism from left at top level
#[test]
fn BFuseSimpleTest__BoxWithPrismFromLeftTopLevel_G2() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for left prism at top level
    let aPoints = [
        Point3::new(3.0, 3.0, 4.0),
        Point3::new(3.0, 4.0, 4.0),
        Point3::new(3.0, 4.0, 5.0),
        Point3::new(3.0, 3.0, 5.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 1 0 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/G3: Box with prism from top at top level
#[test]
fn BFuseSimpleTest__BoxWithPrismFromTopTopLevel_G3() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for top prism at top level
    let aPoints = [
        Point3::new(3.0, 3.0, 5.0),
        Point3::new(4.0, 3.0, 5.0),
        Point3::new(4.0, 4.0, 5.0),
        Point3::new(3.0, 4.0, 5.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 0 -1"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, 0.0, -1.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/G4: Box with prism from back at top level
#[test]
fn BFuseSimpleTest__BoxWithPrismFromBackTopLevel_G4() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for back prism at top level
    let aPoints = [
        Point3::new(3.0, 4.0, 4.0),
        Point3::new(4.0, 4.0, 4.0),
        Point3::new(4.0, 4.0, 5.0),
        Point3::new(3.0, 4.0, 5.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f 0 -1 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/G5: Box with prism from right at top level
#[test]
fn BFuseSimpleTest__BoxWithPrismFromRightTopLevel_G5() {
    // Create box: "box ba 3 3 0 5 7 4"
    let aBox = CreateBox(Point3::new(3.0, 3.0, 0.0), 5.0, 7.0, 4.0);

    // Create profile points for right prism at top level
    let aPoints = [
        Point3::new(4.0, 3.0, 4.0),
        Point3::new(4.0, 4.0, 4.0),
        Point3::new(4.0, 4.0, 5.0),
        Point3::new(4.0, 3.0, 5.0),
    ];

    // Create wire and face
    let aWire = CreatePolygonWire(&aPoints, true);
    let aFace = CreateFaceFromWire(&aWire);

    // Create prism: "prism bb f -1 0 0"
    let aPrism = CreatePrism(&aFace, Vector3::new(-1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBox, &aPrism);
    ValidateResult(&aResult, 170.0, -1.0, false);
}

// Test bfuse_simple/G6: Large prism base with small prism from bottom
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromBottom_G6() {
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

    // Create small prism from bottom
    let aSmallPts = [
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(4.0, 2.0, 0.0),
        Point3::new(4.0, 3.0, 0.0),
        Point3::new(3.0, 3.0, 0.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 0.0, 1.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}
