//! BRepAlgoAPI_Fuse_Test cases CornerCubePlusNurbsBox_C7 .. ProfileBasedPrisms_D9.
#![allow(non_snake_case)]

use super::*;

// Test bfuse_simple/C7: corner cube + nurbs box
#[test]
fn BFuseSimpleTest__CornerCubePlusNurbsBox_C7() {
    let aBox1 = CreateBox(Point3::new(0.5, 0.5, 0.5), 0.5, 0.5, 0.5);

    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox2 = ConvertToNurbs(&aBox2);
    assert!(!aBox2.is_null(), "Failed to convert to NURBS");

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bfuse_simple/C8: nurbs box + offset cube
#[test]
fn BFuseSimpleTest__NurbsBoxPlusOffsetCube_C8() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    let aBox2 = CreateBox(Point3::new(1.0, 1.0, 0.0), 0.5, 0.5, 1.0);

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.25, -1.0, false);
}

// Test bfuse_simple/C9: offset cube + nurbs box
#[test]
fn BFuseSimpleTest__OffsetCubePlusNurbsBox_C9() {
    let aBox1 = CreateBox(Point3::new(1.0, 1.0, 0.0), 0.5, 0.5, 1.0);

    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox2 = ConvertToNurbs(&aBox2);
    assert!(!aBox2.is_null(), "Failed to convert to NURBS");

    let aResult = PerformFuse(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.25, -1.0, false);
}

// Test bfuse_simple/D1: nurbs box + rotated narrow box
#[test]
fn BFuseSimpleTest__NurbsBoxPlusRotatedNarrowBox_D1() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);

    // Create rotated narrow box: r=sqrt(2), box(0,0,0,r,0.25,1), rotate 45 degrees around Z
    let r = SQRT_2;
    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), r, 0.25, 1.0);
    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
            ),
            PI / 4.0,
        )
        .unwrap();
    aBox2.move_by(&Location::new(aTrsf), false).unwrap();

    let aResult = PerformFuse(&aBox2, &aBox1);
    ValidateResult(&aResult, 6.41789, -1.0, false);
}

// Test bfuse_simple/D2: nurbs box + rotated narrow box variation
#[test]
fn BFuseSimpleTest__NurbsBoxPlusRotatedNarrowBoxVariation_D2() {
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);

    // Create rotated narrow box: r=sqrt(31), box(0,0,0,r/4,0.25,1), rotate 34.73 degrees around Z
    let r = 5.5677643628300219;
    let mut aBox2 = CreateBox(Point3::new(0.0, 0.0, 0.0), r / 4.0, 0.25, 1.0);
    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
            ),
            34.73 * PI / 180.0,
        )
        .unwrap();
    aBox2.move_by(&Location::new(aTrsf), false).unwrap();

    let aResult = PerformFuse(&aBox2, &aBox1);
    ValidateResult(&aResult, 6.32953, -1.0, false);
}

// Test bfuse_simple/D3: sphere + box
#[test]
fn BFuseSimpleTest__SpherePlusBox_D3() {
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformFuse(&aSphere, &aBox);
    ValidateResult(&aResult, 14.6394, -1.0, false);
}

// Test bfuse_simple/D4: sphere + box (reversed order)
#[test]
fn BFuseSimpleTest__BoxPlusSphere_D4() {
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformFuse(&aBox, &aSphere);
    ValidateResult(&aResult, 14.6394, -1.0, false);
}

// Test bfuse_simple/D5: rotated sphere + box
#[test]
fn BFuseSimpleTest__RotatedSpherePlusBox_D5() {
    let mut aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);

    // Apply rotations: -90 degrees around Z, then -45 degrees around Y
    let mut aTrsf1 = Transform3::default();
    let mut aTrsf2 = Transform3::default();
    aTrsf1
        .set_rotation(
            Axis1::new(
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
            ),
            -PI / 2.0,
        )
        .unwrap();
    aTrsf2
        .set_rotation(
            Axis1::new(
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
            ),
            -PI / 4.0,
        )
        .unwrap();
    let aTrsfCombined = aTrsf2.multiplied(aTrsf1);
    aSphere
        .move_by(&Location::new(aTrsfCombined), false)
        .unwrap();

    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let aResult = PerformFuse(&aSphere, &aBox);
    ValidateResult(&aResult, 14.6393, -1.0, false);
}

