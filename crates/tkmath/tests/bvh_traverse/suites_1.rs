// =======================================================================================
// Tests for BVH_PairTraverse
// =======================================================================================

#[test]
fn BVH_PairTraverseTest__IsOutVerification() {
    // Verify that IsOut works correctly for non-overlapping boxes
    let aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(3.0, 1.0, 0.0));
    let aBox2 = BvhBox::from_corners(
        BvhVec3d::new(100.0, 0.0, 0.0),
        BvhVec3d::new(103.0, 1.0, 0.0),
    );

    // These boxes are far apart and should not overlap
    assert!(aBox1.is_out_corners(aBox2.corner_min(), aBox2.corner_max()));
    assert!(aBox2.is_out_corners(aBox1.corner_min(), aBox1.corner_max()));
}

#[test]
fn BVH_PairTraverseTest__TriangulationBoxVerification() {
    // Create two triangulations and verify their bounding boxes don't overlap
    let (mut aTri1, mut aTri2) = SeparatedTriangulations();

    // Mark as dirty to force bounding box computation
    aTri1.mark_dirty();
    aTri2.mark_dirty();

    let aBox1 = aTri1.bounding_box().unwrap();
    let aBox2 = aTri2.bounding_box().unwrap();

    // Verify the boxes are where we expect them
    assert!(aBox1.is_valid());
    assert!(aBox2.is_valid());

    // Box1 should be roughly [0, 0, 0] to [3, 1, 0]
    assert_near(aBox1.corner_min().x, 0.0, 0.01);
    assert_near(aBox1.corner_max().x, 3.0, 0.01);

    // Box2 should be roughly [100, 0, 0] to [103, 1, 0]
    assert_near(aBox2.corner_min().x, 100.0, 0.01);
    assert_near(aBox2.corner_max().x, 103.0, 0.01);

    // The boxes should not overlap
    assert!(aBox1.is_out(&aBox2));
    assert!(aBox2.is_out(&aBox1));
}

#[test]
fn BVH_PairTraverseTest__BVHRootBoxVerification() {
    // Create triangulations and verify their BVH root boxes
    let (mut aTri1, mut aTri2) = SeparatedTriangulations();

    aTri1.mark_dirty();
    aTri2.mark_dirty();

    let mut aBVH1 = BvhTree::<BvhVec3d>::default();
    let mut aBVH2 = BvhTree::<BvhVec3d>::default();

    let aBuilder = BvhBinnedBuilder::<BvhVec3d>::default();
    let aBox1 = aTri1.bounding_box().unwrap();
    aBuilder.build(&mut aTri1, &mut aBVH1, &aBox1).unwrap();
    let aBox2 = aTri2.bounding_box().unwrap();
    aBuilder.build(&mut aTri2, &mut aBVH2, &aBox2).unwrap();

    // Check the root node bounding boxes
    let aMin1 = *aBVH1.min_point(0).unwrap();
    let aMax1 = *aBVH1.max_point(0).unwrap();
    let aMin2 = *aBVH2.min_point(0).unwrap();
    let aMax2 = *aBVH2.max_point(0).unwrap();

    // Verify root boxes are separated
    assert_near(aMin1.x, 0.0, 0.01);
    assert_near(aMax1.x, 3.0, 0.01);
    assert_near(aMin2.x, 100.0, 0.01);
    assert_near(aMax2.x, 103.0, 0.01);

    // The root boxes should not overlap
    let aBox1 = BvhBox::from_corners(aMin1, aMax1);
    assert!(aBox1.is_out_corners(&aMin2, &aMax2));

    // Check if the nodes are inner or leaf nodes
    let aData1 = *array::value(aBVH1.node_info_buffer(), 0).unwrap();
    let aData2 = *array::value(aBVH2.node_info_buffer(), 0).unwrap();

    // aData.x() == 0 means inner node, != 0 means leaf node
    // With 3 triangles, the tree might be a single leaf node
    assert_eq!(
        aData1.x, 1,
        "Root of BVH1 should be a leaf node (3 elements)"
    );
    assert_eq!(
        aData2.x, 1,
        "Root of BVH2 should be a leaf node (3 elements)"
    );
}

#[test]
fn BVH_PairTraverseTest__CountAllPairs() {
    let aBVH1 = CreateSimpleTriangulationBVH(5);
    let aBVH2 = CreateSimpleTriangulationBVH(5);

    let mut aSelector = BVH_CountAllPairs::default();
    let aCount = aSelector.select_trees(Some(&aBVH1), Some(&aBVH2)).unwrap();

    assert_eq!(aCount, 25); // 5 x 5 pairs
}

