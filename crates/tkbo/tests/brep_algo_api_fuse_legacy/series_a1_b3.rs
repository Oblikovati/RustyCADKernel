//! BRepAlgoAPI_Fuse_Test cases SpherePlusBox_A1 .. NurbsBoxPlusAdjacentBox_B3.
#![allow(non_snake_case)]

use super::*;

// Test bfuse_simple/A1: psphere s 1; box b 1 1 1; bfuse result s b; checkprops result -s 14.6394
#[test]
fn BFuseSimpleTest__SpherePlusBox_A1() {
    let aSphere = CreateUnitSphere();
    let aBox = CreateUnitBox();

    let aResult = PerformFuse(&aSphere, &aBox);
    ValidateResult(&aResult, 14.6394, -1.0, false);
}

// Test bfuse_simple/A2: rotated sphere + box
#[test]
fn BFuseSimpleTest__RotatedSpherePlusBox_A2() {
    let aSphere = CreateUnitSphere();

    // Apply standard rotation: Z(-90deg) then Y(-45deg)
    let aRotatedSphere = RotateStandard(&aSphere);

    let aBox = CreateUnitBox();
    let aResult = PerformFuse(&aRotatedSphere, &aBox);
    ValidateResult(&aResult, 14.6393, -1.0, false);
}

// Test bfuse_simple/A3: box + rotated sphere
#[test]
fn BFuseSimpleTest__BoxPlusRotatedSphere_A3() {
    let aSphere = CreateUnitSphere();

    // Apply standard rotation: Z(-90deg) then Y(-45deg)
    let aRotatedSphere = RotateStandard(&aSphere);

    let aBox = CreateUnitBox();
    let aResult = PerformFuse(&aBox, &aRotatedSphere);
    ValidateResult(&aResult, 14.6393, -1.0, false);
}

// Test bfuse_simple/A4: sphere + rotated box
#[test]
fn BFuseSimpleTest__SpherePlusRotatedBox_A4() {
    let aSphere = CreateUnitSphere();
    let aBox = CreateUnitBox();

    // Apply Y rotation: 90deg
    let aRotatedBox = RotateY(&aBox, 90.0);

    let aResult = PerformFuse(&aSphere, &aRotatedBox);
    ValidateResult(&aResult, 14.6393, -1.0, false);
}

// Test bfuse_simple/A5: rotated box + sphere
#[test]
fn BFuseSimpleTest__RotatedBoxPlusSphere_A5() {
    let aSphere = CreateUnitSphere();
    let aBox = CreateUnitBox();

    // Apply Y rotation: 90deg
    let aRotatedBox = RotateY(&aBox, 90.0);

    let aResult = PerformFuse(&aRotatedBox, &aSphere);
    ValidateResult(&aResult, 14.6393, -1.0, false);
}

// Test bfuse_simple/A6: nurbs box + identical box (should be 6.0 surface area)
#[test]
fn BFuseSimpleTest__IdenticalNurbsBoxPlusBox_A6() {
    // Create first box and convert to NURBS
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bfuse_simple/A7: box + nurbs box (identical)
#[test]
fn BFuseSimpleTest__IdenticalBoxPlusNurbsBox_A7() {
    let aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);

    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox2 = ConvertToNurbs(&aBox2);
    assert!(!aBox2.is_null(), "Failed to convert to NURBS");

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bfuse_simple/A8: nurbs box + larger box
#[test]
fn BFuseSimpleTest__NurbsBoxPlusLargerBox_A8() {
    // Create first box and convert to NURBS
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(-0.5, -0.5, -0.5), 2.0, 2.0, 2.0);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 24.0, -1.0, false);
}

// Test bfuse_simple/A9: larger box + nurbs box
#[test]
fn BFuseSimpleTest__LargerBoxPlusNurbsBox_A9() {
    let aBox1 = CreateBox(Point3::new(-0.5, -0.5, -0.5), 2.0, 2.0, 2.0);

    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox2 = ConvertToNurbs(&aBox2);
    assert!(!aBox2.is_null(), "Failed to convert to NURBS");

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 24.0, -1.0, false);
}

// Test bfuse_simple/B1: nurbs box + box
#[test]
fn BFuseSimpleTest__NurbsBoxPlusBox_B1() {
    // Create first box and convert to NURBS
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    // Create second box
    let aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 0.5, 1.0, 0.5);

    let aResult = PerformFuse(&aBox2, &aBox1);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bfuse_simple/B2: box + nurbs box
#[test]
fn BFuseSimpleTest__BoxPlusNurbsBox_B2() {
    let aBox1 = CreateBox(Point3::new(0.0, -0.5, 0.0), 0.5, 0.5, 1.0);

    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox2 = ConvertToNurbs(&aBox2);
    assert!(!aBox2.is_null(), "Failed to convert to NURBS");

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 7.5, -1.0, false);
}

// Test bfuse_simple/B3: box + nurbs box (adjacent)
#[test]
fn BFuseSimpleTest__NurbsBoxPlusAdjacentBox_B3() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(0.0, -0.5, 0.0), 0.5, 1.5, 1.0);

    let aResult = PerformFuse(&aBox2, &aBox1);
    ValidateResult(&aResult, 7.5, -1.0, false);
}
