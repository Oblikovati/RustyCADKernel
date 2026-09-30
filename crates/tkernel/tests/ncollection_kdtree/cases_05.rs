#[test]
fn NCollection_KDTreeTest__BruteForce_BoxSearchComparison() {
    const THE_N: i32 = 200;
    let mut aRng = TestRandom::new(99);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-50, 50), aRng.next_double(-50, 50));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    for q in (0)..(20) {
        let mut aX1: f64 = aRng.next_double(-60, 60);
        let mut aY1: f64 = aRng.next_double(-60, 60);
        let mut aX2: f64 = aX1 + aRng.next_double(5, 40);
        let mut aY2: f64 = aY1 + aRng.next_double(5, 40);
        let mut aMin = TestPoint2D::new(aX1, aY1);
        let mut aMax = TestPoint2D::new(aX2, aY2);
        let aKdResult = aTree.box_search(&aMin, &aMax);

        let mut aBfCount: usize = 0;
        for i in (0)..(THE_N) {
            if (aPoints[(i) as usize].X >= aX1
                && aPoints[(i) as usize].X <= aX2
                && aPoints[(i) as usize].Y >= aY1
                && aPoints[(i) as usize].Y <= aY2)
            {
                aBfCount += 1;
            }
        }
        assert_eq!(aKdResult.len(), aBfCount);
    }
}

#[test]
fn NCollection_KDTreeTest__LargeDataSet_1000Points() {
    const THE_N: i32 = 1000;
    let mut aRng = TestRandom::new(2024);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-1000, 1000), aRng.next_double(-1000, 1000));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    assert_eq!(aTree.len(), ((THE_N) as usize));

    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&aQuery, Some(&mut aSqDist));
    assert_ne!(aIdx, 0usize);

    let mut aBfBest: f64 = f64::MAX;
    for i in (0)..(THE_N) {
        let mut aDist: f64 = sqDist2D(aQuery, aPoints[(i) as usize]);
        if (aDist < aBfBest) {
            aBfBest = aDist;
        }
    }
    near!(aSqDist, aBfBest, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__LargeDataSet_10000Points() {
    const THE_N: i32 = 10000;
    let mut aRng = TestRandom::new(7777);
    let mut aPoints: Array1<TestPoint2D> = Array1::with_bounds(1, THE_N).unwrap();
    for i in (1)..=(THE_N) {
        aPoints
            .set_value(
                i,
                TestPoint2D::new(aRng.next_double(-5000, 5000), aRng.next_double(-5000, 5000)),
            )
            .unwrap();
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&aPoints.as_slice());
    assert_eq!(aTree.len(), ((THE_N) as usize));

    for q in (0)..(10) {
        let mut aQuery = {
            let coordinate_1 = aRng.next_double(-5500, 5500);
            let coordinate_0 = aRng.next_double(-5500, 5500);
            TestPoint2D::new(coordinate_0, coordinate_1)
        };
        let mut aKdSqDist: f64 = 0.0;
        aTree.nearest_point(&aQuery, Some(&mut aKdSqDist));

        let mut aBfBest: f64 = f64::MAX;
        for i in (1)..=(THE_N) {
            let mut aDist: f64 = sqDist2D(aQuery, (*aPoints.value(i).unwrap()));
            if (aDist < aBfBest) {
                aBfBest = aDist;
            }
        }
        near!(aKdSqDist, aBfBest, 1e-10);
    }
}

#[test]
fn NCollection_KDTreeTest__DuplicatePoints() {
    let mut aPoints = [
        TestPoint2D::new(1, 1),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(2, 2),
        TestPoint2D::new(2, 2),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    assert_eq!(aTree.len(), 5usize);

    let mut aSqDist: f64 = 0.0;
    let mut aQuery = TestPoint2D::new(1, 1);
    aTree.nearest_point(&aQuery, Some(&mut aSqDist));
    near!(aSqDist, 0.0, 1e-15);

    let aResult = aTree.range_search(&aQuery, 0.0);
    assert_eq!(aResult.len(), 3usize);
}

#[test]
fn NCollection_KDTreeTest__CustomPointType2D() {
    let mut aPoints = [
        TestPoint2D::new(1, 2),
        TestPoint2D::new(3, 4),
        TestPoint2D::new(5, 6),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aQuery = TestPoint2D::new(3.1, 4.1);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 3.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 4.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__CustomPointType3D() {
    let mut aPoints = [
        TestPoint3D::new(1, 2, 3),
        TestPoint3D::new(4, 5, 6),
        TestPoint3D::new(7, 8, 9),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aQuery = TestPoint3D::new(4.1, 5.1, 6.1);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 4.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 5.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Z, 6.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__Rebuild_DifferentData() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aPoints1 = [TestPoint2D::new(0, 0), TestPoint2D::new(1, 1)];
    aTree.build(&(&aPoints1)[..2 as usize]);
    assert_eq!(aTree.len(), 2usize);
    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint2D::new(0, 0), Some(&mut aSqDist));
    near!(aSqDist, 0.0, 1e-15);

    let mut aPoints2 = [
        TestPoint2D::new(10, 10),
        TestPoint2D::new(20, 20),
        TestPoint2D::new(30, 30),
    ];
    aTree.build(&(&aPoints2)[..3 as usize]);
    assert_eq!(aTree.len(), 3usize);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(11, 11), Some(&mut aSqDist));
    near!((*aTree.point(aIdx).unwrap()).X, 10.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__BuildFromArray1_NonUnitLower() {
    let mut aPoints: Array1<TestPoint2D> = Array1::with_bounds(5, 7).unwrap();
    aPoints.set_value(5, TestPoint2D::new(1.0, 2.0)).unwrap();
    aPoints.set_value(6, TestPoint2D::new(3.0, 4.0)).unwrap();
    aPoints.set_value(7, TestPoint2D::new(5.0, 6.0)).unwrap();
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&aPoints.as_slice());
    assert_eq!(aTree.len(), 3usize);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(3.1, 4.1), None);
    near!((*aTree.point(aIdx).unwrap()).X, 3.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 4.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__BuildZeroCount() {
    let mut aPoint = TestPoint2D::new(1, 1);
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(std::slice::from_ref(&aPoint))[..0 as usize]);
    assert!(aTree.is_empty());
}

#[test]
fn NCollection_KDTreeTest__PointAccessor_AllPoints() {
    let mut aPoints = [
        TestPoint2D::new(1, 2),
        TestPoint2D::new(3, 4),
        TestPoint2D::new(5, 6),
        TestPoint2D::new(7, 8),
        TestPoint2D::new(9, 10),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);

    let mut anOrigSet: TestSet<(i32, i32)> = TestSet::default();
    for i in (0)..(5) {
        anOrigSet.insert((
            ((aPoints[(i) as usize].X) as i32),
            ((aPoints[(i) as usize].Y) as i32),
        ));
    }
    let mut aTreeSet: TestSet<(i32, i32)> = TestSet::default();
    for i in (1)..=(aTree.len()) {
        aTreeSet.insert((
            (((*aTree.point(i).unwrap()).X) as i32),
            (((*aTree.point(i).unwrap()).Y) as i32),
        ));
    }
    assert_eq!(anOrigSet, aTreeSet);
}
