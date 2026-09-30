#[test]
fn NCollection_KDTreeTest__BuildThreePoints() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 0),
        TestPoint2D::new(0, 1),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    assert_eq!(aTree.len(), 3usize);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(0.1, 0.1), None);
    near!((*aTree.point(aIdx).unwrap()).X, 0.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 0.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__BuildSevenPoints() {
    let mut aPoints = [
        TestPoint2D::new(4, 0),
        TestPoint2D::new(2, 0),
        TestPoint2D::new(6, 0),
        TestPoint2D::new(1, 0),
        TestPoint2D::new(3, 0),
        TestPoint2D::new(5, 0),
        TestPoint2D::new(7, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..7 as usize]);
    assert_eq!(aTree.len(), 7usize);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(4.9, 0), None);
    near!((*aTree.point(aIdx).unwrap()).X, 5.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__BuildFifteenPoints() {
    let mut aPoints = vec![TestPoint2D::default(); 15 as usize];
    for i in (0)..(15) {
        aPoints[(i) as usize] = TestPoint2D::new(((i) as f64), ((i % 3) as f64));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..15 as usize]);
    assert_eq!(aTree.len(), 15usize);
}

#[test]
fn NCollection_KDTreeTest__BuildSixPoints() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(2, 2),
        TestPoint2D::new(3, 3),
        TestPoint2D::new(4, 4),
        TestPoint2D::new(5, 5),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..6 as usize]);
    assert_eq!(aTree.len(), 6usize);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(2.9, 2.9), None);
    near!((*aTree.point(aIdx).unwrap()).X, 3.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__ClearAndRebuild() {
    let mut aPoints1 = [TestPoint2D::new(0, 0), TestPoint2D::new(1, 1)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints1)[..2 as usize]);
    aTree.clear();
    assert!(aTree.is_empty());
    let mut aPoints2 = [
        TestPoint2D::new(5, 5),
        TestPoint2D::new(6, 6),
        TestPoint2D::new(7, 7),
    ];
    aTree.build(&(&aPoints2)[..3 as usize]);
    assert_eq!(aTree.len(), 3usize);
    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(5.1, 5.1), Some(&mut aSqDist));
    near!((*aTree.point(aIdx).unwrap()).X, 5.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__CopyIndependence() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(2, 2),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aCopy: KDTree<TestPoint2D, 2> = aTree.clone();

    let mut aPoints2 = [TestPoint2D::new(10, 10)];
    aTree.build(&(&aPoints2)[..1 as usize]);

    assert_eq!(aCopy.len(), 3usize);
    let mut aIdx: usize = aCopy.nearest_point(&TestPoint2D::new(1.1, 1.1), None);
    near!((*aCopy.point(aIdx).unwrap()).X, 1.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__MoveFromEmptyTree() {
    let mut anEmpty: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aMoved: KDTree<TestPoint2D, 2> = std::mem::take(&mut anEmpty);
    assert!(aMoved.is_empty());
    assert!(anEmpty.is_empty());
}

#[test]
fn NCollection_KDTreeTest__CopyEmptyTree() {
    let mut anEmpty: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aCopy: KDTree<TestPoint2D, 2> = anEmpty.clone();
    assert!(aCopy.is_empty());
}

#[test]
fn NCollection_KDTreeTest__MoveAssignmentFromEmpty() {
    let mut aPoints = [TestPoint2D::new(1, 1), TestPoint2D::new(2, 2)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut anEmpty: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree = std::mem::take(&mut anEmpty);
    assert!(aTree.is_empty());
}

#[test]
fn NCollection_KDTreeTest__CopyAssignmentOverwrite() {
    let mut aPoints1 = [TestPoint2D::new(0, 0)];
    let mut aPoints2 = [
        TestPoint2D::new(10, 10),
        TestPoint2D::new(20, 20),
        TestPoint2D::new(30, 30),
    ];
    let mut aTree1: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree1.build(&(&aPoints1)[..1 as usize]);
    let mut aTree2: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree2.build(&(&aPoints2)[..3 as usize]);
    aTree1 = aTree2.clone();
    assert_eq!(aTree1.len(), 3usize);
    let mut aIdx: usize = aTree1.nearest_point(&TestPoint2D::new(10, 10), None);
    near!((*aTree1.point(aIdx).unwrap()).X, 10.0, 1e-15);
}
