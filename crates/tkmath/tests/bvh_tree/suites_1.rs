#[test]
fn BVH_TreeTest__SinglePrimitive() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 7, 7);

    assert_eq!(aTree.nb_primitives(0).unwrap(), 1);
    assert_eq!(aTree.beg_primitive(0).unwrap(), 7);
    assert_eq!(aTree.end_primitive(0).unwrap(), 7);
}

#[test]
fn BVH_TreeTest__Float3DTree() {
    let mut aTree = BvhTree::<BvhVec3f, BvhBinaryTree>::default();

    let mut aMin = BvhVec3f::new(0.0, 0.0, 0.0);
    let mut aMax = BvhVec3f::new(1.0, 1.0, 1.0);

    aTree.add_leaf_node(aMin, aMax, 0, 5);

    assert_eq!(aTree.length(), 1);
    assert_near_f32(aTree.min_point(0).unwrap().x, 0.0, 1e-5);
    assert_near_f32(aTree.max_point(0).unwrap().x, 1.0, 1e-5);
}

#[test]
fn BVH_TreeTest__Tree2D() {
    let mut aTree = BvhTree::<BvhVec2d, BvhBinaryTree>::default();

    let mut aMin = BvhVec2d::new(0.0, 0.0);
    let mut aMax = BvhVec2d::new(1.0, 1.0);

    aTree.add_leaf_node(aMin, aMax, 0, 5);

    assert_eq!(aTree.length(), 1);
    assert_near(aTree.min_point(0).unwrap().x, 0.0, precision::confusion());
    assert_near(aTree.min_point(0).unwrap().y, 0.0, precision::confusion());
}

#[test]
fn BVH_TreeTest__Tree4D() {
    let mut aTree = BvhTree::<BvhVec4d, BvhBinaryTree>::default();

    let mut aMin = BvhVec4d::new(0.0, 0.0, 0.0, 0.0);
    let mut aMax = BvhVec4d::new(1.0, 1.0, 1.0, 1.0);

    aTree.add_leaf_node(aMin, aMax, 0, 5);

    assert_eq!(aTree.length(), 1);
    assert_near(aTree.min_point(0).unwrap().x, 0.0, precision::confusion());
    assert_near(aTree.min_point(0).unwrap().w, 0.0, precision::confusion());
}

#[test]
fn BVH_TreeTest__MultipleLeaves() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    for i in 0..10 {
        let mut aMin = BvhVec3d::new(f64::from(i) * 1.0, 0.0, 0.0);
        let mut aMax = BvhVec3d::new(f64::from(i) * 1.0 + 1.0, 1.0, 1.0);
        aTree.add_leaf_node(aMin, aMax, i, i);
    }

    assert_eq!(aTree.length(), 10);

    for i in 0..10 {
        assert!(aTree.is_outer(i).unwrap());
        assert_eq!(aTree.beg_primitive(i).unwrap(), i);
        assert_eq!(aTree.end_primitive(i).unwrap(), i);
    }
}

#[test]
fn BVH_TreeTest__DeepTree() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    // Create leaves
    let mut aMin1 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMax1 = BvhVec3d::new(1.0, 1.0, 1.0);
    let mut aLeaf1 = aTree.add_leaf_node(aMin1, aMax1, 0, 0);

    let mut aMin2 = BvhVec3d::new(1.0, 0.0, 0.0);
    let mut aMax2 = BvhVec3d::new(2.0, 1.0, 1.0);
    let mut aLeaf2 = aTree.add_leaf_node(aMin2, aMax2, 1, 1);

    let mut aMin3 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aMax3 = BvhVec3d::new(3.0, 1.0, 1.0);
    let mut aLeaf3 = aTree.add_leaf_node(aMin3, aMax3, 2, 2);

    let mut aMin4 = BvhVec3d::new(3.0, 0.0, 0.0);
    let mut aMax4 = BvhVec3d::new(4.0, 1.0, 1.0);
    let mut aLeaf4 = aTree.add_leaf_node(aMin4, aMax4, 3, 3);

    // Create intermediate nodes
    let mut aMinI1 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMaxI1 = BvhVec3d::new(2.0, 1.0, 1.0);
    let mut aInner1 = aTree.add_inner_node(aMinI1, aMaxI1, aLeaf1, aLeaf2);

    let mut aMinI2 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aMaxI2 = BvhVec3d::new(4.0, 1.0, 1.0);
    let mut aInner2 = aTree.add_inner_node(aMinI2, aMaxI2, aLeaf3, aLeaf4);

    // Create root
    let mut aMinRoot = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMaxRoot = BvhVec3d::new(4.0, 1.0, 1.0);
    let mut aRoot = aTree.add_inner_node(aMinRoot, aMaxRoot, aInner1, aInner2);

    assert_eq!(aTree.length(), 7);
    assert!(!aTree.is_outer(aRoot).unwrap());
    assert!(!aTree.is_outer(aInner1).unwrap());
    assert!(!aTree.is_outer(aInner2).unwrap());
    assert!(aTree.is_outer(aLeaf1).unwrap());
    assert!(aTree.is_outer(aLeaf4).unwrap());
}

