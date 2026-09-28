// =============================================================================
// BVH_SweepPlaneBuilder Construction Tests
// =============================================================================

#[test]
fn BVH_SweepPlaneBuilderTest__DefaultConstructor() {
    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::default();
    assert_eq!(
        aBuilder.builder_transient().leaf_node_size(),
        LEAF_NODE_SIZE_DEFAULT
    );
    assert_eq!(aBuilder.builder_transient().max_tree_depth(), MAX_TREE_DEPTH);
}

#[test]
fn BVH_SweepPlaneBuilderTest__CustomParameters() {
    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::new(5, 20, 1);
    assert_eq!(aBuilder.builder_transient().leaf_node_size(), 5);
    assert_eq!(aBuilder.builder_transient().max_tree_depth(), 20);
}

// =============================================================================
// BVH Building Tests
// =============================================================================

#[test]
fn BVH_SweepPlaneBuilderTest__BuildEmptyTriangulation() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();
    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::default();

    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBox = BvhBox::<BvhVec3d>::default();

    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    assert_eq!(aBVH.length(), 0);
}

#[test]
fn BVH_SweepPlaneBuilderTest__BuildSingleTriangle() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Add vertices
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 1.0, 0.0));

    // Add triangle
    array::append(&mut aTriangulation.elements, BvhVec4i::new(0, 1, 2, 0));

    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::new(1, 32, 1);
    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBox = ComputeSetBox(&aTriangulation);

    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    // Single triangle should create one leaf node
    assert_eq!(aBVH.length(), 1);
    assert!(aBVH.is_outer(0).unwrap());
    assert_eq!(aBVH.beg_primitive(0).unwrap(), 0);
    assert_eq!(aBVH.end_primitive(0).unwrap(), 0);
}

#[test]
fn BVH_SweepPlaneBuilderTest__BuildTwoTriangles() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create two triangles forming a quad
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, 0.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(1.0, 1.0, 0.0));
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 1.0, 0.0));

    array::append(&mut aTriangulation.elements, BvhVec4i::new(0, 1, 2, 0));
    array::append(&mut aTriangulation.elements, BvhVec4i::new(0, 2, 3, 0));

    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::new(1, 32, 1);
    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBox = ComputeSetBox(&aTriangulation);

    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    // Two triangles with leaf size 1 should create internal nodes
    assert!(aBVH.length() > 2);
    assert!(aBVH.depth() >= 1);
}

#[test]
fn BVH_SweepPlaneBuilderTest__BuildMultipleTriangles_Grid() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create a 2x2 grid of quads (8 triangles)
    let aGridSize = 3;
    for i in 0..aGridSize {
        for j in 0..aGridSize {
            array::append(
                &mut aTriangulation.vertices,
                BvhVec3d::new(i as f64, j as f64, 0.0),
            );
        }
    }

    // Add triangles for the grid
    for i in 0..aGridSize - 1 {
        for j in 0..aGridSize - 1 {
            let aV0 = i * aGridSize + j;
            let aV1 = i * aGridSize + (j + 1);
            let aV2 = (i + 1) * aGridSize + (j + 1);
            let aV3 = (i + 1) * aGridSize + j;

            array::append(&mut aTriangulation.elements, BvhVec4i::new(aV0, aV1, aV2, 0));
            array::append(&mut aTriangulation.elements, BvhVec4i::new(aV0, aV2, aV3, 0));
        }
    }

    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::new(2, 32, 1);
    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBox = ComputeSetBox(&aTriangulation);

    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    let aTotalTriangles = (aGridSize - 1) * (aGridSize - 1) * 2;
    assert!(aBVH.length() > 0);
    assert!(aBVH.depth() >= 1);

    // Verify tree bounds contain all triangles
    let aRootBox = BvhBox::from_corners(*aBVH.min_point(0).unwrap(), *aBVH.max_point(0).unwrap());
    for i in 0..aTotalTriangles {
        let aTriBox = aTriangulation.element_box(i).unwrap();
        assert!(!aRootBox.is_out(&aTriBox));
    }
}

