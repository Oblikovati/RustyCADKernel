#[test]
fn NCollection_KDTreeTest__RangeSearch2D_ValidIndices() {
    const THE_N: i32 = 30;
    let mut aRng = TestRandom::new(444);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-10, 10), aRng.next_double(-10, 10));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let aResult = aTree.range_search(&TestPoint2D::new(0, 0), 15.0);
    for i in (0)..(aResult.len()) {
        assert!(aResult[(i) as usize] >= 1usize);
        assert!(aResult[(i) as usize] <= ((THE_N) as usize));
    }
}

#[test]
fn NCollection_KDTreeTest__RangeSearch3D_BruteForce() {
    const THE_N: i32 = 150;
    let mut aRng = TestRandom::new(888);
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
    for q in (0)..(15) {
        let mut aQuery = {
            let coordinate_2 = aRng.next_double(-60, 60);
            let coordinate_1 = aRng.next_double(-60, 60);
            let coordinate_0 = aRng.next_double(-60, 60);
            TestPoint3D::new(coordinate_0, coordinate_1, coordinate_2)
        };
        let mut aRadius: f64 = aRng.next_double(10, 40);
        let aKdResult = aTree.range_search(&aQuery, aRadius);
        let mut aBfCount: usize = 0;
        let mut aRadiusSq: f64 = aRadius * aRadius;
        for i in (0)..(THE_N) {
            if (sqDist3D(aQuery, aPoints[(i) as usize]) <= aRadiusSq) {
                aBfCount += 1;
            }
        }
        assert_eq!(aKdResult.len(), aBfCount);
    }
}

#[test]
fn NCollection_KDTreeTest__RangeSearch2D_VerifyPointsInRange() {
    const THE_N: i32 = 50;
    let mut aRng = TestRandom::new(111);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-20, 20), aRng.next_double(-20, 20));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aRadius: f64 = 10.0;
    let aResult = aTree.range_search(&aQuery, aRadius);

    for i in (0)..(aResult.len()) {
        let mut aDist: f64 = sqDist2D(aQuery, (*aTree.point(aResult[(i) as usize]).unwrap()));
        assert!(aDist <= aRadius * aRadius + 1e-10);
    }
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_ZeroVolumeBox() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(5, 5),
        TestPoint2D::new(5, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);

    let aResult = aTree.box_search(&TestPoint2D::new(5, 5), &TestPoint2D::new(5, 5));
    assert_eq!(aResult.len(), 1usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_DegenerateLineBox() {
    let mut aPoints = [
        TestPoint2D::new(0, 5),
        TestPoint2D::new(1, 5),
        TestPoint2D::new(2, 5),
        TestPoint2D::new(3, 5),
        TestPoint2D::new(2, 6),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let aResult = aTree.box_search(&TestPoint2D::new(0, 5), &TestPoint2D::new(3, 5));
    assert_eq!(aResult.len(), 4usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_NegativeQuadrant() {
    let mut aPoints = [
        TestPoint2D::new(-10, -10),
        TestPoint2D::new(-5, -5),
        TestPoint2D::new(-1, -1),
        TestPoint2D::new(5, 5),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let aResult = aTree.box_search(&TestPoint2D::new(-11, -11), &TestPoint2D::new(-4, -4));
    assert_eq!(aResult.len(), 2usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_NarrowBox() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(0.001, 5),
        TestPoint2D::new(0.002, 10),
        TestPoint2D::new(1, 5),
        TestPoint2D::new(-1, 5),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let aResult = aTree.box_search(&TestPoint2D::new(-0.01, -100), &TestPoint2D::new(0.01, 100));
    assert_eq!(aResult.len(), 3usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_SinglePointInOut() {
    let mut aPoint = TestPoint2D::new(5, 5);
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(std::slice::from_ref(&aPoint))[..1 as usize]);
    let aResult1 = aTree.box_search(&TestPoint2D::new(4, 4), &TestPoint2D::new(6, 6));
    assert_eq!(aResult1.len(), 1usize);
    let aResult2 = aTree.box_search(&TestPoint2D::new(6, 6), &TestPoint2D::new(7, 7));
    assert_eq!(aResult2.len(), 0usize);
}

#[test]
fn NCollection_KDTreeTest__BoxSearch3D_BruteForce() {
    const THE_N: i32 = 150;
    let mut aRng = TestRandom::new(246);
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
    for q in (0)..(15) {
        let mut aX1: f64 = aRng.next_double(-60, 60);
        let mut aY1: f64 = aRng.next_double(-60, 60);
        let mut aZ1: f64 = aRng.next_double(-60, 60);
        let mut aX2: f64 = aX1 + aRng.next_double(5, 40);
        let mut aY2: f64 = aY1 + aRng.next_double(5, 40);
        let mut aZ2: f64 = aZ1 + aRng.next_double(5, 40);
        let aKdResult = aTree.box_search(
            &TestPoint3D::new(aX1, aY1, aZ1),
            &TestPoint3D::new(aX2, aY2, aZ2),
        );
        let mut aBfCount: usize = 0;
        for i in (0)..(THE_N) {
            if (aPoints[(i) as usize].X >= aX1
                && aPoints[(i) as usize].X <= aX2
                && aPoints[(i) as usize].Y >= aY1
                && aPoints[(i) as usize].Y <= aY2
                && aPoints[(i) as usize].Z >= aZ1
                && aPoints[(i) as usize].Z <= aZ2)
            {
                aBfCount += 1;
            }
        }
        assert_eq!(aKdResult.len(), aBfCount);
    }
}

#[test]
fn NCollection_KDTreeTest__BoxSearch2D_VerifyPointsInBox() {
    const THE_N: i32 = 50;
    let mut aRng = TestRandom::new(222);
    let mut aPoints = vec![TestPoint2D::default(); THE_N as usize];
    for i in (0)..(THE_N) {
        aPoints[(i) as usize] =
            TestPoint2D::new(aRng.next_double(-20, 20), aRng.next_double(-20, 20));
    }
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aMin = TestPoint2D::new(-5, -5);
    let mut aMax = TestPoint2D::new(5, 5);
    let aResult = aTree.box_search(&aMin, &aMax);
    for i in (0)..(aResult.len()) {
        let aP = (*aTree.point(aResult[(i) as usize]).unwrap());
        assert!(aP.X >= -5.0);
        assert!(aP.X <= 5.0);
        assert!(aP.Y >= -5.0);
        assert!(aP.Y <= 5.0);
    }
}