// Test bfuse_simple/D6: rotated sphere + box (reversed order)
#[test]
fn BFuseSimpleTest__BoxPlusRotatedSphere_D6() {
    let mut aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);

    // Apply rotations: -90 degrees around Z, then -45 degrees around Y
    let mut aTrsf1 = Transform3::default();
    let mut aTrsf2 = Transform3::default();
    aTrsf1
        .set_rotation(
            Axis1::new(
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
            ),
            -PI / 2.0,
        )
        .unwrap();
    aTrsf2
        .set_rotation(
            Axis1::new(
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
            ),
            -PI / 4.0,
        )
        .unwrap();
    let aTrsfCombined = aTrsf2.multiplied(aTrsf1);
    aSphere
        .move_by(&Location::new(aTrsfCombined), false)
        .unwrap();

    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let aResult = PerformFuse(&aBox, &aSphere);
    ValidateResult(&aResult, 14.6393, -1.0, false);
}

// Test bfuse_simple/D7: sphere + rotated box
#[test]
fn BFuseSimpleTest__SpherePlusRotatedBox_D7() {
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);

    let mut aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    // Rotate box 90 degrees around Y axis: "trotate b 0 0 1 0 1 0 90"
    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(
                Point3::new(0.0, 0.0, 1.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
            ),
            PI / 2.0,
        )
        .unwrap();
    aBox.move_by(&Location::new(aTrsf), false).unwrap();

    let aResult = PerformFuse(&aSphere, &aBox);
    ValidateResult(&aResult, 14.6393, -1.0, false);
}

// Test bfuse_simple/D8: rotated box + sphere (reversed order)
#[test]
fn BFuseSimpleTest__RotatedBoxPlusSphere_D8() {
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.0);

    let mut aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    // Rotate box 90 degrees around Y axis: "trotate b 0 0 1 0 1 0 90"
    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(
                Point3::new(0.0, 0.0, 1.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
            ),
            PI / 2.0,
        )
        .unwrap();
    aBox.move_by(&Location::new(aTrsf), false).unwrap();

    let aResult = PerformFuse(&aBox, &aSphere);
    ValidateResult(&aResult, 14.6393, -1.0, false);
}

// Test bfuse_simple/D9: profile-based prisms (cts40125 bug)
#[test]
fn BFuseSimpleTest__ProfileBasedPrisms_D9() {
    // Create first profile: "profile f1 x 100 y 100 x -200 y -200 x 100"
    // This creates a closed rectangular profile starting at origin
    let aPlane1 = Plane3::from_position(Axis3::from_axis2(
        Axis2::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        )
        .unwrap(),
    ));
    let aProfileOps1 = [
        ProfileOperation::new(ProfileCmd::X, &[100.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[-200.0]),
        ProfileOperation::new(ProfileCmd::Y, &[-200.0]),
        ProfileOperation::new(ProfileCmd::X, &[100.0]),
    ];
    let aProfile1 = CreateProfile(&aPlane1, &aProfileOps1);
    let aPrism1 = CreatePrism(&aProfile1, Vector3::new(0.0, 0.0, 100.0));

    // Create second profile: "profile f2 x -100 y 100 x 100; ttranslate f2 0 0 100"
    // This creates a triangular profile translated to z=100
    let aPlane2 = Plane3::from_position(Axis3::from_axis2(
        Axis2::new(
            Point3::new(0.0, 0.0, 100.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        )
        .unwrap(),
    ));
    let aProfileOps2 = [
        ProfileOperation::new(ProfileCmd::X, &[-100.0]),
        ProfileOperation::new(ProfileCmd::Y, &[100.0]),
        ProfileOperation::new(ProfileCmd::X, &[100.0]),
    ];
    let aProfile2 = CreateProfile(&aPlane2, &aProfileOps2);
    let aPrism2 = CreatePrism(&aProfile2, Vector3::new(0.0, 0.0, 100.0));

    let aResult = PerformFuse(&aPrism1, &aPrism2);
    ValidateResult(&aResult, 180000.0, -1.0, false);
}
