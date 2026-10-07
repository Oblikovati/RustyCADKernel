//! BOPAlgo_OpenSolid_Test.cxx: the tests/boolean/opensolid A1..A9 cases.
#![allow(non_snake_case)]

use super::*;

// Equivalent to tests/boolean/opensolid/A1.
#[test]
fn BOPAlgo_OpenSolidTest__A1_TwoOpenBoxes() {
    let box1 = MakeOpenBox(origin(0.0, 0.0, 0.0), 10.0, 10.0, 10.0, 0, 4);
    let box2 = MakeOpenBox(origin(5.0, 0.0, 0.0), 10.0, 10.0, 10.0, 0, 4);
    CheckTwoShapeBOP(
        &box1,
        &box2,
        [
            OpenResult([8, 12, 5, 5, 1, 1], 350.0),
            OpenResult([16, 26, 11, 11, 1, 1], 650.0),
            OpenResult([8, 12, 5, 5, 1, 1], 350.0),
            OpenResult([8, 12, 5, 5, 1, 1], 350.0),
        ],
    );
}

// Equivalent to tests/boolean/opensolid/A2.
#[test]
fn BOPAlgo_OpenSolidTest__A2_OpenBoxesWithPartialIntersection() {
    let box1 = MakeOpenBox(origin(0.0, 0.0, 0.0), 10.0, 10.0, 10.0, 0, 4);
    let box2 = MakeOpenBox(origin(2.5, 0.0, 0.0), 5.0, 10.0, 5.0, 0, 4);
    CheckTwoShapeBOP(
        &box1,
        &box2,
        [
            OpenResult([8, 12, 5, 5, 1, 1], 200.0),
            OpenResult([16, 24, 9, 9, 1, 1], 500.0),
            OpenResult([16, 24, 8, 8, 1, 1], 500.0),
            OpenResult([0, 0, 0, 0, 0, 0], 0.0),
        ],
    );
}

// Equivalent to tests/boolean/opensolid/A3.
#[test]
fn BOPAlgo_OpenSolidTest__A3_OpenBoxesWithDifferentOrientation() {
    let box1 = MakeOpenBox(origin(-15.0, 0.0, 0.0), 20.0, 5.0, 5.0, 1, 5);
    let box2 = MakeOpenBox(origin(0.0, 0.0, 0.0), 5.0, 5.0, 20.0, 0, 4);
    CheckTwoShapeBOP(
        &box1,
        &box2,
        [
            OpenResult([8, 12, 6, 6, 1, 1], 150.0),
            OpenResult([16, 28, 12, 12, 1, 1], 700.0),
            OpenResult([8, 12, 5, 5, 1, 1], 325.0),
            OpenResult([8, 12, 5, 5, 1, 1], 325.0),
        ],
    );
}

