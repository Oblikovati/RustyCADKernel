//! Complete Bnd_B2 legacy suite with float and double instantiations.
#![allow(non_snake_case, unused_mut, unused_assignments)]
use rustycad_tkmath::{
    bnd::CompactBox2,
    gp::{Axis2d, Direction2, Point2, Transform2, Xy},
};
#[test]
fn Bnd_B2dTest__DefaultConstructor() {
    let mut aBox = CompactBox2::<f64>::default();
    assert!(aBox.is_void());
}

#[test]
fn Bnd_B2dTest__ConstructorWithCenterAndHSize() {
    let mut aCenter = Xy::new(5.0, 10.0);
    let mut aHSize = Xy::new(2.0, 3.0);

    let mut aBox = CompactBox2::<f64>::new(aCenter, aHSize);

    assert!(!(aBox.is_void()));

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();

    assert!((aMin.x() - 3.0).abs() <= 1e-7);
    assert!((aMin.y() - 7.0).abs() <= 1e-7);
    assert!((aMax.x() - 7.0).abs() <= 1e-7);
    assert!((aMax.y() - 13.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B2dTest__Clear() {
    let mut aBox = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));
    assert!(!(aBox.is_void()));

    aBox.clear();
    assert!(aBox.is_void());
}

#[test]
fn Bnd_B2dTest__AddPoint() {
    let mut aBox = CompactBox2::<f64>::default();

    aBox.add_xy(Xy::new(1.0, 2.0));
    assert!(!(aBox.is_void()));

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();
    assert!((aMin.x() - 1.0).abs() <= 1e-7);
    assert!((aMin.y() - 2.0).abs() <= 1e-7);
    assert!((aMax.x() - 1.0).abs() <= 1e-7);
    assert!((aMax.y() - 2.0).abs() <= 1e-7);

    aBox.add_xy(Xy::new(4.0, 5.0));
    aMin = aBox.corner_min();
    aMax = aBox.corner_max();
    assert!((aMin.x() - 1.0).abs() <= 1e-7);
    assert!((aMin.y() - 2.0).abs() <= 1e-7);
    assert!((aMax.x() - 4.0).abs() <= 1e-7);
    assert!((aMax.y() - 5.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B2dTest__AddPnt2d() {
    let mut aBox = CompactBox2::<f64>::default();
    aBox.add_point(Point2::new(1.0, 2.0));

    assert!(!(aBox.is_void()));
    let mut aMin = aBox.corner_min();
    assert!((aMin.x() - 1.0).abs() <= 1e-7);
    assert!((aMin.y() - 2.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B2dTest__AddBox() {
    let mut aBox1 = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));
    let mut aBox2 = CompactBox2::<f64>::new(Xy::new(3.0, 3.0), Xy::new(1.0, 1.0));

    aBox1.add_box(&aBox2);

    let mut aMin = aBox1.corner_min();
    let mut aMax = aBox1.corner_max();
    assert!((aMin.x() - -1.0).abs() <= 1e-7);
    assert!((aMin.y() - -1.0).abs() <= 1e-7);
    assert!((aMax.x() - 4.0).abs() <= 1e-7);
    assert!((aMax.y() - 4.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B2dTest__SquareExtent() {
    let mut aBox = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(3.0, 4.0));
    let aSqExtent: f64 = aBox.square_extent();

    // Square diagonal = 4 * (3^2 + 4^2) = 4 * 25 = 100
    assert!((aSqExtent - 100.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B2dTest__Enlarge() {
    let mut aBox = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));
    aBox.enlarge(0.5);

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();
    assert!((aMin.x() - -1.5).abs() <= 1e-7);
    assert!((aMin.y() - -1.5).abs() <= 1e-7);
    assert!((aMax.x() - 1.5).abs() <= 1e-7);
    assert!((aMax.y() - 1.5).abs() <= 1e-7);
}

#[test]
fn Bnd_B2dTest__Limit() {
    // Test limiting a large box by a smaller box inside it
    // aBox1: (-5, -5) to (5, 5)
    // aBox2: (0, 0) to (4, 4)
    let mut aBox1 = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(5.0, 5.0));
    let mut aBox2 = CompactBox2::<f64>::new(Xy::new(2.0, 2.0), Xy::new(2.0, 2.0));

    let isLimited: bool = aBox1.limit(&aBox2);
    assert!(isLimited);

    // After limiting, aBox1's min corner should align with aBox2's min corner
    let mut aMin = aBox1.corner_min();
    let mut aMax = aBox1.corner_max();
    assert!((aMin.x() - 0.0).abs() <= 1e-7);
    assert!((aMin.y() - 0.0).abs() <= 1e-7);
    assert!((aMax.x() - 5.0).abs() <= 1e-7);
    assert!((aMax.y() - 5.0).abs() <= 1e-7);

    // Test with non-intersecting boxes
    let mut aBox3 = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));
    let mut aBox4 = CompactBox2::<f64>::new(Xy::new(10.0, 10.0), Xy::new(1.0, 1.0));
    let isLimited2: bool = aBox3.limit(&aBox4);
    assert!(!(isLimited2));
}

#[test]
fn Bnd_B2dTest__IsOutPoint() {
    let mut aBox = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));

    assert!(!(aBox.is_out_point(Xy::new(0.0, 0.0))));
    assert!(!(aBox.is_out_point(Xy::new(0.5, 0.5))));
    assert!(aBox.is_out_point(Xy::new(2.0, 0.0)));
    assert!(aBox.is_out_point(Xy::new(0.0, 2.0)));
}

#[test]
fn Bnd_B2dTest__IsOutCircle() {
    let mut aBox = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));

    // Circle at (0, 0) with small radius - should intersect
    assert!(!(aBox.is_out_circle(Xy::new(0.0, 0.0), 0.5, false)));

    // Circle far away - should not intersect
    assert!(aBox.is_out_circle(Xy::new(10.0, 10.0), 1.0, false));
}

#[test]
fn Bnd_B2dTest__IsOutBox() {
    let mut aBox1 = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));
    let mut aBox2 = CompactBox2::<f64>::new(Xy::new(0.5, 0.5), Xy::new(1.0, 1.0));
    let mut aBox3 = CompactBox2::<f64>::new(Xy::new(5.0, 5.0), Xy::new(1.0, 1.0));

    assert!(!(aBox1.is_out_box(&aBox2)));
    assert!(aBox1.is_out_box(&aBox3));
}

