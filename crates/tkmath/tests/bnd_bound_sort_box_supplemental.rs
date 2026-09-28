// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Supplemental translation of GTests/Bnd_BoundSortBox_Test.cxx.
//!
//! The upstream file links BRepPrimAPI/BRepBndLib (modeling toolkits), so its native
//! GoogleTest run is not mapped yet; every body is translated here, the B-rep boxes of
//! BUC60729 replaced by the equivalent plain boxes. Each test names its upstream case.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};

// Helper function to create a box from min and max points
fn CreateBox(xmin: f64, ymin: f64, zmin: f64, xmax: f64, ymax: f64, zmax: f64) -> BoundingBox3 {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([xmin, ymin, zmin], [xmax, ymax, zmax]);
    aBox
}

/// The `Bnd_BoundSortBoxTest` fixture.
struct Fixture {
    mySmallBox: BoundingBox3,
    myLargeBox: BoundingBox3,
    myOffsetBox: BoundingBox3,
    myNonIntersectingBox: BoundingBox3,
    myGlobalBox: BoundingBox3,
    myBoxes: Array1<'static, BoundingBox3>,
}
fn SetUp() -> Fixture {
    let mySmallBox = CreateBox(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    let myLargeBox = CreateBox(-10.0, -10.0, -10.0, 10.0, 10.0, 10.0);
    let myOffsetBox = CreateBox(5.0, 5.0, 5.0, 7.0, 7.0, 7.0);
    let myNonIntersectingBox = CreateBox(20.0, 20.0, 20.0, 30.0, 30.0, 30.0);
    let mut myBoxes = Array1::with_bounds(1, 4).unwrap();
    myBoxes[1] = mySmallBox.clone();
    myBoxes[2] = myLargeBox.clone();
    myBoxes[3] = myOffsetBox.clone();
    myBoxes[4] = myNonIntersectingBox.clone();
    let myGlobalBox = CreateBox(-20.0, -20.0, -20.0, 40.0, 40.0, 40.0);
    Fixture {
        mySmallBox,
        myLargeBox,
        myOffsetBox,
        myNonIntersectingBox,
        myGlobalBox,
        myBoxes,
    }
}
fn contains(result: &List<i32>, index: i32) -> bool {
    result.iter().any(|value| *value == index)
}
fn two_boxes(first: BoundingBox3, second: BoundingBox3) -> Array1<'static, BoundingBox3> {
    let mut boxes = Array1::with_bounds(1, 2).unwrap();
    boxes[1] = first;
    boxes[2] = second;
    boxes
}

/// Upstream `Bnd_BoundSortBoxTest.InitializeWithBoxes`.
#[test]
fn Bnd_BoundSortBoxTest__InitializeWithBoxes() {
    let f = SetUp();
    let mut sortBox = BoundSortBox::default();
    sortBox.initialize(f.myBoxes.clone()).unwrap();

    // Test comparing with a box that intersects mySmallBox
    let testBox = CreateBox(0.5, 0.5, 0.5, 1.5, 1.5, 1.5);
    let result = sortBox.compare_box(&testBox).unwrap();
    assert_eq!(2, result.extent(), "Expected to find 2 intersections");

    // Check that box indices 1 (mySmallBox) and 2 (myLargeBox) are in the result
    assert!(
        contains(result, 1),
        "Small box (index 1) should be in the result"
    );
    assert!(
        contains(result, 2),
        "Large box (index 2) should be in the result"
    );
}

/// Upstream `Bnd_BoundSortBoxTest.InitializeWithEnclosingBox`.
#[test]
fn Bnd_BoundSortBoxTest__InitializeWithEnclosingBox() {
    let f = SetUp();
    let mut sortBox = BoundSortBox::default();
    sortBox
        .initialize_with_enclosing(&f.myGlobalBox, f.myBoxes.clone())
        .unwrap();

    // Test comparing with myOffsetBox
    let result = sortBox.compare_box(&f.myOffsetBox).unwrap();
    assert_eq!(2, result.extent(), "Expected to find 2 intersections");

    // Check that box indices 2 (myLargeBox) and 3 (myOffsetBox) are in the result
    assert!(
        contains(result, 2),
        "Large box (index 2) should be in the result"
    );
    assert!(
        contains(result, 3),
        "Offset box (index 3) should be in the result"
    );
}