#[test]
fn BVH_SweepPlaneBuilderTest__SplitAlongDifferentAxes() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create triangles spread along X axis (should prefer X-axis split)
    for i in 0..10 {
        let x = i as f64 * 2.0;
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 1.0, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.5, 1.0, 0.0));

        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::new(1, 32, 1);
    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBox = ComputeSetBox(&aTriangulation);

    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    // Should create splits along X axis
    assert!(aBVH.length() > 1);
    assert!(aBVH.depth() >= 1);
}

#[test]
fn BVH_SweepPlaneBuilderTest__DegenerateCase_AllSamePosition() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // All triangles at same position (degenerate case)
    for i in 0..5 {
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.1, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(0.0, 0.1, 0.0));

        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::new(1, 32, 1);
    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBox = ComputeSetBox(&aTriangulation);

    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    // Should still build valid tree even with degenerate geometry
    assert!(aBVH.length() > 0);
}

// =============================================================================
// SAH Quality Tests
// =============================================================================

#[test]
fn BVH_SweepPlaneBuilderTest__SAHQuality_VerifyBetter() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create well-separated triangle groups (should result in good SAH)
    // Group 1: left side
    for i in 0..5 {
        let x = i as f64 * 0.5;
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.4, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.2, 0.4, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    // Group 2: right side (spatially separated)
    for i in 0..5 {
        let x = 10.0 + i as f64 * 0.5;
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.4, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.2, 0.4, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new((5 + i) * 3, (5 + i) * 3 + 1, (5 + i) * 3 + 2, 0),
        );
    }

    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::new(2, 32, 1);
    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBox = ComputeSetBox(&aTriangulation);

    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    // Spatially separated groups should create clear split
    assert!(aBVH.depth() >= 1);

    // Root should have two children covering separate regions
    if !aBVH.is_outer(0).unwrap() {
        let aLeftChild = aBVH.child::<0>(0).unwrap();
        let aRightChild = aBVH.child::<1>(0).unwrap();

        let aLeftBox = BvhBox::from_corners(
            *aBVH.min_point(aLeftChild).unwrap(),
            *aBVH.max_point(aLeftChild).unwrap(),
        );
        let aRightBox = BvhBox::from_corners(
            *aBVH.min_point(aRightChild).unwrap(),
            *aBVH.max_point(aRightChild).unwrap(),
        );

        // Boxes should not significantly overlap for well-separated geometry
        let mut aOverlap = 0.0;
        for i in 0..3 {
            let aMin = std_max(aLeftBox.corner_min()[i], aRightBox.corner_min()[i]);
            let aMax = std_min(aLeftBox.corner_max()[i], aRightBox.corner_max()[i]);
            if aMax > aMin {
                aOverlap += aMax - aMin;
            }
        }

        // Overlap should be minimal for well-separated groups
        let aTotalSize = aBox.corner_max()[0] - aBox.corner_min()[0];
        assert!(aOverlap < aTotalSize * 0.1); // Less than 10% overlap
    }
}

// =============================================================================
// Leaf Size Tests
// =============================================================================

#[test]
fn BVH_SweepPlaneBuilderTest__LeafSize_RespectMaxSize() {
    let mut aTriangulation = BvhTriangulation::<BvhVec3d>::default();

    // Create 20 triangles
    for i in 0..20 {
        let x = i as f64;
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.9, 0.0, 0.0));
        array::append(&mut aTriangulation.vertices, BvhVec3d::new(x + 0.5, 0.9, 0.0));
        array::append(
            &mut aTriangulation.elements,
            BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
        );
    }

    let aBuilder = BvhSweepPlaneBuilder::<BvhVec3d>::new(5, 32, 1); // Leaf size = 5
    let mut aBVH = BvhTree::<BvhVec3d>::default();
    let aBox = ComputeSetBox(&aTriangulation);

    aBuilder
        .build(&mut aTriangulation, &mut aBVH, &aBox)
        .unwrap();

    // Verify leaf nodes don't exceed the specified size
    for i in 0..aBVH.length() {
        if aBVH.is_outer(i).unwrap() {
            let aLeafSize = aBVH.end_primitive(i).unwrap() - aBVH.beg_primitive(i).unwrap() + 1;
            assert!(aLeafSize <= 5);
        }
    }
}
