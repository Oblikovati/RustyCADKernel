#[test]
fn NCollection_KDTreeTest__NearestPoints_CirclePoints() {
    const THE_N: i32 = 8;
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        let mut anAngle: f64 = 2.0 * 3.14159265358979323846 * (i as f64) / (THE_N as f64);
        aPoints[(i) as usize] = TestPoint2D::new(10.0 * (anAngle).cos(), 10.0 * (anAngle).sin());
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aSqDist: f64 = 0.0;

    let aResult = aTree.nearest_points(&TestPoint2D::new(0, 0), 1e-6, &mut aSqDist);
    assert_eq!(aResult.len(), ((THE_N) as usize));
    near!(aSqDist, 100.0, 1e-6);
}

#[test]
fn NCollection_KDTreeTest__NearestPoints_ValidIndices() {
    const THE_N: i32 = 50;
    let mut aRng = TestRandom::new(7070);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-10, 10), aRng.next_double(-10, 10));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aSqDist: f64 = 0.0;
    let aResult = aTree.nearest_points(&TestPoint2D::new(0, 0), 0.5, &mut aSqDist);
    assert!(aResult.len() >= 1usize);
    for i in (0)..(aResult.len()) {
        assert!(aResult[(i) as usize] >= 1usize);
        assert!(aResult[(i) as usize] <= ((THE_N) as usize));
    }
}

#[test]
fn NCollection_KDTreeRadiiTest__DefaultConstructor() {
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    assert!(aTree.is_empty());
    assert_eq!(aTree.len(), 0usize);
}

#[test]
fn NCollection_KDTreeRadiiTest__BuildFromCArray() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(5.0, 0.0),
        TestPoint2D::new(0.0, 5.0),
    ];
    let mut aRadii = [1.0, 2.0, 3.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    assert!(!aTree.is_empty());
    assert_eq!(aTree.len(), 3usize);
}

#[test]
fn NCollection_KDTreeRadiiTest__BuildFromArray1() {
    let mut aPoints: Array1<TestPoint2D> = Array1::with_bounds(1, 3).unwrap();
    aPoints.set_value(1, TestPoint2D::new(0.0, 0.0)).unwrap();
    aPoints.set_value(2, TestPoint2D::new(5.0, 0.0)).unwrap();
    aPoints.set_value(3, TestPoint2D::new(0.0, 5.0)).unwrap();
    let mut aRadii: Array1<f64> = Array1::with_bounds(1, 3).unwrap();
    aRadii.set_value(1, 1.0).unwrap();
    aRadii.set_value(2, 2.0).unwrap();
    aRadii.set_value(3, 3.0).unwrap();
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&aPoints.as_slice(), aRadii.as_slice());
    assert!(!aTree.is_empty());
    assert_eq!(aTree.len(), 3usize);
}

#[test]
fn NCollection_KDTreeRadiiTest__BuildEmptyArray() {
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&EMPTY_POINTS)[..0 as usize], &EMPTY_RADII[..0 as usize]);
    assert!(aTree.is_empty());
}

#[test]
fn NCollection_KDTreeRadiiTest__RadiusAccessor() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(5.0, 0.0),
        TestPoint2D::new(0.0, 5.0),
    ];
    let mut aRadii = [1.5, 2.5, 3.5];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);

    for i in (1)..=(3) {
        let mut aR: f64 = aTree.radius(i).unwrap();
        let mut aExpected: f64 = if i == 1 {
            1.5
        } else if i == 2 {
            2.5
        } else {
            3.5
        };
        near!(aR, aExpected, 1e-15);
    }
}

#[test]
fn NCollection_KDTreeRadiiTest__Clear() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(5.0, 0.0)];
    let mut aRadii = [1.0, 2.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);
    assert!(!aTree.is_empty());
    aTree.clear();
    assert!(aTree.is_empty());
    assert_eq!(aTree.len(), 0usize);
}

#[test]
fn NCollection_KDTreeRadiiTest__CopyConstructor() {
    let mut aPoints = [
        TestPoint2D::new(0.0, 0.0),
        TestPoint2D::new(5.0, 0.0),
        TestPoint2D::new(0.0, 5.0),
    ];
    let mut aRadii = [1.0, 2.0, 3.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..3 as usize], &aRadii[..3 as usize]);
    let mut aCopy: KDTree<TestPoint2D, 2, true> = aTree.clone();
    assert_eq!(aCopy.len(), 3usize);
    for i in (1)..=(3) {
        near!(aCopy.radius(i).unwrap(), aTree.radius(i).unwrap(), 1e-15);
    }

    let mut aQuery = TestPoint2D::new(2.0, 1.0);
    assert_eq!(
        aTree.nearest_weighted(&aQuery, None),
        aCopy.nearest_weighted(&aQuery, None)
    );
}

#[test]
fn NCollection_KDTreeRadiiTest__MoveConstructor() {
    let mut aPoints = [TestPoint2D::new(0.0, 0.0), TestPoint2D::new(5.0, 0.0)];
    let mut aRadii = [1.0, 2.0];
    let mut aTree: KDTree<TestPoint2D, 2, true> = KDTree::default();
    aTree.build_radii(&(&aPoints)[..2 as usize], &aRadii[..2 as usize]);
    let mut aMoved: KDTree<TestPoint2D, 2, true> = std::mem::take(&mut aTree);
    assert_eq!(aMoved.len(), 2usize);
    assert!(aTree.is_empty());
}
