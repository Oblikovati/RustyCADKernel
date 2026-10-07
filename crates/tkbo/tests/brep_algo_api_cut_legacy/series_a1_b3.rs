//! BRepAlgoAPI_Cut_Test cases SphereMinusBox_A1 .. NurbsBoxMinusAdjacentBox_B3.
#![allow(non_snake_case)]

use super::*;

// Test bcut_simple/A1: psphere s 1; box b 1 1 1; bcut result s b; checkprops result -s 13.3518
#[test]
fn BCutSimpleTest__SphereMinusBox_A1() {
    let aSphere = CreateUnitSphere();
    let aBox = CreateUnitBox();

    let aResult = PerformCut(&aSphere, &aBox);
    ValidateResult(&aResult, 13.3518, -1.0, false);
}

// Migrated from tests/bugs/modalg_5/bug24706.  A Boolean cut must retain the
// Closed flag on the shell of the resulting box-with-spherical-cut solid.
#[test]
fn BCutSimpleTest__ModalgBug_24706_CutResultShellIsClosed() {
    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let aSphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 2.0);

    let mut aCut = BRepAlgoAPICut::from_shapes(&aBox, &aSphere).unwrap();
    aCut.build().unwrap();
    assert!(aCut.is_done());
    let aResult = aCut.shape().clone();
    assert!(!aResult.is_null());
    assert!(
        BRepCheckAnalyzer::new(&aResult, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    assert_eq!(countSubShapes(&aResult, ShapeEnum::Solid), 1);
    assert_eq!(countSubShapes(&aResult, ShapeEnum::Shell), 1);

    let anExplorer = Explorer::new(&aResult, ShapeEnum::Shell, ShapeEnum::Shape).unwrap();
    assert!(anExplorer.more());
    assert!(
        topods::shell(anExplorer.current())
            .unwrap()
            .closed()
            .unwrap()
    );
}

// Migrated from tests/bugs/modalg_5/bug25242.  Cutting one disjoint member
// from a fused compound must leave the other box with the DRAW topology.
#[test]
fn BCutSimpleTest__ModalgBug_25242_CutFusedDisjointBoxesKeepsTopology() {
    let aBoxA = CreateBox(Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let aBoxB = CreateBox(Point3::new(20.0, 0.0, 0.0), 10.0, 10.0, 10.0);

    let mut aFuse = BRepAlgoAPIFuse::from_shapes(&aBoxA, &aBoxB).unwrap();
    aFuse.build().unwrap();
    assert!(aFuse.is_done());

    let mut aCut = BRepAlgoAPICut::from_shapes(&aFuse.shape().clone(), &aBoxA).unwrap();
    aCut.build().unwrap();
    assert!(aCut.is_done());
    let aResult = aCut.shape().clone();
    assert!(!aResult.is_null());

    assert_eq!(aResult.shape_type().unwrap(), ShapeEnum::Compound);
    assert_eq!(countUniqueSubShapes(&aResult, ShapeEnum::Vertex), 8);
    assert_eq!(countUniqueSubShapes(&aResult, ShapeEnum::Edge), 12);
    assert_eq!(countUniqueSubShapes(&aResult, ShapeEnum::Wire), 6);
    assert_eq!(countUniqueSubShapes(&aResult, ShapeEnum::Face), 6);
    assert_eq!(countUniqueSubShapes(&aResult, ShapeEnum::Shell), 1);
    assert_eq!(countUniqueSubShapes(&aResult, ShapeEnum::Solid), 1);
    expect_near(GetVolume(&aResult), 1000.0, precision::confusion());
}

// Test bcut_simple/A2: rotated sphere - box
#[test]
fn BCutSimpleTest__RotatedSphereMinusBox_A2() {
    let aSphere = CreateUnitSphere();

    // Apply standard rotation: Z(-90deg) then Y(-45deg)
    let aRotatedSphere = RotateStandard(&aSphere);

    let aBox = CreateUnitBox();
    let aResult = PerformCut(&aRotatedSphere, &aBox);
    ValidateResult(&aResult, 13.3517, -1.0, false);
}

// Test bcut_simple/A3: box - rotated sphere (reverse operation)
#[test]
fn BCutSimpleTest__BoxMinusRotatedSphere_A3() {
    let aSphere = CreateUnitSphere();

    // Apply standard rotation: Z(-90deg) then Y(-45deg)
    let aRotatedSphere = RotateStandard(&aSphere);

    let aBox = CreateUnitBox();
    let aResult = PerformCut(&aBox, &aRotatedSphere);
    ValidateResult(&aResult, 5.2146, -1.0, false);
}

// Test bcut_simple/A4: sphere - Y-rotated box
#[test]
fn BCutSimpleTest__SphereMinusRotatedBox_A4() {
    let aSphere = CreateUnitSphere();
    let aBox = CreateUnitBox();
    let aRotatedBox = RotateY90(&aBox);

    let aResult = PerformCut(&aSphere, &aRotatedBox);
    ValidateResult(&aResult, 13.3517, -1.0, false);
}

// Test bcut_simple/A5: Y-rotated box - sphere (reverse operation)
#[test]
fn BCutSimpleTest__RotatedBoxMinusSphere_A5() {
    let aSphere = CreateUnitSphere();
    let aBox = CreateUnitBox();
    let aRotatedBox = RotateY90(&aBox);

    let aResult = PerformCut(&aRotatedBox, &aSphere);
    ValidateResult(&aResult, 5.2146, -1.0, false);
}

// Test bcut_simple/A6: identical NURBS box - box (should result in empty)
#[test]
fn BCutSimpleTest__IdenticalNurbsBoxMinusBox_A6() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aBox);
    ValidateResult(&aResult, -1.0, -1.0, true);
}

