#[test]
fn Bnd_BoxTest__DefaultConstructor() {
    let mut aBox = BoundingBox3::default();

    assert!(aBox.is_void());
    assert!(!(aBox.is_whole()));
    assert_eq!(0.0, aBox.gap());
}

#[test]
fn Bnd_BoxTest__PointConstructor() {
    let mut aMinPnt = Point3::new(1.0, 2.0, 3.0);
    let mut aMaxPnt = Point3::new(4.0, 5.0, 6.0);

    let mut aBox = BoundingBox3::new(aMinPnt, aMaxPnt);

    assert!(!(aBox.is_void()));
    assert!(!(aBox.is_whole()));

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(1.0, aXmin);
    assert_eq!(2.0, aYmin);
    assert_eq!(3.0, aZmin);
    assert_eq!(4.0, aXmax);
    assert_eq!(5.0, aYmax);
    assert_eq!(6.0, aZmax);
}

//=================================================================================================
// Set and Update Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__SetWithPoint() {
    let mut aBox = BoundingBox3::default();
    let mut aPnt = Point3::new(1.5, 2.5, 3.5);

    aBox.set_point(aPnt);

    assert!(!(aBox.is_void()));

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(1.5, aXmin);
    assert_eq!(1.5, aXmax);
    assert_eq!(2.5, aYmin);
    assert_eq!(2.5, aYmax);
    assert_eq!(3.5, aZmin);
    assert_eq!(3.5, aZmax);
}

#[test]
fn Bnd_BoxTest__SetWithPointAndDirection() {
    let mut aBox = BoundingBox3::default();
    let mut aPnt = Point3::new(1.0, 2.0, 3.0);
    let mut aDir = Direction3::X;

    aBox.set_ray(aPnt, aDir);

    assert!(!(aBox.is_void()));
    assert!(aBox.is_open_xmax());
    assert!(!(aBox.is_open_xmin()));
}

#[test]
fn Bnd_BoxTest__UpdateWithBounds() {
    let mut aBox = BoundingBox3::default();

    aBox.update_bounds([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);

    assert!(!(aBox.is_void()));

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(1.0, aXmin);
    assert_eq!(2.0, aYmin);
    assert_eq!(3.0, aZmin);
    assert_eq!(4.0, aXmax);
    assert_eq!(5.0, aYmax);
    assert_eq!(6.0, aZmax);
}

#[test]
fn Bnd_BoxTest__UpdateWithPoint() {
    let mut aBox = BoundingBox3::default();

    // First point establishes the box
    aBox.add_point(Point3::new(1.0, 2.0, 3.0));

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(1.0, aXmin);
    assert_eq!(1.0, aXmax);

    // Second point expands the box
    aBox.add_point(Point3::new(0.5, 2.5, 3.5));
    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(0.5, aXmin);
    assert_eq!(1.0, aXmax);
    assert_eq!(2.0, aYmin);
    assert_eq!(2.5, aYmax);
}

#[test]
fn Bnd_BoxTest__UpdateExpansion() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    // Update with bounds that expand the box
    aBox.update_bounds([-1.0, -1.0, -1.0], [2.0, 2.0, 2.0]);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(-1.0, aXmin);
    assert_eq!(-1.0, aYmin);
    assert_eq!(-1.0, aZmin);
    assert_eq!(2.0, aXmax);
    assert_eq!(2.0, aYmax);
    assert_eq!(2.0, aZmax);
}

//=================================================================================================
// Gap and Tolerance Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__GapOperations() {
    let mut aBox = BoundingBox3::default();

    assert_eq!(0.0, aBox.gap());

    aBox.set_gap(0.5);
    assert_eq!(0.5, aBox.gap());

    aBox.enlarge(0.3);
    assert_eq!(0.5, aBox.gap());

    aBox.enlarge(0.8);
    assert_eq!(0.8, aBox.gap());

    aBox.enlarge(-1.0);
    assert_eq!(1.0, aBox.gap());
}

//=================================================================================================
// Corner Methods Tests
//=================================================================================================
