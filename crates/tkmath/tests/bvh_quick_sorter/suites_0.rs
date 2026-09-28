// =============================================================================
// BVH_QuickSorter Basic Tests
// =============================================================================

#[test]
fn BVH_QuickSorterTest__Constructor() {
    let mut aSorterX = BvhQuickSorter::<BvhVec3d>::new(0);
    let mut aSorterY = BvhQuickSorter::<BvhVec3d>::new(1);
    let mut aSorterZ = BvhQuickSorter::<BvhVec3d>::new(2);

    // Constructor should not crash
    assert!(true);
}

#[test]
fn BVH_QuickSorterTest__SortEmptySet() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();
    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);

    // Sorting empty set should not crash
    aSorter.perform(&mut aTriangulation).unwrap();
    assert_eq!(aTriangulation.size(), 0);
}

#[test]
fn BVH_QuickSorterTest__SortSingleElement() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 1.0, 0.0));
    array::append(&mut aTriangulation.elements, BvhVec4i::new(0, 1, 2, 0));

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);
    aSorter.perform(&mut aTriangulation).unwrap();

    // Single element should remain unchanged
    assert_eq!(aTriangulation.size(), 1);
}

// =============================================================================
// Sorting Correctness Tests
// =============================================================================

#[test]
fn BVH_QuickSorterTest__SortAlongXAxis() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create triangles with increasing X centroids: 5, 1, 3
    // Triangle 0: centroid X = 5
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(4.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(5.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(6.0, 1.0, 0.0));
    array::append(&mut aTriangulation.elements, BvhVec4i::new(0, 1, 2, 0));

    // Triangle 1: centroid X = 1
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(2.0, 1.0, 0.0));
    array::append(&mut aTriangulation.elements, BvhVec4i::new(3, 4, 5, 0));

    // Triangle 2: centroid X = 3
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(2.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(3.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(4.0, 1.0, 0.0));
    array::append(&mut aTriangulation.elements, BvhVec4i::new(6, 7, 8, 0));

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0); // Sort along X
    aSorter.perform(&mut aTriangulation).unwrap();

    // After sorting: should be ordered 1, 3, 5 (indices 1, 2, 0)
    assert!(Center(&aTriangulation, 0, 0) < Center(&aTriangulation, 1, 0));
    assert!(Center(&aTriangulation, 1, 0) < Center(&aTriangulation, 2, 0));

    // Verify actual values
    assert_near(Center(&aTriangulation, 0, 0), 1.0, 1e-10);
    assert_near(Center(&aTriangulation, 1, 0), 3.0, 1e-10);
    assert_near(Center(&aTriangulation, 2, 0), 5.0, 1e-10);
}

#[test]
fn BVH_QuickSorterTest__SortAlongYAxis() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create triangles with Y centroids: 3, 1, 2
    for i in 0..3 {
        let y: f64 = if i == 0 {
            3.0
        } else if i == 1 {
            1.0
        } else {
            2.0
        };
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, y - 0.5, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, y, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.5, y + 0.5, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(1); // Sort along Y
    aSorter.perform(&mut aTriangulation).unwrap();

    // Should be ordered by Y centroid
    assert!(Center(&aTriangulation, 0, 1) < Center(&aTriangulation, 1, 1));
    assert!(Center(&aTriangulation, 1, 1) < Center(&aTriangulation, 2, 1));
}

#[test]
fn BVH_QuickSorterTest__SortAlongZAxis() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create triangles with Z centroids: 2, 0, 1
    for i in 0..3 {
        let z: f64 = if i == 0 {
            2.0
        } else if i == 1 {
            0.0
        } else {
            1.0
        };
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 0.0, z));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, 0.0, z));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.5, 1.0, z));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(2); // Sort along Z
    aSorter.perform(&mut aTriangulation).unwrap();

    // Should be ordered by Z centroid
    assert!(Center(&aTriangulation, 0, 2) < Center(&aTriangulation, 1, 2));
    assert!(Center(&aTriangulation, 1, 2) < Center(&aTriangulation, 2, 2));
}

