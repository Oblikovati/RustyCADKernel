#[test]
fn Bnd_Box2dTest__IsOut_Box_FastPath_WithGap() {
    // Boxes separated by 1.0, but gap bridges it
    let mut aBox1 = BoundingBox2::default();
    aBox1.update_bounds([0.0, 0.0], [5.0, 5.0]);
    aBox1.set_gap(1.0);
    let mut aBox2 = BoundingBox2::default();
    aBox2.update_bounds([6.0, 0.0], [10.0, 5.0]);
    // Gap from box1 = 1.0, gap from box2 = 0.0, total = 1.0
    // Xmin1 - Xmax2 = 0 - 10 = -10 (not > 1.0) -> not out in X
    // Xmin2 - Xmax1 = 6 - 5 = 1 (not > 1.0, equal) -> not out
    assert!(!(aBox1.is_out_box(&aBox2)));

    // Now increase separation
    let mut aBox3 = BoundingBox2::default();
    aBox3.update_bounds([7.0, 0.0], [10.0, 5.0]);
    // Xmin3 - Xmax1 = 7 - 5 = 2 > 1.0 -> out
    assert!(aBox1.is_out_box(&aBox3));
}

#[test]
fn Bnd_Box2dTest__IsOut_VoidBox() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([0.0, 0.0], [10.0, 10.0]);
    let mut aVoid = BoundingBox2::default();
    assert!(aBox.is_out_box(&aVoid));
    assert!(aVoid.is_out_box(&aBox));
}

#[test]
fn Bnd_Box2dTest__IsOut_WholeBox() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([0.0, 0.0], [10.0, 10.0]);
    let mut aWhole = BoundingBox2::default();
    aWhole.set_whole();
    assert!(!(aBox.is_out_box(&aWhole)));
    assert!(!(aWhole.is_out_box(&aBox)));
}

#[test]
fn Bnd_Box2dTest__IsOut_OpenBox() {
    let mut aBox1 = BoundingBox2::default();
    aBox1.update_bounds([0.0, 0.0], [10.0, 10.0]);
    aBox1.open_xmin(); // extends to -infinity in X

    let mut aBox2 = BoundingBox2::default();
    aBox2.update_bounds([-100.0, 0.0], [-50.0, 10.0]);

    // aBox1 is open in Xmin, so extends to -infinity: should overlap
    assert!(!(aBox1.is_out_box(&aBox2)));
}

#[test]
fn Bnd_Box2dTest__IsOut_Line() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([0.0, 0.0], [10.0, 10.0]);

    // Line that intersects
    let mut aLinInside = Line2::new(Point2::new(5.0, -5.0), Direction2::new(0.0, 1.0).unwrap());
    assert!(!(aBox.is_out_line(aLinInside)));

    // Line that doesn't intersect
    let mut aLinOutside = Line2::new(Point2::new(20.0, -5.0), Direction2::new(0.0, 1.0).unwrap());
    assert!(aBox.is_out_line(aLinOutside));
}

#[test]
fn Bnd_Box2dTest__IsOut_Segment() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([0.0, 0.0], [10.0, 10.0]);

    // Segment that crosses
    assert!(!(aBox.is_out_segment(Point2::new(5.0, -5.0), Point2::new(5.0, 15.0))));

    // Segment fully outside
    assert!(aBox.is_out_segment(Point2::new(20.0, 0.0), Point2::new(20.0, 10.0)));
}

#[test]
fn Bnd_Box2dTest__Transformed() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([0.0, 0.0], [10.0, 10.0]);

    let mut aTrsf = Transform2::default();
    aTrsf = Transform2::translation([5.0, 5.0]);
    let mut aTransformed = aBox.transformed(aTrsf);

    let [aXmin, aYmin, aXmax, aYmax] = aTransformed.get().unwrap();
    assert_eq!(aXmin, 5.0);
    assert_eq!(aYmin, 5.0);
    assert_eq!(aXmax, 15.0);
    assert_eq!(aYmax, 15.0);
}

#[test]
fn Bnd_Box2dTest__SquareExtent() {
    let mut aBox = BoundingBox2::default();
    aBox.update_bounds([0.0, 0.0], [3.0, 4.0]);
    // Diagonal^2 = 3^2 + 4^2 = 25
    assert_eq!(aBox.square_extent(), 25.0);
}

#[test]
fn Bnd_Box2dTest__SquareExtent_Void() {
    let mut aBox = BoundingBox2::default();
    assert_eq!(aBox.square_extent(), 0.0);
}

#[test]
fn Bnd_Box2dTest__Add_Direction() {
    let mut aBox = BoundingBox2::default();
    aBox.set_point(Point2::new(5.0, 5.0));
    aBox.add_direction(Direction2::new(1.0, 0.0).unwrap());
    assert!(aBox.is_open_xmax());
    assert!(!(aBox.is_open_xmin()));
}
