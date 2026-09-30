#[test]
fn NCollection_KDTreeTest__BruteForce_AllQueries_500Points() {
    const THE_N: i32 = 500;
    let mut aRng = TestRandom::new(2026);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-500, 500), aRng.next_double(-500, 500));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    for q in (0)..(30) {
        let mut aQuery = {
            let coordinate_1 = aRng.next_double(-600, 600);
            let coordinate_0 = aRng.next_double(-600, 600);
            TestPoint2D::new(coordinate_0, coordinate_1)
        };

        {
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

        {
            const K: usize = 10;
            let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
            let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
            aTree.k_nearest_points(&aQuery, K as usize, &mut anIndices, &mut aDists);
            let mut aBfDists = vec![0.0_f64; THE_N as usize];
            for i in (0)..(THE_N) {
                aBfDists[(i) as usize] = sqDist2D(aQuery, aPoints[(i) as usize]);
            }
            aBfDists.sort_by(|left, right| left.partial_cmp(right).unwrap());
            near!(
                (*aDists.value(((K) as i32)).unwrap()),
                aBfDists[(K - 1) as usize],
                1e-10
            );
        }

        {
            let mut aRadius: f64 = aRng.next_double(20, 100);
            let aKdResult = aTree.range_search(&aQuery, aRadius);
            let mut aRadiusSq: f64 = aRadius * aRadius;
            let mut aBfCount: usize = 0;
            for i in (0)..(THE_N) {
                if (sqDist2D(aQuery, aPoints[(i) as usize]) <= aRadiusSq) {
                    aBfCount += 1;
                }
            }
            assert_eq!(aKdResult.len(), aBfCount);
        }

        {
            let mut aHalf: f64 = aRng.next_double(20, 100);
            let mut aMin = TestPoint2D::new(aQuery.X - aHalf, aQuery.Y - aHalf);
            let mut aMax = TestPoint2D::new(aQuery.X + aHalf, aQuery.Y + aHalf);
            let aKdResult = aTree.box_search(&aMin, &aMax);
            let mut aBfCount: usize = 0;
            for i in (0)..(THE_N) {
                if (aPoints[(i) as usize].X >= aMin.X
                    && aPoints[(i) as usize].X <= aMax.X
                    && aPoints[(i) as usize].Y >= aMin.Y
                    && aPoints[(i) as usize].Y <= aMax.Y)
                {
                    aBfCount += 1;
                }
            }
            assert_eq!(aKdResult.len(), aBfCount);
        }
    }
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_EmptyTree() {
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    let mut aSqDist: f64 = 0.0;
    let aResult = aTree.nearest_points(&TestPoint2D::new(0, 0), 0.0, &mut aSqDist);
    assert_eq!(aResult.len(), 0);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_SinglePoint() {
    let mut aPoint = TestPoint2D::new(3, 4);
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(std::slice::from_ref(&aPoint))[..1 as usize]);
    let mut aSqDist: f64 = 0.0;
    let aResult = aTree.nearest_points(&TestPoint2D::new(0, 0), 0.0, &mut aSqDist);
    assert_eq!(aResult.len(), 1);
    near!(aSqDist, 25.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_TwoEquidistant() {
    let mut aPoints = [TestPoint2D::new(1, 0), TestPoint2D::new(-1, 0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut aSqDist: f64 = 0.0;
    let aResult = aTree.nearest_points(&TestPoint2D::new(0, 0), 1e-12, &mut aSqDist);
    assert_eq!(aResult.len(), 2);
    near!(aSqDist, 1.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_FourEquidistant() {
    let mut aPoints = [
        TestPoint2D::new(5, 0),
        TestPoint2D::new(-5, 0),
        TestPoint2D::new(0, 5),
        TestPoint2D::new(0, -5),
        TestPoint2D::new(100, 100),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut aSqDist: f64 = 0.0;
    let aResult = aTree.nearest_points(&TestPoint2D::new(0, 0), 1e-12, &mut aSqDist);
    assert_eq!(aResult.len(), 4);
    near!(aSqDist, 25.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_SixEquidistant3D() {
    let mut aPoints = [
        TestPoint3D::new(2, 0, 0),
        TestPoint3D::new(-2, 0, 0),
        TestPoint3D::new(0, 2, 0),
        TestPoint3D::new(0, -2, 0),
        TestPoint3D::new(0, 0, 2),
        TestPoint3D::new(0, 0, -2),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..6 as usize]);
    let mut aSqDist: f64 = 0.0;
    let aResult = aTree.nearest_points(&TestPoint3D::new(0, 0, 0), 1e-12, &mut aSqDist);
    assert_eq!(aResult.len(), 6);
    near!(aSqDist, 4.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_NoneEquidistant() {
    let mut aPoints = [
        TestPoint2D::new(1, 0),
        TestPoint2D::new(3, 0),
        TestPoint2D::new(6, 0),
        TestPoint2D::new(10, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aSqDist: f64 = 0.0;
    let aResult = aTree.nearest_points(&TestPoint2D::new(0, 0), 1e-12, &mut aSqDist);
    assert_eq!(aResult.len(), 1);
    near!(aSqDist, 1.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_AllSamePoint() {
    let mut aPoints = [
        TestPoint2D::new(5, 5),
        TestPoint2D::new(5, 5),
        TestPoint2D::new(5, 5),
        TestPoint2D::new(5, 5),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aSqDist: f64 = 0.0;
    let aResult = aTree.nearest_points(&TestPoint2D::new(0, 0), 1e-12, &mut aSqDist);
    assert_eq!(aResult.len(), 4);
    near!(aSqDist, 50.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_WithTolerance() {
    let mut aPoints = [
        TestPoint2D::new(1.0, 0),
        TestPoint2D::new(1.001, 0),
        TestPoint2D::new(1.002, 0),
        TestPoint2D::new(5.0, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aSqDist: f64 = 0.0;

    let aResult1 = aTree.nearest_points(&TestPoint2D::new(0, 0), 0.0, &mut aSqDist);
    assert_eq!(aResult1.len(), 1);

    let aResult2 = aTree.nearest_points(&TestPoint2D::new(0, 0), 0.005, &mut aSqDist);
    assert_eq!(aResult2.len(), 3);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_ExactMatchQuery() {
    let mut aPoints = [
        TestPoint2D::new(3, 4),
        TestPoint2D::new(0, 0),
        TestPoint2D::new(10, 10),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut aSqDist: f64 = 0.0;

    let aResult = aTree.nearest_points(&TestPoint2D::new(3, 4), 1e-12, &mut aSqDist);
    assert_eq!(aResult.len(), 1);
    near!(aSqDist, 0.0, 1e-15);
}