/// Upstream `Bnd_BoundSortBoxTest.InitializeWithCount`.
#[test]
fn Bnd_BoundSortBoxTest__InitializeWithCount() {
    let f = SetUp();
    let mut sortBox = BoundSortBox::default();
    sortBox.initialize_with_count(&f.myGlobalBox, 3).unwrap();

    // Add boxes manually
    sortBox.add(&f.mySmallBox, 1).unwrap();
    sortBox.add(&f.myLargeBox, 2).unwrap();
    sortBox.add(&f.myNonIntersectingBox, 3).unwrap();

    // Test comparing with a box that should only intersect myLargeBox
    let testBox = CreateBox(-5.0, -5.0, -5.0, -2.0, -2.0, -2.0);
    let result = sortBox.compare_box(&testBox).unwrap();
    assert_eq!(1, result.extent(), "Expected to find 1 intersection");

    // Verify that only the large box was found
    assert_eq!(
        Some(&2),
        result.first(),
        "Large box (index 2) should be the only result"
    );
}

/// Upstream `Bnd_BoundSortBoxTest.CompareWithPlane`.
#[test]
fn Bnd_BoundSortBoxTest__CompareWithPlane() {
    let f = SetUp();
    let mut sortBox = BoundSortBox::default();
    sortBox.initialize(f.myBoxes.clone()).unwrap();

    // Create a plane that intersects the large box but not others
    let point = Point3::new(0.0, 0.0, 9.0);
    let direction = Direction3::Z;
    let plane = Plane3::new(point, direction).unwrap();

    let result = sortBox.compare_plane(plane).unwrap();

    // Only the large box should intersect this plane
    assert_eq!(1, result.extent(), "Expected to find 1 intersection");
    assert_eq!(
        Some(&2),
        result.first(),
        "Large box (index 2) should be the only result"
    );
}

/// Upstream `Bnd_BoundSortBoxTest.VoidBoxes`.
#[test]
fn Bnd_BoundSortBoxTest__VoidBoxes() {
    let f = SetUp();
    let void_box = BoundingBox3::default(); // Default constructed box is void
    let boxes = two_boxes(void_box.clone(), f.mySmallBox.clone());

    let mut sortBox = BoundSortBox::default();
    sortBox.initialize(boxes).unwrap();

    // Test comparing with mySmallBox - should only find itself
    let result = sortBox.compare_box(&f.mySmallBox).unwrap();
    assert_eq!(1, result.extent(), "Expected to find 1 intersection");
    assert_eq!(
        Some(&2),
        result.first(),
        "Small box (index 2) should be the only result"
    );

    // Test comparing with void box - should find nothing
    let void_result = sortBox.compare_box(&void_box).unwrap();
    assert_eq!(
        0,
        void_result.extent(),
        "Expected to find 0 intersections with void box"
    );
}

