#[test]
fn NCollection_KDTreeTest__Degenerate_PointsOnCircle() {
    const THE_N: i32 = 36;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        let mut anAngle: f64 = 2.0 * 3.14159265358979323846 * (i as f64) / (THE_N as f64);
        aPoints[(i) as usize] = TestPoint2D::new(100.0 * (anAngle).cos(), 100.0 * (anAngle).sin());
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint2D::new(0, 0), Some(&mut aSqDist));
    near!(aSqDist, 10000.0, 1.0);

    let aRange = aTree.range_search(&TestPoint2D::new(0, 0), 100.01);
    assert_eq!(aRange.len(), ((THE_N) as usize));
}

#[test]
fn NCollection_KDTreeTest__Degenerate_TwoClustersFar() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(0.1, 0.1),
        TestPoint2D::new(-0.1, 0.1),
        TestPoint2D::new(0, -0.1),
        TestPoint2D::new(1000, 1000),
        TestPoint2D::new(1000.1, 1000.1),
        TestPoint2D::new(999.9, 1000),
        TestPoint2D::new(1000, 999.9),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..8 as usize]);

    let mut aIdx1: usize = aTree.nearest_point(&TestPoint2D::new(0.05, 0.05), None);
    assert!((*aTree.point(aIdx1).unwrap()).X < 1.0);

    let mut aIdx2: usize = aTree.nearest_point(&TestPoint2D::new(1000.05, 1000.05), None);
    assert!((*aTree.point(aIdx2).unwrap()).X > 999.0);

    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    aTree.k_nearest_points(
        &TestPoint2D::new(0, 0),
        4 as usize,
        &mut anIndices,
        &mut aDists,
    );
    for i in (1)..=(4) {
        assert!((*aTree.point((*anIndices.value(i).unwrap())).unwrap()).X < 1.0);
    }
}