// Equivalent to tests/boolean/opensolid/A4.
#[test]
fn BOPAlgo_OpenSolidTest__A4_MultiObjectOpenSolidOperations() {
    let box1 = make_box(origin(0.0, 0.0, 0.0), 40.0, 10.0, 10.0);
    let box2 = make_box(origin(0.0, 0.0, 30.0), 40.0, 10.0, 10.0);
    let box3 = make_box(origin(0.0, 0.0, 0.0), 10.0, 10.0, 40.0);
    let box4 = make_box(origin(30.0, 0.0, 0.0), 10.0, 10.0, 40.0);
    let fuse = expectation([32, 64, 32, 32, 1, 1], 4800.0, 12000.0);
    let cut = expectation([16, 24, 12, 12, 2, 2], 2000.0, 4000.0);
    let single_cut = expectation([8, 12, 6, 6, 1, 1], 1000.0, 2000.0);
    let common = expectation([32, 48, 24, 24, 4, 4], 2400.0, 4000.0);
    let single_common = expectation([16, 24, 12, 12, 2, 2], 1200.0, 2000.0);
    let all = list(&[box1.clone(), box2.clone(), box3.clone(), box4.clone()]);
    let objects = list(&[box1.clone(), box2.clone()]);
    let tools = list(&[box3.clone(), box4.clone()]);
    let all_tools = list(&[box2.clone(), box3, box4]);
    CheckBuildBOP(
        &all,
        &all,
        &List::default(),
        Operation::Fuse,
        &fuse,
        "all fuse",
    );
    CheckBuildBOP(
        &all,
        &objects,
        &tools,
        Operation::Fuse,
        &fuse,
        "objects/tools fuse",
    );
    CheckBuildBOP(
        &all,
        &objects,
        &tools,
        Operation::Cut,
        &cut,
        "objects/tools cut",
    );
    CheckBuildBOP(
        &all,
        &tools,
        &objects,
        Operation::Cut,
        &cut,
        "tools/objects cut",
    );
    CheckBuildBOP(
        &all,
        &objects,
        &tools,
        Operation::Common,
        &common,
        "objects/tools common",
    );
    let single_object = list(&[box1]);
    let second_object = list(&[box2]);
    CheckBuildBOP(
        &all,
        &single_object,
        &all_tools,
        Operation::Fuse,
        &fuse,
        "single object/tools fuse",
    );
    CheckBuildBOP(
        &all,
        &single_object,
        &tools,
        Operation::Cut,
        &single_cut,
        "single object/tools cut",
    );
    CheckBuildBOP(
        &all,
        &second_object,
        &tools,
        Operation::Cut,
        &single_cut,
        "second object/tools cut",
    );
    CheckBuildBOP(
        &all,
        &single_object,
        &tools,
        Operation::Common,
        &single_common,
        "first object/tools common",
    );
    CheckBuildBOP(
        &all,
        &second_object,
        &tools,
        Operation::Common,
        &single_common,
        "second object/tools common",
    );
}

// Equivalent to tests/boolean/opensolid/A5.
#[test]
fn BOPAlgo_OpenSolidTest__A5_InvertedToolOpenSolidOperations() {
    let box1 = make_box(origin(0.0, 0.0, 0.0), 10.0, 2.0, 2.0);
    let box2 = make_box(origin(2.0, 0.5, -3.0), 1.0, 1.0, 8.0);
    let mut box3 = make_box(origin(7.0, 0.5, -3.0), 1.0, 1.0, 8.0);
    box3.reverse();
    let tools = list(&[box2, box3]);
    let object = list(&[box1]);
    CheckBOP(
        &object,
        &tools,
        Operation::Common,
        &expectation([24, 36, 20, 16, 2, 2], 110.0, 38.0),
    );
    CheckBOP(
        &object,
        &tools,
        Operation::Fuse,
        &expectation([16, 24, 12, 12, 2, 2], 28.0, -6.0),
    );
    CheckBOP(
        &object,
        &tools,
        Operation::Cut,
        &expectation([8, 12, 6, 6, 1, 1], 10.0, 2.0),
    );
    CheckBOP(
        &tools,
        &object,
        Operation::Cut,
        &expectation([40, 60, 32, 28, 3, 3], 164.0, -46.0),
    );
}

