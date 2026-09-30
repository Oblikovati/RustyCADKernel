#[test]
fn NCollection_KDTreeTest__KNearest3D_Basic() {
    let mut aPoints = [
        TestPoint3D::new(0, 0, 0),
        TestPoint3D::new(1, 0, 0),
        TestPoint3D::new(0, 1, 0),
        TestPoint3D::new(0, 0, 1),
        TestPoint3D::new(10, 10, 10),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint3D::new(0.1, 0.1, 0.1);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 3 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 3usize);
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_SinglePoint() {
    let mut aPoint = TestPoint2D::new(5.0, 5.0);
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(std::slice::from_ref(&aPoint))[..1 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 5 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 1usize);
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_EmptyTree() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 5 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 0usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_EmptyResult() {
    let mut aPoints = [TestPoint2D::new(10, 10), TestPoint2D::new(20, 20)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aQuery = TestPoint2D::new(0, 0);
    let aResult = aTree.range_search(&aQuery, 1.0);
    assert_eq!(aResult.len(), 0usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_AllInRange() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(0.1, 0.1),
        TestPoint2D::new(-0.1, -0.1),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aQuery = TestPoint2D::new(0, 0);
    let aResult = aTree.range_search(&aQuery, 1.0);
    assert_eq!(aResult.len(), 3usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_SomeInRange() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(0.5, 0),
        TestPoint2D::new(2.0, 0),
        TestPoint2D::new(5.0, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aQuery = TestPoint2D::new(0, 0);
    let aResult = aTree.range_search(&aQuery, 1.0);
    assert_eq!(aResult.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_ExactRadius() {
    let mut aPoints = [TestPoint2D::new(3.0, 4.0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..1 as usize]);
    let mut aQuery = TestPoint2D::new(0, 0);
    let aResult = aTree.range_search(&aQuery, 5.0);
    assert_eq!(aResult.len(), 1usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_ZeroRadius() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(0, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aQuery = TestPoint2D::new(0, 0);
    let aResult = aTree.range_search(&aQuery, 0.0);
    assert_eq!(aResult.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch3D_Sphere() {
    let mut aPoints = [
        TestPoint3D::new(0, 0, 0),
        TestPoint3D::new(1, 0, 0),
        TestPoint3D::new(0, 1, 0),
        TestPoint3D::new(0, 0, 1),
        TestPoint3D::new(2, 2, 2),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut aQuery = TestPoint3D::new(0, 0, 0);
    let aResult = aTree.range_search(&aQuery, 1.0);
    assert_eq!(aResult.len(), 4usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_EmptyTree() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aQuery = TestPoint2D::new(0, 0);
    let aResult = aTree.range_search(&aQuery, 10.0);
    assert_eq!(aResult.len(), 0usize);
}