#[test]
fn BVH_TreeTest__ModifyPrimitiveIndices() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 0, 5);

    // Modify primitive indices
    *aTree.beg_primitive_mut(0).unwrap() = 10;
    *aTree.end_primitive_mut(0).unwrap() = 20;

    assert_eq!(aTree.beg_primitive(0).unwrap(), 10);
    assert_eq!(aTree.end_primitive(0).unwrap(), 20);
    assert_eq!(aTree.nb_primitives(0).unwrap(), 11);
}

#[test]
fn BVH_TreeTest__ModifyMinMaxPoints() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 0, 0);

    // Modify min/max points
    *aTree.min_point_mut(0).unwrap() = BvhVec3d::new(-1.0, -1.0, -1.0);
    *aTree.max_point_mut(0).unwrap() = BvhVec3d::new(2.0, 2.0, 2.0);

    assert_near(aTree.min_point(0).unwrap().x, -1.0, precision::confusion());
    assert_near(aTree.max_point(0).unwrap().x, 2.0, precision::confusion());
}

#[test]
fn BVH_TreeTest__ChangeChild() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    let mut aLeaf1 =
        aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 0, 0);
    let mut aLeaf2 =
        aTree.add_leaf_node(BvhVec3d::new(1.0, 0.0, 0.0), BvhVec3d::new(2.0, 1.0, 1.0), 1, 1);
    let mut aLeaf3 =
        aTree.add_leaf_node(BvhVec3d::new(2.0, 0.0, 0.0), BvhVec3d::new(3.0, 1.0, 1.0), 2, 2);
    let mut aRoot = aTree.add_inner_node(
        BvhVec3d::new(0.0, 0.0, 0.0),
        BvhVec3d::new(2.0, 1.0, 1.0),
        aLeaf1,
        aLeaf2,
    );

    assert_eq!(aTree.child::<0>(aRoot).unwrap(), aLeaf1);
    assert_eq!(aTree.child::<1>(aRoot).unwrap(), aLeaf2);

    // Change children
    *aTree.child_mut::<0>(aRoot).unwrap() = aLeaf3;

    assert_eq!(aTree.child::<0>(aRoot).unwrap(), aLeaf3);
    assert_eq!(aTree.child::<1>(aRoot).unwrap(), aLeaf2);
}

#[test]
fn BVH_TreeTest__NodeInfoBuffer() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 5, 10);

    // Access node info buffer
    let aBuffer = aTree.node_info_buffer();
    let mut aSize = array::size(aBuffer);
    assert_eq!(aSize, 1);
}

#[test]
fn BVH_TreeTest__MinMaxPointBuffers() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 0, 0);

    // Access point buffers
    let aMinBuffer = aTree.min_point_buffer();
    let aMaxBuffer = aTree.max_point_buffer();

    let mut aMinSize = array::size(aMinBuffer);
    let mut aMaxSize = array::size(aMaxBuffer);
    assert_eq!(aMinSize, 1);
    assert_eq!(aMaxSize, 1);
}
