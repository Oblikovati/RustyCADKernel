#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_AllContain() {
    let mut aPoints = [
        TestPoint2D::new(1.0, 0.0),
        TestPoint2D::new(0.0, 1.0),
        TestPoint2D::new(-1.0, 0.0),
        TestPoint2D::new(0.0, -1.0),
    ];
    let mut aRadii = [2.0, 2.0, 2.0, 2.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..4 as usize], &aRadii[..4 as usize]);
    let aResult = aTree.containing_search(&TestPoint2D::new(0.0, 0.0));
    assert_eq!(aResult.len(), 4);
}

#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_NoneContain() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(1.0, 0.0),
        TestPoint2D::new(0.0, 1.0),
    ];
    let mut aRadii = [0.01, 0.01, 0.01];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    let aResult = aTree.containing_search(&TestPoint2D::new(100.0, 100.0));
    assert_eq!(aResult.len(), 0);
}

#[test]
fn NCollection_KDTreeRadiiTest__ContainingSearch_3D() {
    let mut aPoints = [
        TestPoint3D::new(0.0, 0.0, 0.0),
        TestPoint3D::new(10.0, 0.0, 0.0),
    ];
    let mut aRadii = [5.0, 3.0];
    let mut aTree: KDTree<TestPoint3D, 3, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);

    let aResult = aTree.containing_search(&TestPoint3D::new(2.0, 0.0, 0.0));
    assert_eq!(aResult.len(), 1);
    near!((*aTree.point(aResult[(0) as usize]).unwrap()).X, 0.0, 1e-15);
}

#[test]
fn NCollection_KDTreeRadiiTest__NearestWeighted_EmptyTree() {
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    let mut aGap: f64 = 0.0;
    assert_eq!(
        aTree.nearest_weighted(&TestPoint2D::new(0.0, 0.0), Some(&mut aGap)),
        0usize
    );
}

#[test]
fn NCollection_KDTreeRadiiTest__NearestWeighted_SinglePoint() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0)];
    let mut aRadii = [3.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..1 as usize], &aRadii[..1 as usize]);
    let mut aGap: f64 = 0.0;

    let mut aIdx: usize = aTree.nearest_weighted(&TestPoint2D::new(1.0, 0.0), Some(&mut aGap));
    assert_eq!(aIdx, 1usize);
    near!(aGap, -2.0, 1e-10);
}

#[test]
fn NCollection_KDTreeRadiiTest__NearestWeighted_CloserSurface() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(10.0, 0.0)];
    let mut aRadii = [1.0, 5.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);
    let mut aGap: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_weighted(&TestPoint2D::new(2.0, 0.0), Some(&mut aGap));
    near!((*aTree.point(aIdx).unwrap()).X, 0.0, 1e-15);
    near!(aGap, 1.0, 1e-10);
}

#[test]
fn NCollection_KDTreeRadiiTest__NearestWeighted_InsideSphere() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(10.0, 0.0)];
    let mut aRadii = [5.0, 1.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);
    let mut aGap: f64 = 0.0;

    let mut aIdx: usize = aTree.nearest_weighted(&TestPoint2D::new(1.0, 0.0), Some(&mut aGap));
    near!((*aTree.point(aIdx).unwrap()).X, 0.0, 1e-15);
    near!(aGap, -4.0, 1e-10);
    assert!(aGap < 0.0);
}

#[test]
fn NCollection_KDTreeRadiiTest__NearestWeighted_LargeRadiusWins() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(100.0, 0.0)];
    let mut aRadii = [0.1, 200.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);
    let mut aGap: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_weighted(&TestPoint2D::new(1.0, 0.0), Some(&mut aGap));
    near!((*aTree.point(aIdx).unwrap()).X, 100.0, 1e-15);
    near!(aGap, -101.0, 1e-10);
}

#[test]
fn NCollection_KDTreeRadiiTest__NearestWeighted_WithoutGapOutput() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(10.0, 0.0)];
    let mut aRadii = [1.0, 1.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);

    let mut aIdx: usize = aTree.nearest_weighted(&TestPoint2D::new(1.0, 0.0), None);
    near!((*aTree.point(aIdx).unwrap()).X, 0.0, 1e-15);
}

#[test]
fn NCollection_KDTreeRadiiTest__NearestWeighted_3D() {
    let mut aPoints = [
        TestPoint3D::new(0.0, 0.0, 0.0),
        TestPoint3D::new(10.0, 0.0, 0.0),
        TestPoint3D::new(0.0, 10.0, 0.0),
    ];
    let mut aRadii = [2.0, 1.0, 1.0];
    let mut aTree: KDTree<TestPoint3D, 3, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    let mut aGap: f64 = 0.0;

    let mut aIdx: usize = aTree.nearest_weighted(&TestPoint3D::new(1.0, 0.0, 0.0), Some(&mut aGap));
    near!((*aTree.point(aIdx).unwrap()).X, 0.0, 1e-15);
    near!(aGap, -1.0, 1e-10);
}
