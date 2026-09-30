#[test]
fn NCollection_KDTreeTest__KNearest2D_AllDuplicates() {
    let mut aPoints = [
        TestPoint2D::new(3, 3),
        TestPoint2D::new(3, 3),
        TestPoint2D::new(3, 3),
        TestPoint2D::new(3, 3),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aCount: usize = aTree.k_nearest_points(
        &TestPoint2D::new(0, 0),
        3 as usize,
        &mut anIndices,
        &mut aDists,
    );
    assert_eq!(aCount, 3usize);

    for i in (1)..=(3) {
        near!((*aDists.value(i).unwrap()), 18.0, 1e-10);
    }
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_ValidIndices() {
    const THE_N: i32 = 50;
    let mut aRng = TestRandom::new(555);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-10, 10), aRng.next_double(-10, 10));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aCount: usize = aTree.k_nearest_points(
        &TestPoint2D::new(0, 0),
        10 as usize,
        &mut anIndices,
        &mut aDists,
    );
    assert_eq!(aCount, 10usize);
    for i in (1)..=((aCount) as i32) {
        assert!((*anIndices.value(i).unwrap()) >= 1usize);
        assert!((*anIndices.value(i).unwrap()) <= ((THE_N) as usize));
    }
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_VerifyDistances() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(3, 4),
        TestPoint2D::new(1, 0),
        TestPoint2D::new(0, 1),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 4 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 4usize);

    for i in (1)..=((aCount) as i32) {
        let mut aExpected: f64 = sqDist2D(
            aQuery,
            (*aTree.point((*anIndices.value(i).unwrap())).unwrap()),
        );
        near!((*aDists.value(i).unwrap()), aExpected, 1e-15);
    }
}

#[test]
fn NCollection_KDTreeTest__KNearest3D_BruteForce() {
    const THE_N: i32 = 100;
    const THE_K: usize = 7;
    let mut aRng = TestRandom::new(333);
    let mut aPoints = vec![TestPoint3D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint3D::new(
            aRng.next_double(-50, 50),
            aRng.next_double(-50, 50),
            aRng.next_double(-50, 50),
        );
    }
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    for q in (0)..(20) {
        let mut aQuery = {
            let coordinate_2 = aRng.next_double(-60, 60);
            let coordinate_1 = aRng.next_double(-60, 60);
            let coordinate_0 = aRng.next_double(-60, 60);
            TestPoint3D::new(coordinate_0, coordinate_1, coordinate_2)
        };
        let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
        let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
        let mut aCount: usize =
            aTree.k_nearest_points(&aQuery, THE_K as usize, &mut anIndices, &mut aDists);
        assert_eq!(aCount, THE_K);

        let mut aBfDists = vec![0.0_f64; THE_N as usize];
        for i in (0)..(THE_N) {
            aBfDists[(i) as usize] = sqDist3D(aQuery, aPoints[(i) as usize]);
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
fn NCollection_KDTreeTest__KNearest2D_LargeK() {
    const THE_N: i32 = 500;
    const THE_K: usize = 50;
    let mut aRng = TestRandom::new(789);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-100, 100), aRng.next_double(-100, 100));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, THE_K as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, THE_K);

    for i in (1)..((aCount) as i32) {
        assert!((*aDists.value(i).unwrap()) <= (*aDists.value(i + 1).unwrap()));
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
fn NCollection_KDTreeTest__KNearest2D_UniqueIndices() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 0),
        TestPoint2D::new(2, 0),
        TestPoint2D::new(3, 0),
        TestPoint2D::new(4, 0),
        TestPoint2D::new(5, 0),
        TestPoint2D::new(6, 0),
        TestPoint2D::new(7, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..8 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aCount: usize = aTree.k_nearest_points(
        &TestPoint2D::new(3.5, 0),
        5 as usize,
        &mut anIndices,
        &mut aDists,
    );
    assert_eq!(aCount, 5usize);
    let mut aIdxSet: TestSet<usize> = TestSet::default();
    for i in (1)..=((aCount) as i32) {
        aIdxSet.insert((*anIndices.value(i).unwrap()));
    }
    assert_eq!(aIdxSet.len(), 5usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_NegativeRadius() {
    let mut aPoints = [TestPoint2D::new(0, 0)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..1 as usize]);
    let aResult = aTree.range_search(&TestPoint2D::new(0, 0), -1.0);
    assert_eq!(aResult.len(), 0usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_LargeRadiusAll() {
    let mut aPoints = [
        TestPoint2D::new(-100, -100),
        TestPoint2D::new(100, 100),
        TestPoint2D::new(0, 0),
        TestPoint2D::new(50, -50),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let aResult = aTree.range_search(&TestPoint2D::new(0, 0), 1e6);
    assert_eq!(aResult.len(), 4usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_VerySmallRadius() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1e-8, 1e-8),
        TestPoint2D::new(1, 1),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let aResult = aTree.range_search(&TestPoint2D::new(0, 0), 1e-7);
    assert_eq!(aResult.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_OffCenter() {
    const THE_N: i32 = 25;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
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
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let aResult = aTree.range_search(&TestPoint2D::new(2, 2), 0.6);
    assert_eq!(aResult.len(), 1usize);

    let aResult2 = aTree.range_search(&TestPoint2D::new(2, 2), 1.0);
    assert_eq!(aResult2.len(), 5usize);
}
