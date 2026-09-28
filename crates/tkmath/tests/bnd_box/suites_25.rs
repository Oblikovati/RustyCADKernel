#[test]
fn Bnd_BoxTest__IsOutVoidBox() {
    let mut aVoidBox = BoundingBox3::default();
    let mut anAnyPnt = Point3::new(1.0, 2.0, 3.0);

    assert!(aVoidBox.is_out_point(anAnyPnt));
}

#[test]
fn Bnd_BoxTest__IsOutWholeBox() {
    let mut aWholeBox = BoundingBox3::default();
    aWholeBox.set_whole();

    let mut anAnyPnt = Point3::new(1000.0, 2000.0, 3000.0);

    assert!(!(aWholeBox.is_out_point(anAnyPnt)));
}

#[test]
fn Bnd_BoxTest__IsOutBox() {
    let mut aBox1 = BoundingBox3::default();
    aBox1.update_bounds([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);

    let mut anOverlappingBox = BoundingBox3::default();
    anOverlappingBox.update_bounds([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);

    let mut aSeparateBox = BoundingBox3::default();
    aSeparateBox.update_bounds([3.0, 3.0, 3.0], [4.0, 4.0, 4.0]);

    let mut aTouchingBox = BoundingBox3::default();
    aTouchingBox.update_bounds([2.0, 2.0, 2.0], [3.0, 3.0, 3.0]);

    assert!(!(aBox1.is_out_box(&anOverlappingBox)));
    assert!(aBox1.is_out_box(&aSeparateBox));
    assert!(!(aBox1.is_out_box(&aTouchingBox)));
}

#[test]
fn Bnd_BoxTest__IsOutPlane() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

    // Plane that intersects the box
    let mut anIntersectingPln = Plane3::new(Point3::new(0.0, 0.0, 0.0), Direction3::X).unwrap();

    // Plane that doesn't intersect the box
    let mut aSeparatePln = Plane3::new(Point3::new(2.0, 0.0, 0.0), Direction3::X).unwrap();

    assert!(!(aBox.is_out_plane(anIntersectingPln).unwrap()));
    assert!(aBox.is_out_plane(aSeparatePln).unwrap());
}

#[test]
fn Bnd_BoxTest__IsOutLine() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);

    // Line that passes through the box
    let mut anIntersectingLin = Line3::new(Point3::new(-1.0, 1.0, 1.0), Direction3::X);

    // Line that misses the box
    let mut aSeparateLin = Line3::new(Point3::new(-1.0, 3.0, 1.0), Direction3::X);

    assert!(!(aBox.is_out_line(anIntersectingLin)));
    assert!(aBox.is_out_line(aSeparateLin));
}

//=================================================================================================
// Distance Method Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__Distance() {
    let mut aBox1 = BoundingBox3::default();
    aBox1.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    let mut aBox2 = BoundingBox3::default();
    aBox2.update_bounds([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]); // 1 unit away in X direction

    let aDistance: f64 = aBox1.distance(&aBox2);
    assert_eq!(1.0, aDistance);

    // Overlapping boxes
    let mut anOverlappingBox = BoundingBox3::default();
    anOverlappingBox.update_bounds([0.5, 0.5, 0.5], [1.5, 1.5, 1.5]);

    let anOverlapDistance: f64 = aBox1.distance(&anOverlappingBox);
    assert_eq!(0.0, anOverlapDistance);
}

//=================================================================================================
// Open/Close Methods Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__OpenCloseMethods() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    assert!(!(aBox.is_open()));

    aBox.open_xmin();
    assert!(aBox.is_open_xmin());
    assert!(aBox.is_open());

    aBox.open_xmax();
    aBox.open_ymin();
    aBox.open_ymax();
    aBox.open_zmin();
    aBox.open_zmax();

    assert!(aBox.is_open_xmax());
    assert!(aBox.is_open_ymin());
    assert!(aBox.is_open_ymax());
    assert!(aBox.is_open_zmin());
    assert!(aBox.is_open_zmax());
    assert!(aBox.is_whole());
}

//=================================================================================================
// Void/Whole State Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__VoidWholeStates() {
    let mut aBox = BoundingBox3::default();

    assert!(aBox.is_void());
    assert!(!(aBox.is_whole()));

    aBox.set_whole();
    assert!(!(aBox.is_void()));
    assert!(aBox.is_whole());

    aBox.set_void();
    assert!(aBox.is_void());
    assert!(!(aBox.is_whole()));

    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    assert!(!(aBox.is_void()));
    assert!(!(aBox.is_whole()));

    assert!(aBox.has_finite_part());
}

//=================================================================================================
// Edge Cases and Error Conditions
//=================================================================================================
