#[test]
fn NCollection_KDTreeTest__NearestPoint2D_ThinStrip() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(10, 0.001),
        TestPoint2D::new(20, -0.001),
        TestPoint2D::new(30, 0.002),
        TestPoint2D::new(40, -0.002),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut aSqDist: f64 = 0.0;
    aTree.nearest_point(&TestPoint2D::new(15, 0), Some(&mut aSqDist));

    let mut aBfBest: f64 = f64::MAX;
    for i in (0)..(5) {
        let mut aDist: f64 = sqDist2D(TestPoint2D::new(15, 0), aPoints[(i) as usize]);
        if (aDist < aBfBest) {
            aBfBest = aDist;
        }
    }
    near!(aSqDist, aBfBest, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_SameXDifferentY() {
    let mut aPoints = [
        TestPoint2D::new(5, 0),
        TestPoint2D::new(5, 1),
        TestPoint2D::new(5, 2),
        TestPoint2D::new(5, 3),
        TestPoint2D::new(5, 4),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(5, 2.3), None);
    near!((*aTree.point(aIdx).unwrap()).X, 5.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 2.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint2D_SameYDifferentX() {
    let mut aPoints = [
        TestPoint2D::new(0, 5),
        TestPoint2D::new(1, 5),
        TestPoint2D::new(2, 5),
        TestPoint2D::new(3, 5),
        TestPoint2D::new(4, 5),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint2D::new(3.7, 5), None);
    near!((*aTree.point(aIdx).unwrap()).X, 4.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 5.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_AllSameXDiffYZ() {
    let mut aPoints = [
        TestPoint3D::new(5, 0, 0),
        TestPoint3D::new(5, 10, 0),
        TestPoint3D::new(5, 0, 10),
        TestPoint3D::new(5, 5, 5),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint3D::new(5, 5.1, 4.9), None);
    near!((*aTree.point(aIdx).unwrap()).X, 5.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 5.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Z, 5.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_SphereSurface() {
    const THE_N: i32 = 8;
    let mut aPoints = vec![TestPoint3D::default(); THE_N as usize];
    aPoints[(0) as usize] = TestPoint3D::new(10, 0, 0);
    aPoints[(1) as usize] = TestPoint3D::new(-10, 0, 0);
    aPoints[(2) as usize] = TestPoint3D::new(0, 10, 0);
    aPoints[(3) as usize] = TestPoint3D::new(0, -10, 0);
    aPoints[(4) as usize] = TestPoint3D::new(0, 0, 10);
    aPoints[(5) as usize] = TestPoint3D::new(0, 0, -10);
    aPoints[(6) as usize] = TestPoint3D::new(5.77, 5.77, 5.77);
    aPoints[(7) as usize] = TestPoint3D::new(-5.77, -5.77, -5.77);
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint3D::new(9.5, 0.5, 0.5), None);
    near!((*aTree.point(aIdx).unwrap()).X, 10.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_MixedSignCoordinates() {
    let mut aPoints = [
        TestPoint3D::new(-1, 2, -3),
        TestPoint3D::new(4, -5, 6),
        TestPoint3D::new(-7, 8, -9),
        TestPoint3D::new(10, -11, 12),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint3D::new(-1.1, 2.1, -3.1), None);
    near!((*aTree.point(aIdx).unwrap()).X, -1.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 2.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Z, -3.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_Collinear() {
    let mut aPoints = [
        TestPoint3D::new(0, 0, 0),
        TestPoint3D::new(1, 0, 0),
        TestPoint3D::new(2, 0, 0),
        TestPoint3D::new(3, 0, 0),
        TestPoint3D::new(4, 0, 0),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint3D::new(2.4, 0, 0), None);
    near!((*aTree.point(aIdx).unwrap()).X, 2.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_LargeGrid() {
    const THE_N: i32 = 125;
    let mut aPoints = vec![TestPoint3D::default(); THE_N as usize];
    let mut k: i32 = 0;
    for i in (0)..(5) {
        for j in (0)..(5) {
            for l in (0)..(5) {
                aPoints[{
                    let previous = k;
                    k += 1;
                    previous as usize
                }] = TestPoint3D::new(((i * 2) as f64), ((j * 2) as f64), ((l * 2) as f64));
            }
        }
    }
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..THE_N as usize]);
    let mut aIdx: usize = aTree.nearest_point(&TestPoint3D::new(3.1, 5.1, 7.1), None);

    near!((*aTree.point(aIdx).unwrap()).X, 4.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 6.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Z, 8.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_K2() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 0),
        TestPoint2D::new(5, 0),
        TestPoint2D::new(10, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aCount: usize = aTree.k_nearest_points(
        &TestPoint2D::new(0.5, 0),
        2 as usize,
        &mut anIndices,
        &mut aDists,
    );
    assert_eq!(aCount, 2usize);
    let mut aXCoords: TestSet<f64> = TestSet::default();
    for i in (1)..=(2) {
        aXCoords.insert((*aTree.point((*anIndices.value(i).unwrap())).unwrap()).X);
    }
    assert!(aXCoords.count(0.0) > 0);
    assert!(aXCoords.count(1.0) > 0);
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_K0() {
    let mut aPoints = [TestPoint2D::new(0, 0), TestPoint2D::new(1, 1)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aCount: usize = aTree.k_nearest_points(
        &TestPoint2D::new(0, 0),
        0 as usize,
        &mut anIndices,
        &mut aDists,
    );
    assert_eq!(aCount, 0usize);
}