#[test]
fn Bnd_B2dTest__IsOutLine() {
    let mut aBox = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));

    // Line passing through box
    let mut aLine1 = Axis2d::new(Point2::new(-2.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    assert!(!(aBox.is_out_line(aLine1)));

    // Line not intersecting box
    let mut aLine2 = Axis2d::new(Point2::new(-2.0, 5.0), Direction2::new(1.0, 0.0).unwrap());
    assert!(aBox.is_out_line(aLine2));
}

#[test]
fn Bnd_B2dTest__IsOutSegment() {
    let mut aBox = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(1.0, 1.0));

    // Segment intersecting box
    assert!(!(aBox.is_out_segment(Xy::new(-2.0, 0.0), Xy::new(2.0, 0.0))));

    // Segment not intersecting box
    assert!(aBox.is_out_segment(Xy::new(5.0, 5.0), Xy::new(6.0, 6.0)));
}

#[test]
fn Bnd_B2dTest__IsInBox() {
    let mut aBox1 = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(0.5, 0.5));
    let mut aBox2 = CompactBox2::<f64>::new(Xy::new(0.0, 0.0), Xy::new(2.0, 2.0));

    assert!(aBox1.is_in_box(&aBox2));
    assert!(!(aBox2.is_in_box(&aBox1)));
}

#[test]
fn Bnd_B2dTest__Transformed() {
    let mut aBox = CompactBox2::<f64>::new(Xy::new(1.0, 1.0), Xy::new(1.0, 1.0));

    let mut aTrsf = Transform2::default();
    aTrsf = Transform2::translation([2.0, 3.0]);

    let mut aTransformedBox = aBox.transformed(aTrsf);

    let mut aMin = aTransformedBox.corner_min();
    let mut aMax = aTransformedBox.corner_max();

    assert!((aMin.x() - 2.0).abs() <= 1e-7);
    assert!((aMin.y() - 3.0).abs() <= 1e-7);
    assert!((aMax.x() - 4.0).abs() <= 1e-7);
    assert!((aMax.y() - 5.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B2dTest__SetCenterAndHSize() {
    let mut aBox = CompactBox2::<f64>::default();

    aBox.set_center(Xy::new(5.0, 10.0));
    aBox.set_half_size(Xy::new(2.0, 3.0));

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();

    assert!((aMin.x() - 3.0).abs() <= 1e-7);
    assert!((aMin.y() - 7.0).abs() <= 1e-7);
    assert!((aMax.x() - 7.0).abs() <= 1e-7);
    assert!((aMax.y() - 13.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B2fTest__FloatPrecision() {
    let mut aBox = CompactBox2::<f32>::new(Xy::new(1.0, 2.0), Xy::new(0.5, 0.5));

    assert!(!(aBox.is_void()));

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();

    assert!((aMin.x() - 0.5).abs() <= 1e-5);
    assert!((aMin.y() - 1.5).abs() <= 1e-5);
    assert!((aMax.x() - 1.5).abs() <= 1e-5);
    assert!((aMax.y() - 2.5).abs() <= 1e-5);
}
