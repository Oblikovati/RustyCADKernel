#[test]
fn NCollection_KDTreeTest__BoxSearch3D_SinglePointInOut() {
    let mut aPoint = TestPoint3D::new(5, 5, 5);
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(std::slice::from_ref(&aPoint))[..1 as usize]);
    let aResult1 = aTree.box_search(&TestPoint3D::new(4, 4, 4), &TestPoint3D::new(6, 6, 6));
    assert_eq!(aResult1.len(), 1usize);
    let aResult2 = aTree.box_search(&TestPoint3D::new(6, 6, 6), &TestPoint3D::new(7, 7, 7));
    assert_eq!(aResult2.len(), 0usize);
}

#[test]
fn NCollection_KDTreeTest__BruteForce3D_NearestComparison() {
    const THE_N: i32 = 200;
    let mut aRng = TestRandom::new(1001);
    let mut aPoints = vec![TestPoint3D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint3D::new(
            aRng.next_double(-100, 100),
            aRng.next_double(-100, 100),
            aRng.next_double(-100, 100),
        );
    }
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    for q in (0)..(50) {
        let mut aQuery = {
            let coordinate_2 = aRng.next_double(-120, 120);
            let coordinate_1 = aRng.next_double(-120, 120);
            let coordinate_0 = aRng.next_double(-120, 120);
            TestPoint3D::new(coordinate_0, coordinate_1, coordinate_2)
        };
        let mut aKdSqDist: f64 = 0.0;
        aTree.nearest_point(&aQuery, Some(&mut aKdSqDist));
        let mut aBfBest: f64 = f64::MAX;
        for i in (0)..(THE_N) {
            let mut aDist: f64 = sqDist3D(aQuery, aPoints[(i) as usize]);
            if (aDist < aBfBest) {
                aBfBest = aDist;
            }
        }
        near!(aKdSqDist, aBfBest, 1e-10);
    }
}

#[test]
fn NCollection_KDTreeTest__BruteForce_NearestMultipleSeeds() {
    const THE_N: i32 = 150;
    let aSeeds: [u32; 7] = [1, 13, 42, 100, 9999, 12345, 65535];
    for aSeed in aSeeds {
        let mut aRng = TestRandom::new(aSeed);
        let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
        for i in (0)..(THE_N) {
            aPoints[(i) as usize] =
                TestPoint2D::new(aRng.next_double(-200, 200), aRng.next_double(-200, 200));
        }
        let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
        aTree.build(&(&aPoints)[..THE_N as usize]);
        for q in (0)..(20) {
            let mut aQuery = {
                let coordinate_1 = aRng.next_double(-250, 250);
                let coordinate_0 = aRng.next_double(-250, 250);
                TestPoint2D::new(coordinate_0, coordinate_1)
            };
            let mut aKdSqDist: f64 = 0.0;
            aTree.nearest_point(&aQuery, Some(&mut aKdSqDist));
            let mut aBfBest: f64 = f64::MAX;
            for i in (0)..(THE_N) {
                let mut aDist: f64 = sqDist2D(aQuery, aPoints[(i) as usize]);
                if (aDist < aBfBest) {
                    aBfBest = aDist;
                }
            }
            near!(aKdSqDist, aBfBest, 1e-10);
        }
    }
}

#[test]
fn NCollection_KDTreeTest__RangeSearch_IndicesMatchPoints() {
    const THE_N: i32 = 100;
    let mut aRng = TestRandom::new(666);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-30, 30), aRng.next_double(-30, 30));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aQuery = TestPoint2D::new(5, 5);
    let mut aRadius: f64 = 10.0;
    let aResult = aTree.range_search(&aQuery, aRadius);

    for i in (0)..(aResult.len()) {
        let mut anIdx: usize = aResult[(i) as usize];
        assert!(anIdx >= 1usize);
        assert!(anIdx <= ((THE_N) as usize));
        let mut aDist: f64 = sqDist2D(aQuery, (*aTree.point(anIdx).unwrap()));
        assert!(aDist <= aRadius * aRadius + 1e-10);
    }

    let mut aBfCount: usize = 0;
    for i in (0)..(THE_N) {
        if (sqDist2D(aQuery, aPoints[(i) as usize]) <= aRadius * aRadius) {
            aBfCount += 1;
        }
    }
    assert_eq!(aResult.len(), aBfCount);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch_IndicesMatchPoints() {
    const THE_N: i32 = 100;
    let mut aRng = TestRandom::new(777);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-30, 30), aRng.next_double(-30, 30));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aMin = TestPoint2D::new(-10, -10);
    let mut aMax = TestPoint2D::new(10, 10);
    let aResult = aTree.box_search(&aMin, &aMax);
    for i in (0)..(aResult.len()) {
        let mut anIdx: usize = aResult[(i) as usize];
        let aP = (*aTree.point(anIdx).unwrap());
        assert!(aP.X >= -10.0);
        assert!(aP.X <= 10.0);
        assert!(aP.Y >= -10.0);
        assert!(aP.Y <= 10.0);
    }
    let mut aBfCount: usize = 0;
    for i in (0)..(THE_N) {
        if (aPoints[(i) as usize].X >= -10.0
            && aPoints[(i) as usize].X <= 10.0
            && aPoints[(i) as usize].Y >= -10.0
            && aPoints[(i) as usize].Y <= 10.0)
        {
            aBfCount += 1;
        }
    }
    assert_eq!(aResult.len(), aBfCount);
}