#[test]
fn NCollection_KDTreeTest__Degenerate_OneQuadrantOnly() {
    const THE_N: i32 = 30;
    let mut aRng = TestRandom::new(654);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(0, 100), aRng.next_double(0, 100));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(-10, -10), Some(&mut aSqDist));
    assert_ne!(aIdx, 0usize);

    let mut aBfBest: f64 = f64::MAX;
    for i in (0)..(THE_N) {
        let mut aDist: f64 = sqDist2D(TestPoint2D::new(-10, -10), aPoints[(i) as usize]);
        if (aDist < aBfBest) {
            aBfBest = aDist;
        }
    }
    near!(aSqDist, aBfBest, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__LargeDataSet_50000Points() {
    const THE_N: i32 = 50000;
    let mut aRng = TestRandom::new(31337);
    let mut aPoints: Array1<TestPoint2D> = Array1::with_bounds(1, THE_N).unwrap();
    for i in (1)..=(THE_N) {
        aPoints
            .set_value(
                i,
                TestPoint2D::new(
                    aRng.next_double(-10000, 10000),
                    aRng.next_double(-10000, 10000),
                ),
            )
            .unwrap();
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&aPoints.as_slice());
    assert_eq!(aTree.len(), ((THE_N) as usize));

    for q in (0)..(5) {
        let mut aQuery = {
            let coordinate_1 = aRng.next_double(-11000, 11000);
            let coordinate_0 = aRng.next_double(-11000, 11000);
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
        near!(aKdSqDist, aBfBest, 1e-8);
    }
}

#[test]
fn NCollection_KDTreeTest__LargeDataSet_3D_5000Points() {
    const THE_N: i32 = 5000;
    let mut aRng = TestRandom::new(4242);
    let mut aPoints = vec![TestPoint3D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint3D::new(
            aRng.next_double(-1000, 1000),
            aRng.next_double(-1000, 1000),
            aRng.next_double(-1000, 1000),
        );
    }
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    assert_eq!(aTree.len(), ((THE_N) as usize));
    for q in (0)..(10) {
        let mut aQuery = {
            let coordinate_2 = aRng.next_double(-1100, 1100);
            let coordinate_1 = aRng.next_double(-1100, 1100);
            let coordinate_0 = aRng.next_double(-1100, 1100);
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
        near!(aKdSqDist, aBfBest, 1e-8);
    }
}

#[test]
fn NCollection_KDTreeTest__ConsecutiveBuilds() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aRng = TestRandom::new(555);

    for aBuild in (0)..(5) {
        let mut aN: i32 = 10 + aBuild * 20;
        let mut aPoints: Array1<TestPoint2D> = Array1::with_bounds(1, aN).unwrap();
        for i in (1)..=(aN) {
            aPoints
                .set_value(
                    i,
                    TestPoint2D::new(aRng.next_double(-50, 50), aRng.next_double(-50, 50)),
                )
                .unwrap();
        }
        aTree.build(&aPoints.as_slice());
        assert_eq!(aTree.len(), ((aN) as usize));

        let mut aQuery = {
            let coordinate_1 = aRng.next_double(-55, 55);
            let coordinate_0 = aRng.next_double(-55, 55);
            TestPoint2D::new(coordinate_0, coordinate_1)
        };
        let mut aKdSqDist: f64 = 0.0;
        aTree.nearest_point(&aQuery, Some(&mut aKdSqDist));
        let mut aBfBest: f64 = f64::MAX;
        for i in (1)..=(aN) {
            let mut aDist: f64 = sqDist2D(aQuery, (*aPoints.value(i).unwrap()));
            if (aDist < aBfBest) {
                aBfBest = aDist;
            }
        }
        near!(aKdSqDist, aBfBest, 1e-10);
    }
}

#[test]
fn NCollection_KDTreeTest__AllQueryTypesAfterRebuild() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aPoints1 = [TestPoint2D::new(0, 0), TestPoint2D::new(1, 1)];
    aTree.build(&(&aPoints1)[..2 as usize]);
    aTree.clear();
    let mut aPoints2 = [
        TestPoint2D::new(10, 10),
        TestPoint2D::new(20, 20),
        TestPoint2D::new(15, 15),
        TestPoint2D::new(12, 18),
    ];
    aTree.build(&(&aPoints2)[..4 as usize]);

    let mut aSqDist: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(15.1, 15.1), Some(&mut aSqDist));
    near!((*aTree.point(aIdx).unwrap()).X, 15.0, 1e-15);

    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    assert_eq!(
        aTree.k_nearest_points(
            &TestPoint2D::new(15, 15),
            2 as usize,
            &mut anIndices,
            &mut aDists
        ),
        2usize
    );

    let aRange = aTree.range_search(&TestPoint2D::new(15, 15), 10.0);
    assert!(aRange.len() > 0usize);

    let aBox = aTree.box_search(&TestPoint2D::new(9, 9), &TestPoint2D::new(21, 21));
    assert!(aBox.len() > 0usize);
}

#[test]
fn NCollection_KDTreeTest__DuplicatePoints_AllSame3D() {
    const THE_N: i32 = 50;
    let mut aPoints = vec![TestPoint3D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint3D::new(7.0, 7.0, 7.0);
    }
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint3D::new(7, 7, 7), Some(&mut aSqDist));
    near!(aSqDist, 0.0, 1e-15);
    let aRange = aTree.range_search(&TestPoint3D::new(7, 7, 7), 0.0);
    assert_eq!(aRange.len(), ((THE_N) as usize));
}

#[test]
fn NCollection_KDTreeTest__DuplicatePointsMixed() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(5, 5),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..6 as usize]);
    let aRange = aTree.range_search(&TestPoint2D::new(1, 1), 0.0);
    assert_eq!(aRange.len(), 3usize);
    let aRange2 = aTree.range_search(&TestPoint2D::new(0, 0), 0.0);
    assert_eq!(aRange2.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__AllSamePoints_LargeSet2D() {
    const THE_N: i32 = 1000;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(3.0, 7.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    assert_eq!(aTree.len(), ((THE_N) as usize));

    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint2D::new(0.0, 0.0), Some(&mut aSqDist));
    near!(aSqDist, 58.0, 1e-10);

    aTree.nearest_point(&TestPoint2D::new(3.0, 7.0), Some(&mut aSqDist));
    near!(aSqDist, 0.0, 1e-15);
}
