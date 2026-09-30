#[test]
fn NCollection_KDTreeTest__NearestPoint2D_ExactMatch() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(1.0, 1.0),
        TestPoint2D::new(2.0, 2.0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aQuery = TestPoint2D::new(1.0, 1.0);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 1.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 1.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_ClosestOfMany() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(10.0, 10.0),
        TestPoint2D::new(3.0, 4.0),
        TestPoint2D::new(7.0, 1.0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aQuery = TestPoint2D::new(3.1, 3.9);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 3.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 4.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_GridPoints() {
    let mut aPoints = vec![TestPoint2D::default(); 25 as usize];
    let mut k: i32 = 0;
    for i in (0)..(5) {
        for j in (0)..(5) {
            aPoints[{
                let previous = k;
                k += 1;
                previous as usize
            }] = TestPoint2D::new(((i) as f64), ((j) as f64));
        }
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..25 as usize]);

    let mut aQuery = TestPoint2D::new(2.1, 3.2);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 2.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 3.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_AllSamePoint() {
    let mut aPoints = [
        TestPoint2D::new(5.0, 5.0),
        TestPoint2D::new(5.0, 5.0),
        TestPoint2D::new(5.0, 5.0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aQuery = TestPoint2D::new(0.0, 0.0);
    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&aQuery, Some(&mut aSqDist));
    assert_ne!(aIdx, 0usize);
    near!(aSqDist, 50.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_CollinearPoints() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(1.0, 0.0),
        TestPoint2D::new(2.0, 0.0),
        TestPoint2D::new(3.0, 0.0),
        TestPoint2D::new(4.0, 0.0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut aQuery = TestPoint2D::new(2.3, 0.0);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 2.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 0.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_NegativeCoordinates() {
    let mut aPoints = [
        TestPoint2D::new(-5.0, -3.0),
        TestPoint2D::new(-1.0, -2.0),
        TestPoint2D::new(-4.0, -7.0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aQuery = TestPoint2D::new(-1.0, -2.0);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, -1.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, -2.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_QueryFarAway() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(1.0, 1.0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aQuery = TestPoint2D::new(1e6, 1e6);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 1.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 1.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_SqDistanceOutput() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(3.0, 4.0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aSqDist: f64 = 0.0;
    let mut aQuery = TestPoint2D::new(0.0, 0.0);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, Some(&mut aSqDist));
    near!((*aTree.point(aIdx).unwrap()).X, 0.0, 1e-15);
    near!(aSqDist, 0.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_EmptyTree() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aQuery = TestPoint2D::new(1.0, 1.0);
    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&aQuery, Some(&mut aSqDist));
    assert_eq!(aIdx, 0usize);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_SinglePoint() {
    let mut aPoint = TestPoint2D::new(3.0, 4.0);
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(std::slice::from_ref(&aPoint))[..1 as usize]);
    let mut aSqDist: f64 = 0.0;
    let mut aQuery = TestPoint2D::new(0.0, 0.0);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, Some(&mut aSqDist));
    assert_eq!(aIdx, 1usize);
    near!(aSqDist, 25.0, 1e-10);
}
