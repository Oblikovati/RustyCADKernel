#[test]
fn NCollection_KDTreeTest__AllSamePoints_LargeSet3D() {
    const THE_N: i32 = 500;
    let mut aPoints = vec![TestPoint3D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint3D::new(-2.0, 4.0, 6.0);
    }
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    assert_eq!(aTree.len(), ((THE_N) as usize));
    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint3D::new(-2.0, 4.0, 6.0), Some(&mut aSqDist));
    near!(aSqDist, 0.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__AllSamePoints_KNearest() {
    const THE_N: i32 = 100;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(5.0, 5.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();

    let mut aCount: usize = aTree.k_nearest_points(
        &TestPoint2D::new(0.0, 0.0),
        10 as usize,
        &mut anIndices,
        &mut aDists,
    );
    assert_eq!(aCount, 10usize);
    for i in (1)..=((aCount) as i32) {
        near!((*aDists.value(i).unwrap()), 50.0, 1e-10);
    }

    let mut aCount2: usize = aTree.k_nearest_points(
        &TestPoint2D::new(0.0, 0.0),
        200 as usize,
        &mut anIndices,
        &mut aDists,
    );
    assert_eq!(aCount2, ((THE_N) as usize));
}

#[test]
fn NCollection_KDTreeTest__AllSamePoints_RangeSearch() {
    const THE_N: i32 = 100;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(5.0, 5.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let aResult = aTree.range_search(&TestPoint2D::new(5.0, 5.0), 0.0);
    assert_eq!(aResult.len(), THE_N as usize);

    let aResult2 = aTree.range_search(&TestPoint2D::new(4.0, 4.0), 2.0);
    assert_eq!(aResult2.len(), THE_N as usize);

    let aResult3 = aTree.range_search(&TestPoint2D::new(0.0, 0.0), 1.0);
    assert_eq!(aResult3.len(), 0);
}

#[test]
fn NCollection_KDTreeTest__AllSamePoints_BoxSearch() {
    const THE_N: i32 = 100;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(5.0, 5.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let aResult = aTree.box_search(&TestPoint2D::new(4.0, 4.0), &TestPoint2D::new(6.0, 6.0));
    assert_eq!(aResult.len(), THE_N as usize);

    let aResult2 = aTree.box_search(&TestPoint2D::new(5.0, 5.0), &TestPoint2D::new(5.0, 5.0));
    assert_eq!(aResult2.len(), THE_N as usize);

    let aResult3 = aTree.box_search(&TestPoint2D::new(0.0, 0.0), &TestPoint2D::new(4.9, 4.9));
    assert_eq!(aResult3.len(), 0);
}

#[test]
fn NCollection_KDTreeTest__AllSamePoints_NearestPoints() {
    const THE_N: i32 = 50;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(3.0, 3.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aSqDist: f64 = 0.0;

    let aResult = aTree.nearest_points(&TestPoint2D::new(0.0, 0.0), 1e-6, &mut aSqDist);
    assert_eq!(aResult.len(), THE_N as usize);
    near!(aSqDist, 18.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__AllSamePoints_CopyMove() {
    const THE_N: i32 = 100;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(2.0, 8.0);
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    let mut aCopy: KDTree<TestPoint2D, 2> = aTree.clone();
    assert_eq!(aCopy.len(), ((THE_N) as usize));
    let mut aSqDist: f64 = 0.0;
    aCopy.nearest_point(&TestPoint2D::new(2.0, 8.0), Some(&mut aSqDist));
    near!(aSqDist, 0.0, 1e-15);

    let mut aMoved: KDTree<TestPoint2D, 2> = std::mem::take(&mut aCopy);
    assert_eq!(aMoved.len(), ((THE_N) as usize));
    assert!(aCopy.is_empty());
    aMoved.nearest_point(&TestPoint2D::new(2.0, 8.0), Some(&mut aSqDist));
    near!(aSqDist, 0.0, 1e-15);
}

#[test]
fn NCollection_KDTreeRadiiTest__AllSamePoints_ContainingSearch() {
    const THE_N: i32 = 100;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(5.0, 5.0);
        aRadii[(i) as usize] = 2.0;
    }
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);

    let aResult = aTree.containing_search(&TestPoint2D::new(5.0, 5.0));
    assert_eq!(aResult.len(), THE_N as usize);

    let aResult2 = aTree.containing_search(&TestPoint2D::new(6.0, 5.0));
    assert_eq!(aResult2.len(), THE_N as usize);

    let aResult3 = aTree.containing_search(&TestPoint2D::new(15.0, 5.0));
    assert_eq!(aResult3.len(), 0);
}

#[test]
fn NCollection_KDTreeRadiiTest__AllSamePoints_NearestWeighted() {
    const THE_N: i32 = 100;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(5.0, 5.0);
        aRadii[(i) as usize] = 3.0;
    }
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);
    let mut aGap: f64 = 0.0;

    let mut aIdx: usize = aTree.nearest_weighted(&TestPoint2D::new(5.0, 5.0), Some(&mut aGap));
    assert_ne!(aIdx, 0usize);
    near!(aGap, -3.0, 1e-10);

    aTree.nearest_weighted(&TestPoint2D::new(15.0, 5.0), Some(&mut aGap));
    near!(aGap, 7.0, 1e-10);
}

#[test]
fn NCollection_KDTreeRadiiTest__AllSamePoints_VaryingRadii() {
    const THE_N: i32 = 50;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    let mut aRadii = vec![0.0_f64; THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(0.0, 0.0);
        aRadii[(i) as usize] = ((i + 1) as f64);
    }
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..THE_N as usize], &aRadii[..THE_N as usize]);

    let mut aGap: f64 = 0.0;
    let mut aIdx: usize = aTree.nearest_weighted(&TestPoint2D::new(10.0, 0.0), Some(&mut aGap));
    near!(aGap, -40.0, 1e-10);
    near!(aTree.radius(aIdx).unwrap(), 50.0, 1e-15);

    let aResult = aTree.containing_search(&TestPoint2D::new(10.0, 0.0));
    assert_eq!(aResult.len(), 41);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint_QueryOnEveryPoint() {
    const THE_N: i32 = 20;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] = TestPoint2D::new(((i * 3) as f64), ((i * 7 % 11) as f64));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);

    for i in (0)..(THE_N) {
        let mut aSqDist: f64 = 0.0;
        aTree.nearest_point(&aPoints[(i) as usize], Some(&mut aSqDist));
        near!(aSqDist, 0.0, 1e-15);
    }
}