/// Upstream `Bnd_BoundSortBoxTest.TouchingBoxes`.
#[test]
fn Bnd_BoundSortBoxTest__TouchingBoxes() {
    // Create boxes that touch at a single point
    let box1 = CreateBox(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    let box2 = CreateBox(1.0, 1.0, 1.0, 2.0, 2.0, 2.0); // Touches box1 at (1,1,1)

    let mut sortBox = BoundSortBox::default();
    sortBox
        .initialize(two_boxes(box1.clone(), box2.clone()))
        .unwrap();

    // Test comparing with box1
    let result1 = sortBox.compare_box(&box1).unwrap();
    assert_eq!(2, result1.extent(), "Expected to find 2 intersections");

    // Test comparing with box2
    let result2 = sortBox.compare_box(&box2).unwrap();
    assert_eq!(2, result2.extent(), "Expected to find 2 intersections");
}

/// Upstream `Bnd_BoundSortBoxTest.DisjointBoxes`.
#[test]
fn Bnd_BoundSortBoxTest__DisjointBoxes() {
    let f = SetUp();
    let farBox = CreateBox(100.0, 100.0, 100.0, 110.0, 110.0, 110.0);
    let boxes = two_boxes(f.mySmallBox.clone(), farBox);

    // Create enclosing box large enough
    let enclosingBox = CreateBox(-10.0, -10.0, -10.0, 120.0, 120.0, 120.0);

    let mut sortBox = BoundSortBox::default();
    sortBox
        .initialize_with_enclosing(&enclosingBox, boxes)
        .unwrap();

    // Test comparing with a box near mySmallBox
    let testBox = CreateBox(0.5, 0.5, 0.5, 1.5, 1.5, 1.5);
    let result = sortBox.compare_box(&testBox).unwrap();
    assert_eq!(1, result.extent(), "Expected to find 1 intersection");
    assert_eq!(
        Some(&1),
        result.first(),
        "Small box (index 1) should be the only result"
    );
}

/// Upstream `Bnd_BoundSortBoxTest.DegenerateBoxes`.
#[test]
fn Bnd_BoundSortBoxTest__DegenerateBoxes() {
    // Create degenerate boxes
    let mut pointBox = BoundingBox3::default();
    pointBox.update_bounds([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]); // Point at (1,1,1)
    let mut lineBox = BoundingBox3::default();
    lineBox.update_bounds([2.0, 0.0, 0.0], [5.0, 0.0, 0.0]); // Line along X from (2,0,0) to (5,0,0)
    let mut planeBox = BoundingBox3::default();
    planeBox.update_bounds([0.0, 0.0, 3.0], [3.0, 3.0, 3.0]); // XY plane at Z=3

    let mut boxes = Array1::with_bounds(1, 3).unwrap();
    boxes[1] = pointBox;
    boxes[2] = lineBox;
    boxes[3] = planeBox;

    let mut sortBox = BoundSortBox::default();
    sortBox.initialize(boxes).unwrap();

    // Test with a box that should intersect all three degenerate boxes
    let testBox = CreateBox(0.0, 0.0, 0.0, 6.0, 6.0, 6.0);
    let result = sortBox.compare_box(&testBox).unwrap();
    assert_eq!(
        3,
        result.extent(),
        "Expected to find 3 intersections with degenerate boxes"
    );

    // Test with a box that should intersect only the point
    let pointTestBox = CreateBox(0.5, 0.5, 0.5, 1.5, 1.5, 1.5);
    let pointResult = sortBox.compare_box(&pointTestBox).unwrap();
    assert_eq!(
        1,
        pointResult.extent(),
        "Expected to find only the point box"
    );
    assert_eq!(
        Some(&1),
        pointResult.first(),
        "Point box (index 1) should be the only result"
    );
}

/// Upstream `Bnd_BoundSortBox_Test.BUC60729_InitializeWithFaceBoxes`.
///
/// `BRepBndLib::Add` of `BRepPrimAPI_MakeBox(1,1,1)` and of each of its six faces gives the
/// unit cube and its face rectangles, each enlarged by the vertex/edge/face tolerance
/// `Precision::Confusion()`; those plain boxes replace the B-rep here.
#[test]
fn Bnd_BoundSortBox_Test__BUC60729_InitializeWithFaceBoxes() {
    let tolerance = rustycad_tkernel::precision::confusion();
    let mut aMainBox = CreateBox(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    aMainBox.enlarge(tolerance);

    // Initialize BoundSortBox with 6 boxes (for 6 faces of the cube)
    let aMaxNbrBox = 6;
    let mut aBoundSortBox = BoundSortBox::default();
    aBoundSortBox
        .initialize_with_count(&aMainBox, aMaxNbrBox)
        .unwrap();

    // Iterate through faces and add their bounding boxes
    let mut i = 1;
    for aFace in 0..6 {
        let (axis, side) = (aFace / 2, f64::from(aFace % 2));
        let mut low = [0.0; 3];
        let mut high = [1.0; 3];
        low[axis as usize] = side;
        high[axis as usize] = side;
        let mut aBox = BoundingBox3::default();
        aBox.update_bounds(low, high);
        aBox.enlarge(tolerance);
        aBoundSortBox.add(&aBox, i).unwrap();
        i += 1;
    }

    // Verify that 6 faces were processed
    assert_eq!(
        7, i,
        "Expected to process 6 faces (i should be 7 after loop)"
    );

    // The test passes if no exceptions occurred during initialization and addition
    // This test originally verified that the Bnd_BoundSortBox could handle
    // initialization and addition of multiple boxes without crashing
}
