#[test]
fn NCollection_KDTreeRadiiTest__NearestWeighted_AllSameRadii() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(5.0, 0.0),
        TestPoint2D::new(10.0, 0.0),
    ];
    let mut aRadii = [1.0, 1.0, 1.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    let mut aGap: f64 = 0.0;
    let mut aSqDist: f64 = 0.0;
    let mut aWeightedIdx: usize =
        aTree.nearest_weighted(&TestPoint2D::new(4.0, 0.0), Some(&mut aGap));
    let mut aNearestIdx: usize =
        aTree.nearest_point(&TestPoint2D::new(4.0, 0.0), Some(&mut aSqDist));
    assert_eq!(aWeightedIdx, aNearestIdx);
}

#[test]
fn NCollection_KDTreeRadiiTest__BruteForce_ContainingSearch() {
    const THE_N: i32 = 200;
    let mut aRng = TestRandom::new(8080);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-50, 50), aRng.next_double(-50, 50));
        aRadii[(i) as usize] = aRng.next_double(0.1, 10.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);

    for q in (0)..(50) {
        let mut aQuery = TestPoint2D::new(aRng.next_double(-60, 60), aRng.next_double(-60, 60));
        let aResult = aTree.containing_search(&aQuery);

        let mut aBruteForce: TestSet<usize> = TestSet::default();
        for i in (0)..(THE_N) {
            let mut aDist: f64 = sqDist2D(aQuery, aPoints[(i) as usize]);
            if (aDist <= aRadii[(i) as usize] * aRadii[(i) as usize]) {
                aBruteForce.insert(((i + 1) as usize));
            }
        }
        assert_eq!(((aResult.len()) as usize), aBruteForce.len());
        let mut aTreeResult: TestSet<usize> = TestSet::default();
        for i in (0)..(aResult.len()) {
            aTreeResult.insert(aResult[(i) as usize]);
        }
        assert_eq!(aTreeResult, aBruteForce);
    }
}

#[test]
fn NCollection_KDTreeRadiiTest__BruteForce_NearestWeighted() {
    const THE_N: i32 = 200;
    let mut aRng = TestRandom::new(9090);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-50, 50), aRng.next_double(-50, 50));
        aRadii[(i) as usize] = aRng.next_double(0.1, 10.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);

    for q in (0)..(50) {
        let mut aQuery = TestPoint2D::new(aRng.next_double(-60, 60), aRng.next_double(-60, 60));
        let mut aGap: f64 = 0.0;
        let mut aIdx: usize = aTree.nearest_weighted(&aQuery, Some(&mut aGap));

        let mut aBruteGap: f64 = f64::MAX;
        for i in (0)..(THE_N) {
            let mut aDist: f64 = (sqDist2D(aQuery, aPoints[(i) as usize])).sqrt();
            let mut aG: f64 = aDist - aRadii[(i) as usize];
            if (aG < aBruteGap) {
                aBruteGap = aG;
            }
        }
        near!(aGap, aBruteGap, 1e-8);
    }
}

#[test]
fn NCollection_KDTreeRadiiTest__BruteForce_NearestWeighted_3D() {
    const THE_N: i32 = 200;
    let mut aRng = TestRandom::new(1212);
    let mut aPoints = vec![TestPoint3D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint3D::new(
            aRng.next_double(-50, 50),
            aRng.next_double(-50, 50),
            aRng.next_double(-50, 50),
        );
        aRadii[(i) as usize] = aRng.next_double(0.1, 10.0);
    }
    let mut aTree: KDTree<TestPoint3D, 3, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);
    for q in (0)..(50) {
        let mut aQuery = TestPoint3D::new(
            aRng.next_double(-60, 60),
            aRng.next_double(-60, 60),
            aRng.next_double(-60, 60),
        );
        let mut aGap: f64 = 0.0;
        let mut aIdx: usize = aTree.nearest_weighted(&aQuery, Some(&mut aGap));
        let mut aBruteGap: f64 = f64::MAX;
        for i in (0)..(THE_N) {
            let mut aDist: f64 = (sqDist3D(aQuery, aPoints[(i) as usize])).sqrt();
            let mut aG: f64 = aDist - aRadii[(i) as usize];
            if (aG < aBruteGap) {
                aBruteGap = aG;
            }
        }
        near!(aGap, aBruteGap, 1e-8);
    }
}

#[test]
fn NCollection_KDTreeRadiiTest__StressTest_1000Points() {
    const THE_N: i32 = 1000;
    let mut aRng = TestRandom::new(3030);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-100, 100), aRng.next_double(-100, 100));
        aRadii[(i) as usize] = aRng.next_double(0.01, 5.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);
    assert_eq!(aTree.len(), ((THE_N) as usize));

    for q in (0)..(100) {
        let mut aQuery = TestPoint2D::new(aRng.next_double(-110, 110), aRng.next_double(-110, 110));
        aTree.containing_search(&aQuery);
        aTree.nearest_weighted(&aQuery, None);
        aTree.nearest_point(&aQuery, None);
    }
}

#[test]
fn NCollection_KDTreeRadiiTest__StressTest_ZeroRadii() {
    const THE_N: i32 = 100;
    let mut aRng = TestRandom::new(4040);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-10, 10), aRng.next_double(-10, 10));
        aRadii[(i) as usize] = 0.0;
    }
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);

    let aResult = aTree.containing_search(&aPoints[(0) as usize]);
    assert!(aResult.len() >= 1);

    let aResult2 = aTree.containing_search(&TestPoint2D::new(999.0, 999.0));
    assert_eq!(aResult2.len(), 0);
}

#[test]
fn NCollection_KDTreeRadiiTest__StressTest_IdenticalRadii() {
    const THE_N: i32 = 500;
    const THE_R: f64 = 2.5;
    let mut aRng = TestRandom::new(5050);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-50, 50), aRng.next_double(-50, 50));
        aRadii[(i) as usize] = THE_R;
    }
    let mut aTreeR: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTreeR.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);
    let mut aTreeNoR: KDTree<TestPoint2D, 2, false> = KDTree::default();
    aTreeNoR.build(&(&aPoints)[..THE_N as usize]);
    for q in (0)..(50) {
        let mut aQuery = TestPoint2D::new(aRng.next_double(-60, 60), aRng.next_double(-60, 60));
        let mut aGap: f64 = 0.0;
        let mut aSqDist: f64 = 0.0;
        let mut aWIdx: usize = aTreeR.nearest_weighted(&aQuery, Some(&mut aGap));
        let mut aNIdx: usize = aTreeNoR.nearest_point(&aQuery, Some(&mut aSqDist));

        let mut aExpectedGap: f64 = (aSqDist).sqrt() - THE_R;
        near!(aGap, aExpectedGap, 1e-8);

        near!(
            (*aTreeR.point(aWIdx).unwrap()).X,
            (*aTreeNoR.point(aNIdx).unwrap()).X,
            1e-10
        );
        near!(
            (*aTreeR.point(aWIdx).unwrap()).Y,
            (*aTreeNoR.point(aNIdx).unwrap()).Y,
            1e-10
        );
    }
}
