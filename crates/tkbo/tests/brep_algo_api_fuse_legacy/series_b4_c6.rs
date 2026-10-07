//! BRepAlgoAPI_Fuse_Test cases AdjacentBoxPlusNurbsBox_B4 .. NurbsBoxPlusCornerCube_C6.
#![allow(non_snake_case)]

use super::*;

// Test bfuse_simple/B4: adjacent box + nurbs box
#[test]
fn BFuseSimpleTest__AdjacentBoxPlusNurbsBox_B4() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.0, 0.5, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformFuse(&aBox2, &aBox1);
    ValidateResult(&aResult, 8.0, -1.0, false);
}

// Test bfuse_simple/B5: nurbs box + smaller box
#[test]
fn BFuseSimpleTest__NurbsBoxPlusSmallerBox_B5() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.25, 0.25, 0.25), 0.5, 0.5, 0.5);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bfuse_simple/B6: smaller box + nurbs box
#[test]
fn BFuseSimpleTest__SmallerBoxPlusNurbsBox_B6() {
    let aBox1 = CreateBox(Point3::new(0.25, 0.25, 0.25), 0.5, 0.5, 0.5);

    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox2 = ConvertToNurbs(&aBox2);
    assert!(!aBox2.is_null(), "Failed to convert to NURBS");

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bfuse_simple/B7: nurbs box + partially overlapping box
#[test]
fn BFuseSimpleTest__NurbsBoxPlusPartiallyOverlappingBox_B7() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.5, 0.5, 0.5), 1.0, 1.0, 1.0);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 7.75, -1.0, false);
}

// Test bfuse_simple/B8: partially overlapping box + nurbs box
#[test]
fn BFuseSimpleTest__PartiallyOverlappingBoxPlusNurbsBox_B8() {
    let aBox1 = CreateBox(Point3::new(0.5, 0.5, 0.5), 1.0, 1.0, 1.0);

    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox2 = ConvertToNurbs(&aBox2);
    assert!(!aBox2.is_null(), "Failed to convert to NURBS");

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 7.75, -1.0, false);
}

// Test bfuse_simple/B9: nurbs box + extended box
#[test]
fn BFuseSimpleTest__NurbsBoxPlusExtendedBox_B9() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(-0.5, -0.5, -0.5), 2.0, 2.0, 2.0);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 24.0, -1.0, false);
}

// Test bfuse_simple/C1: extended box + nurbs box
#[test]
fn BFuseSimpleTest__ExtendedBoxPlusNurbsBox_C1() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.5, 0.5, 0.5);

    let aResult = PerformFuse(&aBox2, &aBox1);
    ValidateResult(&aResult, 7.0, -1.0, false);
}

// Test bfuse_simple/C2: nurbs box + shifted extended box
#[test]
fn BFuseSimpleTest__NurbsBoxPlusShiftedBox_C2() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.0, -0.5, 0.0), 1.5, 0.5, 0.5);

    let aResult = PerformFuse(&aBox2, &aBox1);
    ValidateResult(&aResult, 8.5, -1.0, false);
}

// Test bfuse_simple/C3: shifted box + nurbs box
#[test]
fn BFuseSimpleTest__ShiftedBoxPlusNurbsBox_C3() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.25, 0.0, 0.0), 0.5, 0.5, 1.0);

    let aResult = PerformFuse(&aBox2, &aBox1);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bfuse_simple/C4: nurbs box + narrow box
#[test]
fn BFuseSimpleTest__NurbsBoxPlusNarrowBox_C4() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.25, 0.25, 0.0), 0.5, 0.5, 1.5);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.75, -1.0, false);
}

// Test bfuse_simple/C5: narrow box + nurbs box
#[test]
fn BFuseSimpleTest__NarrowBoxPlusNurbsBox_C5() {
    let aBox1 = CreateBox(Point3::new(0.25, 0.25, 0.0), 0.5, 0.5, 1.5);

    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox2 = ConvertToNurbs(&aBox2);
    assert!(!aBox2.is_null(), "Failed to convert to NURBS");

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.75, -1.0, false);
}

// Test bfuse_simple/C6: nurbs box + corner cube
#[test]
fn BFuseSimpleTest__NurbsBoxPlusCornerCube_C6() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.5, 0.5, 0.5), 0.5, 0.5, 0.5);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.0, -1.0, false);
}
