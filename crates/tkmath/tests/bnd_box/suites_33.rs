#[test]
fn Bnd_BoxTest__EdgeCases() {
    let mut aBox = BoundingBox3::default();

    // Test with very large numbers
    let aLargeValue: f64 = 1e10;
    aBox.update_bounds(
        [-aLargeValue, -aLargeValue, -aLargeValue],
        [aLargeValue, aLargeValue, aLargeValue],
    );

    assert!(!(aBox.is_void()));

    // Test with very small differences
    let mut aSmallBox = BoundingBox3::default();
    aSmallBox.update_bounds([0.0, 0.0, 0.0], [1e-10, 1e-10, 1e-10]);

    assert!(!(aSmallBox.is_void()));
    assert!(aSmallBox.is_x_thin(1e-9));
}

#[test]
fn Bnd_BoxTest__TransformationWithOpenBox() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    aBox.open_xmax();

    let mut aRotation = Transform3::default();
    aRotation = Transform3::rotation(
        Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::Z),
        std::f64::consts::PI / 2.0,
    )
    .unwrap();

    let mut aTransformed = aBox.transformed(aRotation);

    assert!(aTransformed.is_open());
    assert!(!(aTransformed.is_void()));
}

//=================================================================================================
// Additional Coverage Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__SquareExtent() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);

    // SquareExtent appears to calculate the square of the diagonal length
    // For a box (0,0,0) to (2,3,4), diagonal = sqrt(2^2+3^2+4^2) = sqrt(29)
    // So SquareExtent = 29
    assert_eq!(29.0, aBox.square_extent());
}

#[test]
fn Bnd_BoxTest__SetVoidAndWholeTransitions() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    assert!(!(aBox.is_void()));

    aBox.set_void();
    assert!(aBox.is_void());

    aBox.set_whole();
    assert!(aBox.is_whole());
    assert!(!(aBox.is_void()));
}

#[test]
fn Bnd_BoxTest__DirectOpenOperations() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    // Test individual open operations
    aBox.open_xmin();
    assert!(aBox.is_open_xmin());

    aBox.open_xmax();
    assert!(aBox.is_open_xmax());

    aBox.open_ymin();
    assert!(aBox.is_open_ymin());

    aBox.open_ymax();
    assert!(aBox.is_open_ymax());

    aBox.open_zmin();
    assert!(aBox.is_open_zmin());

    aBox.open_zmax();
    assert!(aBox.is_open_zmax());
}

#[test]
fn Bnd_BoxTest__IsOutWithTransformation() {
    let mut aBox1 = BoundingBox3::default();
    aBox1.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    let mut aBox2 = BoundingBox3::default();
    aBox2.update_bounds([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);

    let mut aTransf = Transform3::default();
    assert!(aBox1.is_out_transformed(&aBox2, aTransf));

    // Transform to overlap
    aTransf = Transform3::translation([-1.5, 0.0, 0.0]);
    assert!(!(aBox1.is_out_transformed(&aBox2, aTransf)));
}

#[test]
fn Bnd_BoxTest__IsOutWithTwoTransformations() {
    let mut aBox1 = BoundingBox3::default();
    aBox1.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    let mut aBox2 = BoundingBox3::default();
    aBox2.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    let mut aT1 = Transform3::default();
    let mut aT2 = Transform3::default();
    aT1 = Transform3::translation([0.0, 0.0, 0.0]);
    aT2 = Transform3::translation([2.0, 0.0, 0.0]);

    assert!(aBox1.is_out_both(aT1, &aBox2, aT2));
}

#[test]
fn Bnd_BoxTest__IsOutLineSegment() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);

    // Line segment that passes through box
    let mut aP1 = Point3::new(-1.0, 1.0, 1.0);
    let mut aP2 = Point3::new(3.0, 1.0, 1.0);
    let mut aDir = Direction3::X;

    assert!(!(aBox.is_out_segment(aP1, aP2, aDir)));

    // Line segment that misses box
    let mut aP3 = Point3::new(-1.0, 3.0, 1.0);
    let mut aP4 = Point3::new(3.0, 3.0, 1.0);

    assert!(aBox.is_out_segment(aP3, aP4, aDir));
}
