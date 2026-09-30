#[test]
fn NCollection_KDTreeTest__MovedTreeQueriesWork() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(5, 5),
        TestPoint2D::new(10, 10),
        TestPoint2D::new(3, 4),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aMoved: KDTree<TestPoint2D, 2> = std::mem::take(&mut aTree);

    let mut aSqDist: f64 = 0.0;
    assert_ne!(
        aMoved.nearest_point(&TestPoint2D::new(3, 3), Some(&mut aSqDist)),
        0usize
    );
    let mut anIdx: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDist: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    assert_eq!(
        aMoved.k_nearest_points(&TestPoint2D::new(0, 0), 2 as usize, &mut anIdx, &mut aDist),
        2usize
    );
    let aRange = aMoved.range_search(&TestPoint2D::new(5, 5), 20.0);
    assert!(aRange.len() > 0usize);
    let aBox = aMoved.box_search(&TestPoint2D::new(-1, -1), &TestPoint2D::new(11, 11));
    assert!(aBox.len() > 0usize);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_CrossHyperplane() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(3, 0),
        TestPoint2D::new(1, 2),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);

    let mut aQuery = TestPoint2D::new(1.5, 1.9);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 1.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 2.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_TwoEquidistant() {
    let mut aPoints = [TestPoint2D::new(-1, 0), TestPoint2D::new(1, 0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint2D::new(0, 0), Some(&mut aSqDist));
    near!(aSqDist, 1.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_VeryClosePoints() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(1e-10, 1e-10),
        TestPoint2D::new(1.0, 1.0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(5e-11, 5e-11), Some(&mut aSqDist));

    assert!(aSqDist < 1e-18);
    let _ = aIdx;
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_LargeCoordinates() {
    let mut aPoints = [
        TestPoint2D::new(1e12, 1e12),
        TestPoint2D::new(-1e12, -1e12),
        TestPoint2D::new(0, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(1e12 + 1.0, 1e12 + 1.0), None);
    near!((*aTree.point(aIdx).unwrap()).X, 1e12, 1.0);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_MidpointQuery() {
    let mut aPoints = [TestPoint2D::new(0, 0), TestPoint2D::new(10, 0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint2D::new(5, 0), Some(&mut aSqDist));
    near!(aSqDist, 25.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_NegativeQuadrantOnly() {
    let mut aPoints = [
        TestPoint2D::new(-10, -10),
        TestPoint2D::new(-5, -5),
        TestPoint2D::new(-1, -1),
        TestPoint2D::new(-20, -30),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(-5.1, -5.1), None);
    near!((*aTree.point(aIdx).unwrap()).X, -5.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_CircularLayout() {
    const THE_N: i32 = 12;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        let mut anAngle: f64 = 2.0 * 3.14159265358979323846 * (i as f64) / (THE_N as f64);
        aPoints[(i) as usize] = TestPoint2D::new(10.0 * (anAngle).cos(), 10.0 * (anAngle).sin());
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint2D::new(0, 0), Some(&mut aSqDist));
    near!(aSqDist, 100.0, 1e-8);

    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(10.1, 0.0), None);
    near!((*aTree.point(aIdx).unwrap()).X, 10.0, 0.1);
    near!((*aTree.point(aIdx).unwrap()).Y, 0.0, 0.1);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_LShape() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 0),
        TestPoint2D::new(2, 0),
        TestPoint2D::new(3, 0),
        TestPoint2D::new(4, 0),
        TestPoint2D::new(0, 1),
        TestPoint2D::new(0, 2),
        TestPoint2D::new(0, 3),
        TestPoint2D::new(0, 4),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..9 as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(0.1, 2.9), None);
    near!((*aTree.point(aIdx).unwrap()).X, 0.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 3.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_MultipleQueries() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(10, 0),
        TestPoint2D::new(0, 10),
        TestPoint2D::new(10, 10),
        TestPoint2D::new(5, 5),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);

    let aCases = [
        (TestPoint2D::new(0.1, 0.1), 0 as f64, 0 as f64),
        (TestPoint2D::new(9.9, 0.1), 10 as f64, 0 as f64),
        (TestPoint2D::new(0.1, 9.9), 0 as f64, 10 as f64),
        (TestPoint2D::new(9.9, 9.9), 10 as f64, 10 as f64),
        (TestPoint2D::new(5.1, 5.1), 5 as f64, 5 as f64),
    ];
    for aCase in aCases {
        let mut aIdx: usize = aTree.nearest_point(&aCase.0, None);
        near!((*aTree.point(aIdx).unwrap()).X, aCase.1, 1e-15);
        near!((*aTree.point(aIdx).unwrap()).Y, aCase.2, 1e-15);
    }
}
