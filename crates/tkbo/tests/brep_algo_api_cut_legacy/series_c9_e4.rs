//! BRepAlgoAPI_Cut_Test cases OffsetCubeMinusNurbsBox_C9 .. NurbsBoxMinusExtendedYShiftedNarrowBox_E4.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/C9: offset small cube - NURBS box
#[test]
fn BCutSimpleTest__OffsetCubeMinusNurbsBox_C9() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetCube = CreateBox(Point3::new(0.0, -0.5, 0.0), 0.5, 0.5, 0.5);

    let aResult = PerformCut(&aOffsetCube, &aNurbsBox);
    ValidateResult(&aResult, 1.5, -1.0, false);
}

// Test bcut_simple/D1: NURBS box - offset corner cube
#[test]
fn BCutSimpleTest__NurbsBoxMinusOffsetCornerCube_D1() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetCornerCube = CreateBox(Point3::new(0.0, -0.5, -0.5), 0.5, 0.5, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aOffsetCornerCube);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/D2: offset corner cube - NURBS box
#[test]
fn BCutSimpleTest__OffsetCornerCubeMinusNurbsBox_D2() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetCornerCube = CreateBox(Point3::new(0.0, -0.5, -0.5), 0.5, 0.5, 0.5);

    let aResult = PerformCut(&aOffsetCornerCube, &aNurbsBox);
    ValidateResult(&aResult, 1.5, -1.0, false);
}

// Test bcut_simple/D3: NURBS box - shifted corner cube
#[test]
fn BCutSimpleTest__NurbsBoxMinusShiftedCornerCube_D3() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aShiftedCornerCube = CreateBox(Point3::new(-0.5, -0.5, -0.5), 0.5, 0.5, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aShiftedCornerCube);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/D4: shifted corner cube - NURBS box
#[test]
fn BCutSimpleTest__ShiftedCornerCubeMinusNurbsBox_D4() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aShiftedCornerCube = CreateBox(Point3::new(-0.5, -0.5, -0.5), 0.5, 0.5, 0.5);

    let aResult = PerformCut(&aShiftedCornerCube, &aNurbsBox);
    ValidateResult(&aResult, 1.5, -1.0, false);
}

// Test bcut_simple/D5: NURBS box - extended X box
#[test]
fn BCutSimpleTest__NurbsBoxMinusExtendedXBox_D5() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aExtendedXBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.5, 0.5, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aExtendedXBox);
    ValidateResult(&aResult, 5.5, -1.0, false);
}

// Test bcut_simple/D6: extended X box - NURBS box
#[test]
fn BCutSimpleTest__ExtendedXBoxMinusNurbsBox_D6() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aExtendedXBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.5, 0.5, 0.5);

    let aResult = PerformCut(&aExtendedXBox, &aNurbsBox);
    ValidateResult(&aResult, 1.5, -1.0, false);
}

// Test bcut_simple/D7: NURBS box - offset extended X box
#[test]
fn BCutSimpleTest__NurbsBoxMinusOffsetExtendedXBox_D7() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetExtendedXBox = CreateBox(Point3::new(0.0, -0.5, 0.0), 1.5, 0.5, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aOffsetExtendedXBox);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/D8: offset extended X box - NURBS box
#[test]
fn BCutSimpleTest__OffsetExtendedXBoxMinusNurbsBox_D8() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetExtendedXBox = CreateBox(Point3::new(0.0, -0.5, 0.0), 1.5, 0.5, 0.5);

    let aResult = PerformCut(&aOffsetExtendedXBox, &aNurbsBox);
    ValidateResult(&aResult, 3.5, -1.0, false);
}

// Test bcut_simple/D9: NURBS box - shifted narrow box
#[test]
fn BCutSimpleTest__NurbsBoxMinusShiftedNarrowBox_D9() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aShiftedNarrowBox = CreateBox(Point3::new(0.25, 0.0, 0.0), 0.5, 0.5, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aShiftedNarrowBox);
    ValidateResult(&aResult, 6.5, -1.0, false);
}

// Test bcut_simple/E1: shifted narrow box - NURBS box (should result in empty)
#[test]
fn BCutSimpleTest__ShiftedNarrowBoxMinusNurbsBox_E1() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aShiftedNarrowBox = CreateBox(Point3::new(0.25, 0.0, 0.0), 0.5, 0.5, 1.0);

    let aResult = PerformCut(&aShiftedNarrowBox, &aNurbsBox);
    ValidateResult(&aResult, -1.0, -1.0, true);
}

// Test bcut_simple/E2: NURBS box - offset shifted narrow box
#[test]
fn BCutSimpleTest__NurbsBoxMinusOffsetShiftedNarrowBox_E2() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetShiftedNarrowBox = CreateBox(Point3::new(0.25, -0.5, 0.0), 0.5, 0.5, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aOffsetShiftedNarrowBox);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/E3: offset shifted narrow box - NURBS box
#[test]
fn BCutSimpleTest__OffsetShiftedNarrowBoxMinusNurbsBox_E3() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetShiftedNarrowBox = CreateBox(Point3::new(0.25, -0.5, 0.0), 0.5, 0.5, 1.0);

    let aResult = PerformCut(&aOffsetShiftedNarrowBox, &aNurbsBox);
    ValidateResult(&aResult, 2.5, -1.0, false);
}

// Test bcut_simple/E4: NURBS box - extended Y shifted narrow box
#[test]
fn BCutSimpleTest__NurbsBoxMinusExtendedYShiftedNarrowBox_E4() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aExtendedYShiftedNarrowBox = CreateBox(Point3::new(0.25, 0.0, 0.0), 0.5, 1.5, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aExtendedYShiftedNarrowBox);
    ValidateResult(&aResult, 6.0, -1.0, false);
}
