#[test]
fn NCollection_KDTreeRadiiTest__CopyAssignment() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(5.0, 0.0)];
    let mut aRadii = [1.0, 2.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);
    let mut aCopy: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aCopy = aTree.clone();
    assert_eq!(aCopy.len(), 2usize);
    assert_eq!(aTree.len(), 2usize);
}

#[test]
fn NCollection_KDTreeRadiiTest__MoveAssignment() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(5.0, 0.0)];
    let mut aRadii = [1.0, 2.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);
    let mut aMoved: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aMoved = std::mem::take(&mut aTree);
    assert_eq!(aMoved.len(), 2usize);
    assert!(aTree.is_empty());
}

#[test]
fn NCollection_KDTreeRadiiTest__NearestPointStillWorks() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(10.0, 0.0),
        TestPoint2D::new(5.0, 5.0),
    ];
    let mut aRadii = [1.0, 1.0, 1.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    let mut aQuery = TestPoint2D::new(4.5, 4.5);
    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&aQuery, Some(&mut aSqDist));
    near!((*aTree.point(aIdx).unwrap()).X, 5.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 5.0, 1e-15);
    near!(aSqDist, 0.5, 1e-10);
}

#[test]
fn NCollection_KDTreeRadiiTest__RangeSearchStillWorks() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(1.0, 0.0),
        TestPoint2D::new(10.0, 10.0),
    ];
    let mut aRadii = [0.5, 0.5, 0.5];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    let aResult = aTree.range_search(&TestPoint2D::new(0.5, 0.0), 1.0);
    assert_eq!(aResult.len(), 2);
}

#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_EmptyTree() {
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    let aResult = aTree.containing_search(&TestPoint2D::new(0.0, 0.0));
    assert_eq!(aResult.len(), 0);
}

#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_QueryInsideSphere() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0)];
    let mut aRadii = [5.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..1 as usize], &aRadii[..1 as usize]);
    let aResult = aTree.containing_search(&TestPoint2D::new(1.0, 1.0));
    assert_eq!(aResult.len(), 1);
}

#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_QueryOutsideSphere() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0)];
    let mut aRadii = [1.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..1 as usize], &aRadii[..1 as usize]);
    let aResult = aTree.containing_search(&TestPoint2D::new(5.0, 5.0));
    assert_eq!(aResult.len(), 0);
}

#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_QueryOnBoundary() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0)];
    let mut aRadii = [5.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..1 as usize], &aRadii[..1 as usize]);
    let aResult = aTree.containing_search(&TestPoint2D::new(3.0, 4.0));
    assert_eq!(aResult.len(), 1);
}

#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_MultipleSpheres() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(2.0, 0.0),
        TestPoint2D::new(10.0, 10.0),
    ];
    let mut aRadii = [3.0, 2.0, 1.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    let aResult = aTree.containing_search(&TestPoint2D::new(1.0, 0.0));
    assert_eq!(aResult.len(), 2);

    let mut aFoundIndices: TestSet<usize> = TestSet::default();
    for i in (0)..(aResult.len()) {
        aFoundIndices.insert(aResult[(i) as usize]);
    }

    assert_eq!(aFoundIndices.len(), 2usize);
}

#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_VaryingRadii() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(50.0, 0.0),
        TestPoint2D::new(-50.0, 0.0),
    ];
    let mut aRadii = [100.0, 0.1, 0.1];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    let aResult = aTree.containing_search(&TestPoint2D::new(40.0, 0.0));

    assert_eq!(aResult.len(), 1);
    near!((*aTree.point(aResult[(0) as usize]).unwrap()).X, 0.0, 1e-15);
}