#[test]
fn BVH_QuickSorterTest__SortRangeInSet() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create 5 triangles with X centroids: 0, 1, 2, 3, 4
    for i in 0..5 {
        let x = f64::from(i);
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 1.0, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.5, 1.0, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    // Reverse the middle 3 elements (indices 1, 2, 3) manually
    aTriangulation.swap(1, 3).unwrap();

    // Now we have centroids: 0, 3, 2, 1, 4
    assert_near(Center(&aTriangulation, 1, 0), 3.5, 1e-10);
    assert_near(Center(&aTriangulation, 2, 0), 2.5, 1e-10);
    assert_near(Center(&aTriangulation, 3, 0), 1.5, 1e-10);

    // Sort only range [1, 3] (middle 3 elements)
    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);
    aSorter.perform_range(&mut aTriangulation, 1, 3).unwrap();

    // Elements 0 and 4 should remain unchanged
    assert_near(Center(&aTriangulation, 0, 0), 0.5, 1e-10);
    assert_near(Center(&aTriangulation, 4, 0), 4.5, 1e-10);

    // Elements 1-3 should be sorted: 1, 2, 3
    assert_near(Center(&aTriangulation, 1, 0), 1.5, 1e-10);
    assert_near(Center(&aTriangulation, 2, 0), 2.5, 1e-10);
    assert_near(Center(&aTriangulation, 3, 0), 3.5, 1e-10);
}

// =============================================================================
// Edge Cases and Stability Tests
// =============================================================================

#[test]
fn BVH_QuickSorterTest__AlreadySorted() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create already sorted triangles
    for i in 0..10 {
        let x = f64::from(i);
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.9, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.5, 0.9, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);
    aSorter.perform(&mut aTriangulation).unwrap();

    // Should remain sorted
    for i in 0..9 {
        assert!(Center(&aTriangulation, i, 0) < Center(&aTriangulation, i + 1, 0));
    }
}

#[test]
fn BVH_QuickSorterTest__ReverseSorted() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create reverse sorted triangles
    for i in 0..10 {
        let x = f64::from(9 - i);
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.9, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.5, 0.9, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);
    aSorter.perform(&mut aTriangulation).unwrap();

    // Should become sorted
    for i in 0..9 {
        assert!(Center(&aTriangulation, i, 0) < Center(&aTriangulation, i + 1, 0));
    }
}

#[test]
fn BVH_QuickSorterTest__DuplicateValues() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create triangles with duplicate centroids: 1, 2, 2, 2, 3
    for i in 0..5 {
        let x: f64 = if i == 0 {
            1.0
        } else if i <= 3 {
            2.0
        } else {
            3.0
        };
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.9, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.5, 0.9, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);
    aSorter.perform(&mut aTriangulation).unwrap();

    // Should be non-decreasing
    for i in 0..4 {
        assert!(Center(&aTriangulation, i, 0) <= Center(&aTriangulation, i + 1, 0));
    }
}

#[test]
fn BVH_QuickSorterTest__AllSameValue() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // All triangles at same position
    for i in 0..5 {
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.9, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.5, 0.9, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);
    aSorter.perform(&mut aTriangulation).unwrap();

    // All should have same centroid
    for i in 0..5 {
        assert_near(Center(&aTriangulation, i, 0), 1.467, 1e-2);
    }
}

#[test]
fn BVH_QuickSorterTest__LargeDataSet() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create 1000 triangles with random-ish order
    for i in 0..1000 {
        // Use simple pseudo-random pattern
        let x = f64::from((i * 37) % 1000);
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.9, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.5, 0.9, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);
    aSorter.perform(&mut aTriangulation).unwrap();

    // Verify sorted
    for i in 0..999 {
        assert!(Center(&aTriangulation, i, 0) <= Center(&aTriangulation, i + 1, 0));
    }
}

#[test]
fn BVH_QuickSorterTest__TwoElements() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Two elements out of order
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(2.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(3.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(2.5, 1.0, 0.0));
    array::append(&mut aTriangulation.elements, BvhVec4i::new(0, 1, 2, 0));

    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.5, 1.0, 0.0));
    array::append(&mut aTriangulation.elements, BvhVec4i::new(3, 4, 5, 0));

    let mut aSorter = BvhQuickSorter::<BvhVec3d>::new(0);
    aSorter.perform(&mut aTriangulation).unwrap();

    assert!(Center(&aTriangulation, 0, 0) < Center(&aTriangulation, 1, 0));
}