#[test]
fn BVH_PairTraverseTest__EmptyFirstTree() {
    let aBVH1 = BvhTree::<BvhVec3d>::default();
    let aBVH2 = CreateSimpleTriangulationBVH(5);

    let mut aSelector = BVH_CountAllPairs::default();
    let aCount = aSelector.select_trees(Some(&aBVH1), Some(&aBVH2)).unwrap();

    assert_eq!(aCount, 0);
}

#[test]
fn BVH_PairTraverseTest__EmptySecondTree() {
    let aBVH1 = CreateSimpleTriangulationBVH(5);
    let aBVH2 = BvhTree::<BvhVec3d>::default();

    let mut aSelector = BVH_CountAllPairs::default();
    let aCount = aSelector.select_trees(Some(&aBVH1), Some(&aBVH2)).unwrap();

    assert_eq!(aCount, 0);
}

#[test]
fn BVH_PairTraverseTest__NullTrees() {
    // Two null handles.
    let aBVH1: Option<&BvhTree<BvhVec3d>> = None;
    let aBVH2: Option<&BvhTree<BvhVec3d>> = None;

    let mut aSelector = BVH_CountAllPairs::default();
    let aCount = aSelector.select_trees(aBVH1, aBVH2).unwrap();

    assert_eq!(aCount, 0);
}

#[test]
fn BVH_PairTraverseTest__OverlapDetection_SameTrees() {
    let aBVH = CreateSimpleTriangulationBVH(10);

    let mut aSelector = BVH_OverlapDetector::default();
    let aCount = aSelector.select_trees(Some(&aBVH), Some(&aBVH)).unwrap();

    // Self-overlap: all 10 elements overlap with themselves
    assert!(aCount >= 10);
}

#[test]
fn BVH_PairTraverseTest__OverlapDetection_NonOverlapping() {
    // Create two triangulations in different regions
    // First triangulation at x=0..5, second at x=100..105 (far away)
    let (mut aTri1, mut aTri2) = SeparatedTriangulations();

    // Mark as dirty to force bounding box computation
    aTri1.mark_dirty();
    aTri2.mark_dirty();

    let mut aBVH1 = BvhTree::<BvhVec3d>::default();
    let mut aBVH2 = BvhTree::<BvhVec3d>::default();

    let aBuilder = BvhBinnedBuilder::<BvhVec3d>::default();
    let aBox1 = aTri1.bounding_box().unwrap();
    aBuilder.build(&mut aTri1, &mut aBVH1, &aBox1).unwrap();
    let aBox2 = aTri2.bounding_box().unwrap();
    aBuilder.build(&mut aTri2, &mut aBVH2, &aBox2).unwrap();

    let mut aSelector = BVH_OverlapDetector::default();
    let aCount = aSelector.select_trees(Some(&aBVH1), Some(&aBVH2)).unwrap();

    // Debug: Check how many times RejectNode was called
    // If it's 0, RejectNode is not being called at all
    // If it's > 0 but overlaps are still found, then IsOut is broken
    assert!(
        aSelector.RejectCount() > 0,
        "RejectNode should be called at least once"
    );

    // No overlaps expected
    assert_eq!(
        aCount,
        0,
        "Found {} overlaps (RejectNode called {} times)",
        aCount,
        aSelector.RejectCount()
    );
}

#[test]
fn BVH_PairTraverseTest__AsymmetricPairs() {
    let aBVH1 = CreateSimpleTriangulationBVH(3);
    let aBVH2 = CreateSimpleTriangulationBVH(7);

    let mut aSelector = BVH_CountAllPairs::default();
    let aCount = aSelector.select_trees(Some(&aBVH1), Some(&aBVH2)).unwrap();

    assert_eq!(aCount, 21); // 3 x 7 pairs
}

#[test]
fn BVH_PairTraverseTest__LargeDataSets() {
    let aBVH1 = CreateSimpleTriangulationBVH(50);
    let aBVH2 = CreateSimpleTriangulationBVH(50);

    let mut aSelector = BVH_CountAllPairs::default();
    let aCount = aSelector.select_trees(Some(&aBVH1), Some(&aBVH2)).unwrap();

    assert_eq!(aCount, 2500); // 50 x 50 pairs
}