#[test]
fn NCollection_KDTreeTest__KNearest_IndicesMatchPoints() {
    const THE_N: i32 = 100;
    const THE_K: usize = 10;
    let mut aRng = TestRandom::new(321);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-50, 50), aRng.next_double(-50, 50));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, THE_K as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, THE_K);
    for i in (1)..=((aCount) as i32) {
        let mut anIdx: usize = (*anIndices.value(i).unwrap());
        assert!(anIdx >= 1usize);
        assert!(anIdx <= ((THE_N) as usize));
        let mut aExpected: f64 = sqDist2D(aQuery, (*aTree.point(anIdx).unwrap()));
        near!((*aDists.value(i).unwrap()), aExpected, 1e-10);
    }

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

#[test]
fn NCollection_KDTreeTest__Degenerate_AllOnXAxis() {
    const THE_N: i32 = 20;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(((i) as f64), 0.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(7.3, 0), Some(&mut aSqDist));
    near!((*aTree.point(aIdx).unwrap()).X, 7.0, 1e-15);
    near!(aSqDist, 0.09, 1e-10);

    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    aTree.k_nearest_points(
        &TestPoint2D::new(10, 0),
        3 as usize,
        &mut anIndices,
        &mut aDists,
    );
    assert_eq!(anIndices.len(), 3);

    let aRange = aTree.range_search(&TestPoint2D::new(10, 0), 2.5);

    assert_eq!(aRange.len(), 5usize);
}

#[test]
fn NCollection_KDTreeTest__Degenerate_AllOnYAxis() {
    const THE_N: i32 = 20;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(0.0, ((i) as f64));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(0, 15.4), None);
    near!((*aTree.point(aIdx).unwrap()).Y, 15.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__Degenerate_AllIdentical() {
    const THE_N: i32 = 20;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(42.0, 42.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint2D::new(42, 42), Some(&mut aSqDist));
    near!(aSqDist, 0.0, 1e-15);
    let aRange = aTree.range_search(&TestPoint2D::new(42, 42), 0.0);
    assert_eq!(aRange.len(), ((THE_N) as usize));
}

#[test]
fn NCollection_KDTreeTest__Degenerate_GridAlignedToAxes() {
    const THE_N: i32 = 100;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut k: i32 = 0;
    for i in (0)..(10) {
        for j in (0)..(10) {
            aPoints[{
                let previous = k;
                k += 1;
                previous as usize
            }] = TestPoint2D::new(((i) as f64), ((j) as f64));
        }
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let mut aRng = TestRandom::new(999);
    for q in (0)..(30) {
        let mut aQuery = {
            let coordinate_1 = aRng.next_double(-1, 10);
            let coordinate_0 = aRng.next_double(-1, 10);
            TestPoint2D::new(coordinate_0, coordinate_1)
        };
        let mut aKdSqDist: f64 = 0.0;
        aTree.nearest_point(&aQuery, Some(&mut aKdSqDist));
        let mut aBfBest: f64 = f64::MAX;
        for i in (0)..(THE_N) {
            let mut aDist: f64 = sqDist2D(aQuery, aPoints[(i) as usize]);
            if (aDist < aBfBest) {
                aBfBest = aDist;
            }
        }
        near!(aKdSqDist, aBfBest, 1e-10);
    }
}
