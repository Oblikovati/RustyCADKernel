//! BRepAlgoAPI_Cut_Test cases ExtendedYShiftedNarrowBoxMinusNurbsBox_E5 .. SphereMinusBox_F9.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/E5: extended Y shifted narrow box - NURBS box
#[test]
fn BCutSimpleTest__ExtendedYShiftedNarrowBoxMinusNurbsBox_E5() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aExtendedYShiftedNarrowBox = CreateBox(Point3::new(0.25, 0.0, 0.0), 0.5, 1.5, 1.0);

    let aResult = PerformCut(&aExtendedYShiftedNarrowBox, &aNurbsBox);
    ValidateResult(&aResult, 2.5, -1.0, false);
}

// Test bcut_simple/E6: NURBS box - right half box
#[test]
fn BCutSimpleTest__NurbsBoxMinusRightHalfBox_E6() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aRightHalfBox = CreateBox(Point3::new(0.5, 0.0, 0.0), 1.0, 1.0, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aRightHalfBox);
    ValidateResult(&aResult, 5.5, -1.0, false);
}

// Test bcut_simple/E7: right half box - NURBS box
#[test]
fn BCutSimpleTest__RightHalfBoxMinusNurbsBox_E7() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aRightHalfBox = CreateBox(Point3::new(0.5, 0.0, 0.0), 1.0, 1.0, 0.5);

    let aResult = PerformCut(&aRightHalfBox, &aNurbsBox);
    ValidateResult(&aResult, 2.5, -1.0, false);
}

// Test bcut_simple/E8: NURBS box - offset right half box
#[test]
fn BCutSimpleTest__NurbsBoxMinusOffsetRightHalfBox_E8() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetRightHalfBox = CreateBox(Point3::new(0.5, 0.0, -0.5), 1.0, 1.0, 0.5);

    let aResult = PerformCut(&aNurbsBox, &aOffsetRightHalfBox);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/E9: offset right half box - NURBS box
#[test]
fn BCutSimpleTest__OffsetRightHalfBoxMinusNurbsBox_E9() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aOffsetRightHalfBox = CreateBox(Point3::new(0.5, 0.0, -0.5), 1.0, 1.0, 0.5);

    let aResult = PerformCut(&aOffsetRightHalfBox, &aNurbsBox);
    ValidateResult(&aResult, 4.0, -1.0, false);
}

// Test bcut_simple/F1: NURBS box - rotated rectangular box (sqrt(2) x sqrt(2)/2 x 1, 45deg)
#[test]
fn BCutSimpleTest__NurbsBoxMinusRotatedRectangularBox_F1() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    let r = SQRT_2;
    let mut aRectangularBox = CreateBox(Point3::new(0.0, 0.0, 0.0), r, r / 2.0, 1.0);
    aRectangularBox = RotateShape(
        &aRectangularBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        45.0 * PI / 180.0,
    );
    let aResult = PerformCut(&aNurbsBox, &aRectangularBox);
    ValidateResult(&aResult, 4.41421, -1.0, false);
}

// Test bcut_simple/F2: rotated rectangular box - NURBS box
#[test]
fn BCutSimpleTest__RotatedRectangularBoxMinusNurbsBox_F2() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    let r = SQRT_2;
    let mut aRectangularBox = CreateBox(Point3::new(0.0, 0.0, 0.0), r, r / 2.0, 1.0);
    aRectangularBox = RotateShape(
        &aRectangularBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        45.0 * PI / 180.0,
    );
    let aResult = PerformCut(&aRectangularBox, &aNurbsBox);
    ValidateResult(&aResult, 5.82843, -1.0, false);
}

// Test bcut_simple/F3: NURBS box - rotated square box (sqrt(2)/2 x sqrt(2)/2 x 1, 45deg)
#[test]
fn BCutSimpleTest__NurbsBoxMinusRotatedSquareBox_F3() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    let r = f64::sqrt(2.0) / 2.0;
    let mut aSquareBox = CreateBox(Point3::new(0.0, 0.0, 0.0), r, r, 1.0);
    aSquareBox = RotateShape(
        &aSquareBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        45.0 * PI / 180.0,
    );
    let aResult = PerformCut(&aNurbsBox, &aSquareBox);
    ValidateResult(&aResult, 5.91421, -1.0, false);
}

// Test bcut_simple/F4: rotated square box - NURBS box
#[test]
fn BCutSimpleTest__RotatedSquareBoxMinusNurbsBox_F4() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    let r = f64::sqrt(2.0) / 2.0;
    let mut aSquareBox = CreateBox(Point3::new(0.0, 0.0, 0.0), r, r, 1.0);
    aSquareBox = RotateShape(
        &aSquareBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        45.0 * PI / 180.0,
    );
    let aResult = PerformCut(&aSquareBox, &aNurbsBox);
    ValidateResult(&aResult, 2.91421, -1.0, false);
}

// Test bcut_simple/F5: NURBS box - rotated thin box (sqrt(2) x 0.25 x 1, 45deg)
#[test]
fn BCutSimpleTest__NurbsBoxMinusRotatedThinBox_F5() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    let r = SQRT_2;
    let mut aThinBox = CreateBox(Point3::new(0.0, 0.0, 0.0), r, 0.25, 1.0);
    aThinBox = RotateShape(
        &aThinBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        45.0 * PI / 180.0,
    );
    let aResult = PerformCut(&aNurbsBox, &aThinBox);
    ValidateResult(&aResult, 7.03921, -1.0, false);
}

// Test bcut_simple/F6: rotated thin box - NURBS box
#[test]
fn BCutSimpleTest__RotatedThinBoxMinusNurbsBox_F6() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    let r = SQRT_2;
    let mut aThinBox = CreateBox(Point3::new(0.0, 0.0, 0.0), r, 0.25, 1.0);
    aThinBox = RotateShape(
        &aThinBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        45.0 * PI / 180.0,
    );
    let aResult = PerformCut(&aThinBox, &aNurbsBox);
    ValidateResult(&aResult, 1.83211, -1.0, false);
}

// Test bcut_simple/F7: NURBS box - rotated narrow box (sqrt(31)/4 x 0.25 x 1, 34.73deg)
#[test]
fn BCutSimpleTest__NurbsBoxMinusRotatedNarrowBox_F7() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    let r = 5.5677643628300219;
    let mut aNarrowBox = CreateBox(Point3::new(0.0, 0.0, 0.0), r / 4.0, 0.25, 1.0);
    aNarrowBox = RotateShape(
        &aNarrowBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        34.73 * PI / 180.0,
    );
    let aResult = PerformCut(&aNurbsBox, &aNarrowBox);
    ValidateResult(&aResult, 7.21677, -1.0, false);
}

// Test bcut_simple/F8: rotated narrow box - NURBS box
#[test]
fn BCutSimpleTest__RotatedNarrowBoxMinusNurbsBox_F8() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    let r = 5.5677643628300219;
    let mut aNarrowBox = CreateBox(Point3::new(0.0, 0.0, 0.0), r / 4.0, 0.25, 1.0);
    aNarrowBox = RotateShape(
        &aNarrowBox,
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        34.73 * PI / 180.0,
    );
    let aResult = PerformCut(&aNarrowBox, &aNurbsBox);
    ValidateResult(&aResult, 1.54631, -1.0, false);
}

// Test bcut_simple/F9: sphere - box
#[test]
fn BCutSimpleTest__SphereMinusBox_F9() {
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let aResult = PerformCut(&aSphere, &aBox);
    ValidateResult(&aResult, 13.3518, -1.0, false);
}
