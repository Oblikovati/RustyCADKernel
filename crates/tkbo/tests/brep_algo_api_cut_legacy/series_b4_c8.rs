//! BRepAlgoAPI_Cut_Test cases AdjacentBoxMinusNurbsBox_B4 .. NurbsBoxMinusOffsetCube_C8.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/B4: adjacent box - NURBS box (reverse B3)
#[test]
fn BCutSimpleTest__AdjacentBoxMinusNurbsBox_B4() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aAdjacentBox = CreateBox(Point3::new(1.0, 1.0, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformCut(&aAdjacentBox, &aNurbsBox);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/B5: NURBS box - smaller contained box
#[test]
fn BCutSimpleTest__NurbsBoxMinusSmallerBox_B5() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aSmallerBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 0.5, 1.0, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aSmallerBox);
    ValidateResult(&aResult, 5.5, -1.0, false);
}

// Test bcut_simple/B6: smaller box - NURBS box (should result in empty)
#[test]
fn BCutSimpleTest__SmallerBoxMinusNurbsBox_B6() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aSmallerBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 0.5, 1.0, 0.5);

    let aResult = PerformCut(&aSmallerBox, &aNurbsBox);
    ValidateResult(&aResult, -1.0, -1.0, true);
}

// Test bcut_simple/B7: NURBS box - partially overlapping box
#[test]
fn BCutSimpleTest__NurbsBoxMinusPartiallyOverlappingBox_B7() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aPartialBox = CreateBox(Point3::new(0.0, -0.5, 0.0), 0.5, 0.5, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aPartialBox);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/B8: partially overlapping box - NURBS box
#[test]
fn BCutSimpleTest__PartiallyOverlappingBoxMinusNurbsBox_B8() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aPartialBox = CreateBox(Point3::new(0.0, -0.5, 0.0), 0.5, 0.5, 1.0);

    let aResult = PerformCut(&aPartialBox, &aNurbsBox);
    ValidateResult(&aResult, 2.5, -1.0, false);
}

// Test bcut_simple/B9: NURBS box - extended overlapping box
#[test]
fn BCutSimpleTest__NurbsBoxMinusExtendedBox_B9() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aExtendedBox = CreateBox(Point3::new(0.0, -0.5, 0.0), 0.5, 1.5, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aExtendedBox);
    ValidateResult(&aResult, 4.0, -1.0, false);
}

// Test bcut_simple/C1: extended box - NURBS box (from B8 setup)
#[test]
fn BCutSimpleTest__ExtendedBoxMinusNurbsBox_C1() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aExtendedBox = CreateBox(Point3::new(0.0, -0.5, 0.0), 0.5, 1.5, 1.0);

    let aResult = PerformCut(&aExtendedBox, &aNurbsBox);
    ValidateResult(&aResult, 2.5, -1.0, false);
}

// Test bcut_simple/C2: NURBS box - shifted overlapping box
#[test]
fn BCutSimpleTest__NurbsBoxMinusShiftedBox_C2() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aShiftedBox = CreateBox(Point3::new(0.0, 0.5, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aShiftedBox);
    ValidateResult(&aResult, 4.0, -1.0, false);
}

// Test bcut_simple/C3: shifted box - NURBS box (reverse C2)
#[test]
fn BCutSimpleTest__ShiftedBoxMinusNurbsBox_C3() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aShiftedBox = CreateBox(Point3::new(0.0, 0.5, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformCut(&aShiftedBox, &aNurbsBox);
    ValidateResult(&aResult, 4.0, -1.0, false);
}

// Test bcut_simple/C4: NURBS box - narrow contained box
#[test]
fn BCutSimpleTest__NurbsBoxMinusNarrowBox_C4() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aNarrowBox = CreateBox(Point3::new(0.0, 0.25, 0.0), 1.0, 0.5, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aNarrowBox);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/C5: narrow box - NURBS box (should result in empty)
#[test]
fn BCutSimpleTest__NarrowBoxMinusNurbsBox_C5() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aNarrowBox = CreateBox(Point3::new(0.0, 0.25, 0.0), 1.0, 0.5, 1.0);

    let aResult = PerformCut(&aNarrowBox, &aNurbsBox);
    ValidateResult(&aResult, -1.0, -1.0, true);
}

// Test bcut_simple/C6: NURBS box - small corner cube
#[test]
fn BCutSimpleTest__NurbsBoxMinusCornerCube_C6() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aCornerCube = CreateBox(Point3::new(0.0, 0.0, 0.0), 0.5, 0.5, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aCornerCube);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/C7: small corner cube - NURBS box (should result in empty)
#[test]
fn BCutSimpleTest__CornerCubeMinusNurbsBox_C7() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aCornerCube = CreateBox(Point3::new(0.0, 0.0, 0.0), 0.5, 0.5, 0.5);

    let aResult = PerformCut(&aCornerCube, &aNurbsBox);
    ValidateResult(&aResult, -1.0, -1.0, true);
}

// Test bcut_simple/C8: NURBS box - offset small cube
#[test]
fn BCutSimpleTest__NurbsBoxMinusOffsetCube_C8() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetCube = CreateBox(Point3::new(0.0, -0.5, 0.0), 0.5, 0.5, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aOffsetCube);
    ValidateResult(&aResult, 6.0, -1.0, false);
}
