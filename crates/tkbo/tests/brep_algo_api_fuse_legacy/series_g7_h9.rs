//! BRepAlgoAPI_Fuse_Test cases LargePrismWithSmallPrismFromFront_G7 .. LargePrismWithSmallPrismFromTopCorner_H9.
#![allow(non_snake_case)]

use super::*;

// Test bfuse_simple/G7: Large prism base with small prism from front
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromFront_G7() {
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

    // Create small prism from front
    let aSmallPts = [
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(4.0, 2.0, 0.0),
        Point3::new(4.0, 2.0, 1.0),
        Point3::new(3.0, 2.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/G8: Large prism base with small prism from left
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromLeft_G8() {
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

    // Create small prism from left
    let aSmallPts = [
        Point3::new(3.0, 2.0, 0.0),
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(3.0, 3.0, 1.0),
        Point3::new(3.0, 2.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/G9: Large prism base with small prism from top
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromTop_G9() {
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

    // Create small prism from top
    let aSmallPts = [
        Point3::new(3.0, 2.0, 1.0),
        Point3::new(4.0, 2.0, 1.0),
        Point3::new(4.0, 3.0, 1.0),
        Point3::new(3.0, 3.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 0.0, -1.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H1: Large prism with small prism from back edge
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromBackEdge_H1() {
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

    // Create small prism from back
    let aSmallPts = [
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(4.0, 3.0, 0.0),
        Point3::new(4.0, 3.0, 1.0),
        Point3::new(3.0, 3.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H2: Large prism with small prism from right edge
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromRightEdge_H2() {
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

    // Create small prism from right
    let aSmallPts = [
        Point3::new(4.0, 2.0, 0.0),
        Point3::new(4.0, 3.0, 0.0),
        Point3::new(4.0, 3.0, 1.0),
        Point3::new(4.0, 2.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(-1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H3: Large prism with small prism from bottom corner
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromBottomCorner_H3() {
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
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(9.0, 3.0, 0.0),
        Point3::new(9.0, 4.0, 0.0),
        Point3::new(8.0, 4.0, 0.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 0.0, 1.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H4: Large prism with small prism from front edge
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromFrontEdge_H4() {
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

    // Create small prism from front
    let aSmallPts = [
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(9.0, 3.0, 0.0),
        Point3::new(9.0, 3.0, 1.0),
        Point3::new(8.0, 3.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H5: Large prism with small prism from left edge
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromLeftEdge_H5() {
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

    // Create small prism from left
    let aSmallPts = [
        Point3::new(8.0, 3.0, 0.0),
        Point3::new(8.0, 4.0, 0.0),
        Point3::new(8.0, 4.0, 1.0),
        Point3::new(8.0, 3.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H6: Large prism with small prism from top edge
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromTopEdge_H6() {
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

    // Create small prism from top
    let aSmallPts = [
        Point3::new(8.0, 3.0, 1.0),
        Point3::new(9.0, 3.0, 1.0),
        Point3::new(9.0, 4.0, 1.0),
        Point3::new(8.0, 4.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 0.0, -1.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H7: Large prism with small prism from back edge (corner)
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromBackEdgeCorner_H7() {
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

    // Create small prism from back
    let aSmallPts = [
        Point3::new(8.0, 4.0, 0.0),
        Point3::new(9.0, 4.0, 0.0),
        Point3::new(9.0, 4.0, 1.0),
        Point3::new(8.0, 4.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, -1.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H8: Large prism with small prism from right edge (corner)
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromRightEdgeCorner_H8() {
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

    // Create small prism from right
    let aSmallPts = [
        Point3::new(9.0, 3.0, 0.0),
        Point3::new(9.0, 4.0, 0.0),
        Point3::new(9.0, 4.0, 1.0),
        Point3::new(9.0, 3.0, 1.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(-1.0, 0.0, 0.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}

// Test bfuse_simple/H9: Large prism with small prism from top corner
#[test]
fn BFuseSimpleTest__LargePrismWithSmallPrismFromTopCorner_H9() {
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

    // Create small prism from top
    let aSmallPts = [
        Point3::new(3.0, 3.0, 4.0),
        Point3::new(4.0, 3.0, 4.0),
        Point3::new(4.0, 4.0, 4.0),
        Point3::new(3.0, 4.0, 4.0),
    ];
    let aSmallWire = CreatePolygonWire(&aSmallPts, true);
    let aSmallFace = CreateFaceFromWire(&aSmallWire);
    let aSmallPrism = CreatePrism(&aSmallFace, Vector3::new(0.0, 0.0, 1.0));

    let aResult = PerformFuse(&aBasePrism, &aSmallPrism);
    ValidateResult(&aResult, 152.0, -1.0, false);
}