// Test bcut_simple/A7: identical box - NURBS box (should result in empty)
#[test]
fn BCutSimpleTest__IdenticalBoxMinusNurbsBox_A7() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformCut(&aBox, &aNurbsBox);
    ValidateResult(&aResult, -1.0, -1.0, true);
}

// Test bcut_simple/A8: NURBS box - larger box (should result in empty)
#[test]
fn BCutSimpleTest__NurbsBoxMinusLargerBox_A8() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aLargerBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.5, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aLargerBox);
    ValidateResult(&aResult, -1.0, -1.0, true);
}

// Test bcut_simple/A9: larger box - NURBS box
#[test]
fn BCutSimpleTest__LargerBoxMinusNurbsBox_A9() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aLargerBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.5, 1.0);

    let aResult = PerformCut(&aLargerBox, &aNurbsBox);
    ValidateResult(&aResult, 4.0, -1.0, false);
}

// Test bcut_simple/B1: nurbs box - box
#[test]
fn BCutSimpleTest__NurbsBoxMinusBox_B1() {
    // Create first box and convert to NURBS
    let mut aBox1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aBox1 = ConvertToNurbs(&aBox1);
    assert!(!aBox1.is_null(), "Failed to convert to NURBS");

    // Create second box
    let aBox2 = CreateBox(Point3::new(0.0, 1.0, 0.0), 1.0, 0.5, 1.0);

    let aResult = PerformCut(&aBox1, &aBox2);
    ValidateResult(&aResult, 6.0, -1.0, false);
}

// Test bcut_simple/B2: box - NURBS box (reverse B1)
#[test]
fn BCutSimpleTest__BoxMinusNurbsBox_B2() {
    // Create first box and convert to NURBS
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    // Create second box
    let aBox = CreateBox(Point3::new(0.0, 1.0, 0.0), 1.0, 0.5, 1.0);

    let aResult = PerformCut(&aBox, &aNurbsBox);
    ValidateResult(&aResult, 4.0, -1.0, false);
}

// Test bcut_simple/B3: NURBS box - adjacent box
#[test]
fn BCutSimpleTest__NurbsBoxMinusAdjacentBox_B3() {
    let mut aNurbsBox = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    aNurbsBox = ConvertToNurbs(&aNurbsBox);
    assert!(!aNurbsBox.is_null(), "Failed to convert to NURBS");

    let aAdjacentBox = CreateBox(Point3::new(1.0, 1.0, 0.0), 1.0, 1.0, 1.0);

    let aResult = PerformCut(&aNurbsBox, &aAdjacentBox);
    ValidateResult(&aResult, 6.0, -1.0, false);
}
