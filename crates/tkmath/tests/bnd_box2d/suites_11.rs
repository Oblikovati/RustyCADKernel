#[test]
fn Bnd_Box2dTest__Set_Point() {
    let mut aBox = BoundingBox2::default();
    aBox.set_point(Point2::new(5.0, 7.0));

    let [aXmin, aYmin, aXmax, aYmax] = aBox.get().unwrap();
    assert_eq!(aXmin, 5.0);
    assert_eq!(aYmin, 7.0);
    assert_eq!(aXmax, 5.0);
    assert_eq!(aYmax, 7.0);
}

#[test]
fn Bnd_Box2dTest__Add_Point() {
    let mut aBox = BoundingBox2::default();
    aBox.add_point(Point2::new(1.0, 2.0));
    aBox.add_point(Point2::new(10.0, 20.0));

    let [aXmin, aYmin, aXmax, aYmax] = aBox.get().unwrap();
    assert_eq!(aXmin, 1.0);
    assert_eq!(aYmin, 2.0);
    assert_eq!(aXmax, 10.0);
    assert_eq!(aYmax, 20.0);
}

#[test]
fn Bnd_Box2dTest__Add_Box() {
    let mut aBox1 = BoundingBox2::default();
    aBox1.update_bounds([0.0, 0.0], [5.0, 5.0]);
    let mut aBox2 = BoundingBox2::default();
    aBox2.update_bounds([3.0, 3.0], [10.0, 10.0]);
    aBox1.add_box(&aBox2);

    let [aXmin, aYmin, aXmax, aYmax] = aBox1.get().unwrap();
    assert_eq!(aXmin, 0.0);
    assert_eq!(aYmin, 0.0);
    assert_eq!(aXmax, 10.0);
    assert_eq!(aYmax, 10.0);
}

#[test]
fn Bnd_Box2dTest__Add_VoidBox() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([1.0, 2.0], [5.0, 6.0]);
    let mut aVoidBox = BoundingBox2::default();
    aBox.add_box(&aVoidBox);

    let [aXmin, aYmin, aXmax, aYmax] = aBox.get().unwrap();
    assert_eq!(aXmin, 1.0);
    assert_eq!(aYmin, 2.0);
}

#[test]
fn Bnd_Box2dTest__Add_ToVoidBox() {
    let mut aVoidBox = BoundingBox2::default();
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([1.0, 2.0], [5.0, 6.0]);
    aVoidBox.add_box(&aBox);

    let [aXmin, aYmin, aXmax, aYmax] = aVoidBox.get().unwrap();
    assert_eq!(aXmin, 1.0);
    assert_eq!(aYmin, 2.0);
    assert_eq!(aXmax, 5.0);
    assert_eq!(aYmax, 6.0);
}

#[test]
fn Bnd_Box2dTest__OpenDirections() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([0.0, 0.0], [10.0, 10.0]);
    assert!(!(aBox.is_open_xmin()));
    assert!(!(aBox.is_open_xmax()));
    assert!(!(aBox.is_open_ymin()));
    assert!(!(aBox.is_open_ymax()));

    aBox.open_xmin();
    assert!(aBox.is_open_xmin());
    aBox.open_ymax();
    assert!(aBox.is_open_ymax());
    assert!(!(aBox.is_whole()));
}

#[test]
fn Bnd_Box2dTest__IsOut_Point() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([0.0, 0.0], [10.0, 10.0]);
    assert!(!(aBox.is_out_point(Point2::new(5.0, 5.0))));
    assert!(aBox.is_out_point(Point2::new(15.0, 5.0)));
    assert!(aBox.is_out_point(Point2::new(-1.0, 5.0)));
}

#[test]
fn Bnd_Box2dTest__IsOut_Box_Overlapping() {
    let mut aBox1 = BoundingBox2::default();
    aBox1.update_bounds([0.0, 0.0], [10.0, 10.0]);
    let mut aBox2 = BoundingBox2::default();
    aBox2.update_bounds([5.0, 5.0], [15.0, 15.0]);
    assert!(!(aBox1.is_out_box(&aBox2)));
}

#[test]
fn Bnd_Box2dTest__IsOut_Box_Separated() {
    let mut aBox1 = BoundingBox2::default();
    aBox1.update_bounds([0.0, 0.0], [5.0, 5.0]);
    let mut aBox2 = BoundingBox2::default();
    aBox2.update_bounds([10.0, 10.0], [15.0, 15.0]);
    assert!(aBox1.is_out_box(&aBox2));
}

#[test]
fn Bnd_Box2dTest__IsOut_Box_FastPath() {
    // Both non-void, non-whole, non-open -> fast path
    let mut aBox1 = BoundingBox2::default();
    aBox1.update_bounds([0.0, 0.0], [10.0, 10.0]);
    let mut aBox2 = BoundingBox2::default();
    aBox2.update_bounds([5.0, 5.0], [15.0, 15.0]);
    assert!(!(aBox1.is_out_box(&aBox2)));

    let mut aBox3 = BoundingBox2::default();
    aBox3.update_bounds([20.0, 20.0], [30.0, 30.0]);
    assert!(aBox1.is_out_box(&aBox3));
}
