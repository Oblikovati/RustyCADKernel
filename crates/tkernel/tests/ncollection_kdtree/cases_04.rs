#[test]
fn NCollection_KDTreeTest__RangeSearch2D_SinglePoint() {
    let mut aPoint = TestPoint2D::new(5.0, 5.0);
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(std::slice::from_ref(&aPoint))[..1 as usize]);
    let mut aQuery = TestPoint2D::new(0, 0);

    let aResult1 = aTree.range_search(&aQuery, 1.0);
    assert_eq!(aResult1.len(), 0usize);

    let aResult2 = aTree.range_search(&aQuery, 10.0);
    assert_eq!(aResult2.len(), 1usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_EmptyResult() {
    let mut aPoints = [TestPoint2D::new(10, 10), TestPoint2D::new(20, 20)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aMin = TestPoint2D::new(0, 0);
    let mut aMax = TestPoint2D::new(5, 5);
    let aResult = aTree.box_search(&aMin, &aMax);
    assert_eq!(aResult.len(), 0usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_AllInBox() {
    let mut aPoints = [
        TestPoint2D::new(1, 1),
        TestPoint2D::new(2, 2),
        TestPoint2D::new(3, 3),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aMin = TestPoint2D::new(0, 0);
    let mut aMax = TestPoint2D::new(5, 5);
    let aResult = aTree.box_search(&aMin, &aMax);
    assert_eq!(aResult.len(), 3usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_SomeInBox() {
    let mut aPoints = [
        TestPoint2D::new(1, 1),
        TestPoint2D::new(3, 3),
        TestPoint2D::new(5, 5),
        TestPoint2D::new(7, 7),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aMin = TestPoint2D::new(0, 0);
    let mut aMax = TestPoint2D::new(4, 4);
    let aResult = aTree.box_search(&aMin, &aMax);
    assert_eq!(aResult.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_PointOnBoundary() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(5, 5),
        TestPoint2D::new(5, 0),
        TestPoint2D::new(0, 5),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aMin = TestPoint2D::new(0, 0);
    let mut aMax = TestPoint2D::new(5, 5);
    let aResult = aTree.box_search(&aMin, &aMax);
    assert_eq!(aResult.len(), 4usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch3D_Basic() {
    let mut aPoints = [
        TestPoint3D::new(1, 1, 1),
        TestPoint3D::new(2, 2, 2),
        TestPoint3D::new(5, 5, 5),
        TestPoint3D::new(10, 10, 10),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aMin = TestPoint3D::new(0, 0, 0);
    let mut aMax = TestPoint3D::new(3, 3, 3);
    let aResult = aTree.box_search(&aMin, &aMax);
    assert_eq!(aResult.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_EmptyTree() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aMin = TestPoint2D::new(0, 0);
    let mut aMax = TestPoint2D::new(10, 10);
    let aResult = aTree.box_search(&aMin, &aMax);
    assert_eq!(aResult.len(), 0usize);
}

#[test]
fn NCollection_KDTreeTest__BruteForce_NearestComparison() {
    const THE_N: i32 = 200;
    let mut aRng = TestRandom::new(42);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-100, 100), aRng.next_double(-100, 100));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    for q in (0)..(50) {
        let mut aQuery = {
            let coordinate_1 = aRng.next_double(-120, 120);
            let coordinate_0 = aRng.next_double(-120, 120);
            TestPoint2D::new(coordinate_0, coordinate_1)
        };
        let mut aKdSqDist: f64 = 0.0;
        let mut aKdIdx: usize = aTree.nearest_point(&aQuery, Some(&mut aKdSqDist));

        let mut aBestBfSqDist: f64 = f64::MAX;
        for i in (0)..(THE_N) {
            let mut aDist: f64 = sqDist2D(aQuery, aPoints[(i) as usize]);
            if (aDist < aBestBfSqDist) {
                aBestBfSqDist = aDist;
            }
        }
        near!(aKdSqDist, aBestBfSqDist, 1e-10);
    }
}

#[test]
fn NCollection_KDTreeTest__BruteForce_KNearestComparison() {
    const THE_N: i32 = 200;
    const THE_K: usize = 5;
    let mut aRng = TestRandom::new(123);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-50, 50), aRng.next_double(-50, 50));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    for q in (0)..(30) {
        let mut aQuery = {
            let coordinate_1 = aRng.next_double(-60, 60);
            let coordinate_0 = aRng.next_double(-60, 60);
            TestPoint2D::new(coordinate_0, coordinate_1)
        };
        let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
        let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
        let mut aCount: usize =
            aTree.k_nearest_points(&aQuery, THE_K as usize, &mut anIndices, &mut aDists);
        assert_eq!(aCount, THE_K);

        let mut aBfDists = vec![0.0_f64; THE_N as usize];
        for i in (0)..(THE_N) {
            aBfDists[(i) as usize] = sqDist2D(aQuery, aPoints[(i) as usize]);
        }
        aBfDists.sort_by(|left, right| left.partial_cmp(right).unwrap());

        near!(
            (*aDists.value(((THE_K) as i32)).unwrap()),
            aBfDists[(THE_K - 1) as usize],
            1e-10
        );
    }
}

#[test]
fn NCollection_KDTreeTest__BruteForce_RangeSearchComparison() {
    const THE_N: i32 = 200;
    let mut aRng = TestRandom::new(77);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-50, 50), aRng.next_double(-50, 50));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    for q in (0)..(20) {
        let mut aQuery = {
            let coordinate_1 = aRng.next_double(-60, 60);
            let coordinate_0 = aRng.next_double(-60, 60);
            TestPoint2D::new(coordinate_0, coordinate_1)
        };
        let mut aRadius: f64 = aRng.next_double(5.0, 30.0);
        let aKdResult = aTree.range_search(&aQuery, aRadius);

        let mut aBfCount: usize = 0;
        let mut aRadiusSq: f64 = aRadius * aRadius;
        for i in (0)..(THE_N) {
            if (sqDist2D(aQuery, aPoints[(i) as usize]) <= aRadiusSq) {
                aBfCount += 1;
            }
        }
        assert_eq!(aKdResult.len(), aBfCount);
    }
}
