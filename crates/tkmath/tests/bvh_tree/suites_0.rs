#[test]
fn BVH_TreeTest__DefaultConstructor() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    assert_eq!(aTree.length(), 0);
    assert_eq!(aTree.depth(), 0);
}

#[test]
fn BVH_TreeTest__AddLeafNode() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    let mut aMin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMax = BvhVec3d::new(1.0, 1.0, 1.0);

    let mut aNodeIndex = aTree.add_leaf_node(aMin, aMax, 0, 10);

    assert_eq!(aNodeIndex, 0);
    assert_eq!(aTree.length(), 1);
    assert!(aTree.is_outer(0).unwrap());
    assert_eq!(aTree.beg_primitive(0).unwrap(), 0);
    assert_eq!(aTree.end_primitive(0).unwrap(), 10);
    assert_eq!(aTree.nb_primitives(0).unwrap(), 11);
}

#[test]
fn BVH_TreeTest__AddInnerNode() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    // Add two leaf nodes first
    let mut aMin1 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMax1 = BvhVec3d::new(1.0, 1.0, 1.0);
    let mut aLeaf1 = aTree.add_leaf_node(aMin1, aMax1, 0, 5);

    let mut aMin2 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aMax2 = BvhVec3d::new(3.0, 1.0, 1.0);
    let mut aLeaf2 = aTree.add_leaf_node(aMin2, aMax2, 6, 10);

    // Add inner node
    let mut aMinRoot = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMaxRoot = BvhVec3d::new(3.0, 1.0, 1.0);
    let mut aRoot = aTree.add_inner_node(aMinRoot, aMaxRoot, aLeaf1, aLeaf2);

    assert_eq!(aTree.length(), 3);
    assert!(!aTree.is_outer(aRoot).unwrap());
    assert_eq!(aTree.child::<0>(aRoot).unwrap(), aLeaf1);
    assert_eq!(aTree.child::<1>(aRoot).unwrap(), aLeaf2);
}

#[test]
fn BVH_TreeTest__MinMaxPoints() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    let mut aMin = BvhVec3d::new(1.0, 2.0, 3.0);
    let mut aMax = BvhVec3d::new(4.0, 5.0, 6.0);

    aTree.add_leaf_node(aMin, aMax, 0, 0);

    assert_near(aTree.min_point(0).unwrap().x, 1.0, precision::confusion());
    assert_near(aTree.min_point(0).unwrap().y, 2.0, precision::confusion());
    assert_near(aTree.min_point(0).unwrap().z, 3.0, precision::confusion());
    assert_near(aTree.max_point(0).unwrap().x, 4.0, precision::confusion());
    assert_near(aTree.max_point(0).unwrap().y, 5.0, precision::confusion());
    assert_near(aTree.max_point(0).unwrap().z, 6.0, precision::confusion());
}

#[test]
fn BVH_TreeTest__Clear() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 0, 0);
    aTree.add_leaf_node(BvhVec3d::new(1.0, 0.0, 0.0), BvhVec3d::new(2.0, 1.0, 1.0), 1, 1);

    assert_eq!(aTree.length(), 2);

    aTree.clear();

    assert_eq!(aTree.length(), 0);
    assert_eq!(aTree.depth(), 0);
}

#[test]
fn BVH_TreeTest__Reserve() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    // Reserve should not throw
    aTree.reserve(100).unwrap();

    // Can still add nodes after reserve
    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 0, 0);
    assert_eq!(aTree.length(), 1);
}

#[test]
fn BVH_TreeTest__SetOuterInner() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    // Add leaf and change to inner
    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 0, 0);

    assert!(aTree.is_outer(0).unwrap());

    aTree.set_inner(0).unwrap();
    assert!(!aTree.is_outer(0).unwrap());

    aTree.set_outer(0).unwrap();
    assert!(aTree.is_outer(0).unwrap());
}

#[test]
fn BVH_TreeTest__Level() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 0, 0);

    // Default level is 0
    assert_eq!(aTree.level(0).unwrap(), 0);

    // Change level
    *aTree.level_mut(0).unwrap() = 5;
    assert_eq!(aTree.level(0).unwrap(), 5);
}

#[test]
fn BVH_TreeTest__AddLeafNodeWithBox() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let mut aNodeIndex = aTree.add_leaf_node_box(&aBox, 0, 5);

    assert_eq!(aNodeIndex, 0);
    assert!(aTree.is_outer(0).unwrap());
    assert_eq!(aTree.beg_primitive(0).unwrap(), 0);
    assert_eq!(aTree.end_primitive(0).unwrap(), 5);
}

#[test]
fn BVH_TreeTest__AddInnerNodeWithBox() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 0.0, 0.0), BvhVec3d::new(3.0, 1.0, 1.0));
    let mut aRootBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(3.0, 1.0, 1.0));

    let mut aLeaf1 = aTree.add_leaf_node_box(&aBox1, 0, 5);
    let mut aLeaf2 = aTree.add_leaf_node_box(&aBox2, 6, 10);
    let mut aRoot = aTree.add_inner_node_box(&aRootBox, aLeaf1, aLeaf2);

    assert!(!aTree.is_outer(aRoot).unwrap());
    assert_eq!(aTree.child::<0>(aRoot).unwrap(), aLeaf1);
    assert_eq!(aTree.child::<1>(aRoot).unwrap(), aLeaf2);
}

#[test]
fn BVH_TreeTest__EstimateSAH() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    // Create a simple tree with root and two leaves
    let mut aMin1 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMax1 = BvhVec3d::new(1.0, 1.0, 1.0);
    let mut aLeaf1 = aTree.add_leaf_node(aMin1, aMax1, 0, 0);

    let mut aMin2 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aMax2 = BvhVec3d::new(3.0, 1.0, 1.0);
    let mut aLeaf2 = aTree.add_leaf_node(aMin2, aMax2, 1, 1);

    let mut aMinRoot = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMaxRoot = BvhVec3d::new(3.0, 1.0, 1.0);
    aTree.add_inner_node(aMinRoot, aMaxRoot, aLeaf1, aLeaf2);

    let mut aSAH: f64 = aTree.estimate_sah().unwrap();

    // SAH should be positive
    assert!(aSAH > 0.0);
}

#[test]
fn BVH_TreeTest__NbPrimitives() {
    let mut aTree = BvhTree::<BvhVec3d, BvhBinaryTree>::default();

    aTree.add_leaf_node(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0), 5, 15);

    // NbPrimitives = EndPrimitive - BegPrimitive + 1 = 15 - 5 + 1 = 11
    assert_eq!(aTree.nb_primitives(0).unwrap(), 11);
}
