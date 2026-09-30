#[test]
fn NCollection_KDTreeTest__DefaultConstructor() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    assert!(aTree.is_empty());
    assert_eq!(aTree.len(), 0usize);
}

#[test]
fn NCollection_KDTreeTest__BuildFromCArray() {
    let mut aPoints = [
        TestPoint2D::new(1.0, 2.0),
        TestPoint2D::new(3.0, 4.0),
        TestPoint2D::new(5.0, 6.0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    assert!(!aTree.is_empty());
    assert_eq!(aTree.len(), 3usize);
}

#[test]
fn NCollection_KDTreeTest__BuildFromArray1() {
    let mut aPoints: Array1<TestPoint2D> = Array1::with_bounds(1, 3).unwrap();
    aPoints.set_value(1, TestPoint2D::new(1.0, 2.0)).unwrap();
    aPoints.set_value(2, TestPoint2D::new(3.0, 4.0)).unwrap();
    aPoints.set_value(3, TestPoint2D::new(5.0, 6.0)).unwrap();
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&aPoints.as_slice());
    assert!(!aTree.is_empty());
    assert_eq!(aTree.len(), 3usize);
}

#[test]
fn NCollection_KDTreeTest__BuildSinglePoint() {
    let mut aPoint = TestPoint2D::new(7.0, 8.0);
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(std::slice::from_ref(&aPoint))[..1 as usize]);
    assert_eq!(aTree.len(), 1usize);
    near!((*aTree.point(1).unwrap()).X, 7.0, 1e-15);
    near!((*aTree.point(1).unwrap()).Y, 8.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__BuildTwoPoints() {
    let mut aPoints = [TestPoint2D::new(1.0, 0.0), TestPoint2D::new(0.0, 1.0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    assert_eq!(aTree.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__Clear() {
    let mut aPoints = [TestPoint2D::new(1.0, 2.0), TestPoint2D::new(3.0, 4.0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    assert!(!aTree.is_empty());
    aTree.clear();
    assert!(aTree.is_empty());
    assert_eq!(aTree.len(), 0usize);
}

#[test]
fn NCollection_KDTreeTest__CopyConstructor() {
    let mut aPoints = [
        TestPoint2D::new(1.0, 2.0),
        TestPoint2D::new(3.0, 4.0),
        TestPoint2D::new(5.0, 6.0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aCopy: KDTree<TestPoint2D, 2> = aTree.clone();
    assert_eq!(aCopy.len(), 3usize);

    let mut aQuery = TestPoint2D::new(3.1, 4.1);
    assert_eq!(
        aTree.nearest_point(&aQuery, None),
        aCopy.nearest_point(&aQuery, None)
    );
}

#[test]
fn NCollection_KDTreeTest__MoveConstructor() {
    let mut aPoints = [TestPoint2D::new(1.0, 2.0), TestPoint2D::new(3.0, 4.0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aMoved: KDTree<TestPoint2D, 2> = std::mem::take(&mut aTree);
    assert_eq!(aMoved.len(), 2usize);
    assert!(aTree.is_empty());
}

#[test]
fn NCollection_KDTreeTest__CopyAssignment() {
    let mut aPoints = [TestPoint2D::new(1.0, 2.0), TestPoint2D::new(3.0, 4.0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aCopy: KDTree<TestPoint2D, 2> = KDTree::default();
    aCopy = aTree.clone();
    assert_eq!(aCopy.len(), 2usize);
    assert_eq!(aTree.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__MoveAssignment() {
    let mut aPoints = [TestPoint2D::new(1.0, 2.0), TestPoint2D::new(3.0, 4.0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aMoved: KDTree<TestPoint2D, 2> = KDTree::default();
    aMoved = std::mem::take(&mut aTree);
    assert_eq!(aMoved.len(), 2usize);
    assert!(aTree.is_empty());
}
