#[test]
fn Bnd_BoxTest__AddBox() {
    let mut aBox1 = BoundingBox3::default();
    aBox1.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    aBox1.set_gap(0.1);

    let mut aBox2 = BoundingBox3::default();
    aBox2.update_bounds([0.5, 0.5, 0.5], [2.0, 2.0, 2.0]);
    aBox2.set_gap(0.2);

    aBox1.add_box(&aBox2);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox1.get().unwrap();

    // Get() returns bounds including gap, so expect gap-adjusted values
    assert_eq!(-0.2, aXmin);
    assert_eq!(-0.2, aYmin);
    assert_eq!(-0.2, aZmin);
    assert_eq!(2.2, aXmax);
    assert_eq!(2.2, aYmax);
    assert_eq!(2.2, aZmax);
    assert_eq!(0.2, aBox1.gap());
}

#[test]
fn Bnd_BoxTest__AddVoidBox() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);

    let mut aVoidBox = BoundingBox3::default();
    aBox.add_box(&aVoidBox);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(1.0, aXmin);
    assert_eq!(2.0, aXmax);
}

#[test]
fn Bnd_BoxTest__AddToVoidBox() {
    let mut aVoidBox = BoundingBox3::default();
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);

    aVoidBox.add_box(&aBox);

    assert!(!(aVoidBox.is_void()));

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aVoidBox.get().unwrap();

    assert_eq!(1.0, aXmin);
    assert_eq!(2.0, aXmax);
}

#[test]
fn Bnd_BoxTest__AddPoint() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    let mut aPnt = Point3::new(2.0, -1.0, 0.5);
    aBox.add_point(aPnt);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(0.0, aXmin);
    assert_eq!(-1.0, aYmin);
    assert_eq!(0.0, aZmin);
    assert_eq!(2.0, aXmax);
    assert_eq!(1.0, aYmax);
    assert_eq!(1.0, aZmax);
}

#[test]
fn Bnd_BoxTest__AddDirection() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    let mut aDir = Direction3::X;
    aBox.add_direction(aDir);

    assert!(aBox.is_open_xmax());
    assert!(!(aBox.is_open_xmin()));
    assert!(!(aBox.is_open_ymax()));
    assert!(!(aBox.is_open_ymin()));
}

#[test]
fn Bnd_BoxTest__AddPointWithDirection() {
    let mut aBox = BoundingBox3::default();
    let mut aPnt = Point3::new(1.0, 2.0, 3.0);
    let mut aDir = Direction3::X.reversed();

    aBox.add_ray(aPnt, aDir);

    assert!(!(aBox.is_void()));
    assert!(aBox.is_open_xmin());
    assert!(!(aBox.is_open_xmax()));
}

//=================================================================================================
// IsOut Methods Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__IsOutPoint() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);

    let mut anInsidePnt = Point3::new(1.0, 1.0, 1.0);
    let mut anOutsidePnt = Point3::new(3.0, 1.0, 1.0);
    let mut aBoundaryPnt = Point3::new(2.0, 2.0, 2.0);

    assert!(!(aBox.is_out_point(anInsidePnt)));
    assert!(aBox.is_out_point(anOutsidePnt));
    assert!(!(aBox.is_out_point(aBoundaryPnt)));
}

#[test]
fn Bnd_BoxTest__IsOutPointWithGap() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
    aBox.set_gap(0.1);

    let mut aNearBoundaryPnt = Point3::new(2.05, 1.0, 1.0);
    let mut aFarBoundaryPnt = Point3::new(2.15, 1.0, 1.0);

    assert!(!(aBox.is_out_point(aNearBoundaryPnt)));
    assert!(aBox.is_out_point(aFarBoundaryPnt));
}