// Equivalent to tests/boolean/opensolid/A6.
#[test]
fn BOPAlgo_OpenSolidTest__A6_NestedOpenSolidOperations() {
    let box1 = make_box(origin(0.0, 0.0, 0.0), 10.0, 2.0, 2.0);
    let box2 = make_box(origin(5.0, 0.0, 0.0), 5.0, 5.0, 5.0);
    let box3 = make_box(origin(6.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let tools = list(&[box2, box3]);
    let object = list(&[box1]);
    CheckBOP(
        &object,
        &tools,
        Operation::Common,
        &expectation([16, 25, 12, 12, 2, 2], 56.0, 20.0),
    );
    CheckBOP(
        &object,
        &tools,
        Operation::Fuse,
        &expectation([24, 38, 16, 16, 1, 1], 190.0, 145.0),
    );
    CheckBOP(
        &object,
        &tools,
        Operation::Cut,
        &expectation([8, 12, 6, 6, 1, 1], 48.0, 20.0),
    );
    CheckBOP(
        &tools,
        &object,
        Operation::Cut,
        &expectation([12, 18, 8, 8, 1, 1], 142.0, 105.0),
    );
}

// Equivalent to tests/boolean/opensolid/A7.
#[test]
fn BOPAlgo_OpenSolidTest__A7_ThreeOpenSolidOperations() {
    let box1 = make_box(origin(0.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let box2 = make_box(origin(1.0, 1.0, 1.0), 8.0, 8.0, 8.0);
    let box3 = make_box(origin(-2.0, 3.0, 3.0), 14.0, 4.0, 4.0);
    let one = list(&[box3]);
    let two = list(&[box1, box2]);
    CheckBOP(
        &one,
        &two,
        Operation::Common,
        &expectation([16, 28, 16, 16, 3, 3], 256.0, 160.0),
    );
    CheckBOP(
        &one,
        &two,
        Operation::Fuse,
        &expectation([24, 36, 18, 16, 1, 1], 664.0, 1064.0),
    );
    CheckBOP(
        &one,
        &two,
        Operation::Cut,
        &expectation([16, 24, 12, 12, 2, 2], 128.0, 64.0),
    );
    CheckBOP(
        &two,
        &one,
        Operation::Cut,
        &expectation([32, 52, 28, 24, 2, 2], 1432.0, 840.0),
    );
}

// Equivalent to tests/boolean/opensolid/A8.
#[test]
fn BOPAlgo_OpenSolidTest__A8_OverlappingOpenSolidOperations() {
    let box1 = make_box(origin(0.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let box2 = make_box(origin(-2.0, -2.0, 2.0), 7.0, 14.0, 6.0);
    let box3 = make_box(origin(5.0, -2.0, 2.0), 7.0, 14.0, 6.0);
    let tools = list(&[box3, box2]);
    let object = list(&[box1]);
    CheckBOP(
        &object,
        &tools,
        Operation::Common,
        &expectation([12, 20, 11, 11, 2, 2], 560.0, 600.0),
    );
    CheckBOP(
        &object,
        &tools,
        Operation::Fuse,
        &expectation([32, 50, 20, 20, 1, 1], 888.0, 1576.0),
    );
    CheckBOP(
        &object,
        &tools,
        Operation::Cut,
        &expectation([20, 30, 14, 14, 2, 2], 560.0, 400.0),
    );
    CheckBOP(
        &tools,
        &object,
        Operation::Cut,
        &expectation([24, 40, 18, 18, 2, 2], 816.0, 576.0),
    );
}

// Equivalent to tests/boolean/opensolid/A9.
#[test]
fn BOPAlgo_OpenSolidTest__A9_OpenSolidWithInternalShell() {
    let box1 = make_box(origin(0.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let box2 = make_box(origin(5.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let open1 = MakeOpenSolid(&box1, 0, 4, &MakeInternalFace(1.0));
    let open2 = MakeOpenSolid(&box2, 0, 4, &MakeInternalFace(14.0));
    let object = list(&[open1]);
    let tool = list(&[open2]);
    CheckBOPWithFiller(
        &object,
        &tool,
        Operation::Common,
        &OpenResult([8, 12, 5, 5, 1, 1], 350.0),
    );
    CheckBOPWithFiller(
        &object,
        &tool,
        Operation::Fuse,
        &OpenResult([16, 26, 11, 11, 1, 1], 650.0),
    );
    CheckBOPWithFiller(
        &object,
        &tool,
        Operation::Cut,
        &OpenResult([12, 16, 6, 6, 2, 1], 414.0),
    );
    CheckBOPWithFiller(
        &tool,
        &object,
        Operation::Cut,
        &OpenResult([12, 16, 6, 6, 2, 1], 414.0),
    );
}
