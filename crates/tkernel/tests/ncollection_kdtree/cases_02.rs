#[test]
fn NCollection_KDTreeTest__NearestPoint3D_BasicSearch() {
    let mut aPoints = [
        TestPoint3D::new(0, 0, 0),
        TestPoint3D::new(1, 1, 1),
        TestPoint3D::new(2, 2, 2),
        TestPoint3D::new(3, 3, 3),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..4 as usize]);
    let mut aQuery = TestPoint3D::new(1.1, 0.9, 1.0);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 1.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 1.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Z, 1.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_GridPoints() {
    let mut aPoints = vec![TestPoint3D::default(); 27 as usize];
    let mut k: i32 = 0;
    for i in (0)..(3) {
        for j in (0)..(3) {
            for l in (0)..(3) {
                aPoints[{
                    let previous = k;
                    k += 1;
                    previous as usize
                }] = TestPoint3D::new(((i) as f64), ((j) as f64), ((l) as f64));
            }
        }
    }
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..27 as usize]);
    let mut aQuery = TestPoint3D::new(1.1, 2.2, 0.9);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 1.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Y, 2.0, 1e-15);
    near!((*aTree.point(aIdx).unwrap()).Z, 1.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_Diagonal() {
    let mut aPoints = vec![TestPoint3D::default(); 10 as usize];
    for i in (0)..(10) {
        let mut aVal: f64 = ((i) as f64);
        aPoints[(i) as usize] = TestPoint3D::new(aVal, aVal, aVal);
    }
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..10 as usize]);
    let mut aQuery = TestPoint3D::new(4.6, 4.6, 4.6);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    near!((*aTree.point(aIdx).unwrap()).X, 5.0, 1e-15);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_Clustered() {
    let mut aPoints = [
        TestPoint3D::new(0.1, 0.1, 0.1),
        TestPoint3D::new(-0.1, -0.1, -0.1),
        TestPoint3D::new(0.2, 0.0, 0.0),
        TestPoint3D::new(100.0, 100.0, 100.0),
        TestPoint3D::new(99.9, 100.1, 100.0),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut aQuery = TestPoint3D::new(99.0, 100.0, 100.0);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, None);
    assert!((*aTree.point(aIdx).unwrap()).X > 50.0);
}

#[test]
fn NCollection_KDTreeTest__NearestPoint3D_EquidistantPoints() {
    let mut aPoints = [
        TestPoint3D::new(1, 0, 0),
        TestPoint3D::new(-1, 0, 0),
        TestPoint3D::new(0, 1, 0),
        TestPoint3D::new(0, -1, 0),
        TestPoint3D::new(0, 0, 1),
        TestPoint3D::new(0, 0, -1),
    ];
    let mut aTree: KDTree<TestPoint3D, 3> = KDTree::default();
    aTree.build(&(&aPoints)[..6 as usize]);
    let mut aSqDist: f64 = 0.0;
    let mut aQuery = TestPoint3D::new(0, 0, 0);
    let mut aIdx: usize = aTree.nearest_point(&aQuery, Some(&mut aSqDist));
    assert_ne!(aIdx, 0usize);
    near!(aSqDist, 1.0, 1e-10);
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_K1() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(2, 2),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(1.1, 1.1);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 1 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 1usize);
    near!(
        (*aTree.point((*anIndices.value(1).unwrap())).unwrap()).X,
        1.0,
        1e-15
    );
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_K3() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 0),
        TestPoint2D::new(2, 0),
        TestPoint2D::new(10, 0),
        TestPoint2D::new(20, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(0.5, 0.0);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 3 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 3usize);

    let mut aXCoords: TestSet<f64> = TestSet::default();
    for i in (1)..=(3) {
        aXCoords.insert((*aTree.point((*anIndices.value(i).unwrap())).unwrap()).X);
    }
    assert!(aXCoords.count(0.0) > 0);
    assert!(aXCoords.count(1.0) > 0);
    assert!(aXCoords.count(2.0) > 0);
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_KEqualsN() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(1, 1),
        TestPoint2D::new(2, 2),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..3 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 3 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 3usize);
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_KGreaterThanN() {
    let mut aPoints = [TestPoint2D::new(0, 0), TestPoint2D::new(1, 1)];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..2 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 10 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 2usize);
}

#[test]
fn NCollection_KDTreeTest__KNearest2D_SortedByDistance() {
    let mut aPoints = [
        TestPoint2D::new(0, 0),
        TestPoint2D::new(3, 0),
        TestPoint2D::new(1, 0),
        TestPoint2D::new(2, 0),
        TestPoint2D::new(4, 0),
    ];
    let mut aTree: KDTree<TestPoint2D, 2> = KDTree::default();
    aTree.build(&(&aPoints)[..5 as usize]);
    let mut anIndices: Array1<usize> = Array1::with_bounds(1, 0).unwrap();
    let mut aDists: Array1<f64> = Array1::with_bounds(1, 0).unwrap();
    let mut aQuery = TestPoint2D::new(0, 0);
    let mut aCount: usize =
        aTree.k_nearest_points(&aQuery, 5 as usize, &mut anIndices, &mut aDists);
    assert_eq!(aCount, 5usize);

    for i in (1)..((aCount) as i32) {
        assert!((*aDists.value(i).unwrap()) <= (*aDists.value(i + 1).unwrap()));
    }
}
