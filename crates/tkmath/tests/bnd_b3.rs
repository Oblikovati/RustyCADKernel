//! Complete Bnd_B3 legacy suite with float and double instantiations.
#![allow(non_snake_case, unused_mut, unused_assignments)]
use rustycad_tkmath::{
    bnd::CompactBox3,
    gp::{Axis1, Axis3, Direction3, Point3, Transform3, Xyz},
};
#[test]
fn Bnd_B3dTest__DefaultConstructor() {
    let mut aBox = CompactBox3::<f64>::default();
    assert!(aBox.is_void());
}

#[test]
fn Bnd_B3dTest__ConstructorWithCenterAndHSize() {
    let mut aCenter = Xyz::new(5.0, 10.0, 15.0);
    let mut aHSize = Xyz::new(2.0, 3.0, 4.0);

    let mut aBox = CompactBox3::<f64>::new(aCenter, aHSize);

    assert!(!(aBox.is_void()));

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();

    assert!((aMin.x() - 3.0).abs() <= 1e-7);
    assert!((aMin.y() - 7.0).abs() <= 1e-7);
    assert!((aMin.z() - 11.0).abs() <= 1e-7);
    assert!((aMax.x() - 7.0).abs() <= 1e-7);
    assert!((aMax.y() - 13.0).abs() <= 1e-7);
    assert!((aMax.z() - 19.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B3dTest__Clear() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));
    assert!(!(aBox.is_void()));

    aBox.clear();
    assert!(aBox.is_void());
}

#[test]
fn Bnd_B3dTest__AddPoint() {
    let mut aBox = CompactBox3::<f64>::default();

    aBox.add_xyz(Xyz::new(1.0, 2.0, 3.0));
    assert!(!(aBox.is_void()));

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();
    assert!((aMin.x() - 1.0).abs() <= 1e-7);
    assert!((aMin.y() - 2.0).abs() <= 1e-7);
    assert!((aMin.z() - 3.0).abs() <= 1e-7);
    assert!((aMax.x() - 1.0).abs() <= 1e-7);
    assert!((aMax.y() - 2.0).abs() <= 1e-7);
    assert!((aMax.z() - 3.0).abs() <= 1e-7);

    aBox.add_xyz(Xyz::new(4.0, 5.0, 6.0));
    aMin = aBox.corner_min();
    aMax = aBox.corner_max();
    assert!((aMin.x() - 1.0).abs() <= 1e-7);
    assert!((aMin.y() - 2.0).abs() <= 1e-7);
    assert!((aMin.z() - 3.0).abs() <= 1e-7);
    assert!((aMax.x() - 4.0).abs() <= 1e-7);
    assert!((aMax.y() - 5.0).abs() <= 1e-7);
    assert!((aMax.z() - 6.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B3dTest__AddPnt() {
    let mut aBox = CompactBox3::<f64>::default();
    aBox.add_point(Point3::new(1.0, 2.0, 3.0));

    assert!(!(aBox.is_void()));
    let mut aMin = aBox.corner_min();
    assert!((aMin.x() - 1.0).abs() <= 1e-7);
    assert!((aMin.y() - 2.0).abs() <= 1e-7);
    assert!((aMin.z() - 3.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B3dTest__AddBox() {
    let mut aBox1 = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));
    let mut aBox2 = CompactBox3::<f64>::new(Xyz::new(3.0, 3.0, 3.0), Xyz::new(1.0, 1.0, 1.0));

    aBox1.add_box(&aBox2);

    let mut aMin = aBox1.corner_min();
    let mut aMax = aBox1.corner_max();
    assert!((aMin.x() - -1.0).abs() <= 1e-7);
    assert!((aMin.y() - -1.0).abs() <= 1e-7);
    assert!((aMin.z() - -1.0).abs() <= 1e-7);
    assert!((aMax.x() - 4.0).abs() <= 1e-7);
    assert!((aMax.y() - 4.0).abs() <= 1e-7);
    assert!((aMax.z() - 4.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B3dTest__SquareExtent() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(3.0, 4.0, 5.0));
    let aSqExtent: f64 = aBox.square_extent();

    // Square diagonal = 4 * (3^2 + 4^2 + 5^2) = 4 * 50 = 200
    assert!((aSqExtent - 200.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B3dTest__Enlarge() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));
    aBox.enlarge(0.5);

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();
    assert!((aMin.x() - -1.5).abs() <= 1e-7);
    assert!((aMin.y() - -1.5).abs() <= 1e-7);
    assert!((aMin.z() - -1.5).abs() <= 1e-7);
    assert!((aMax.x() - 1.5).abs() <= 1e-7);
    assert!((aMax.y() - 1.5).abs() <= 1e-7);
    assert!((aMax.z() - 1.5).abs() <= 1e-7);
}

#[test]
fn Bnd_B3dTest__Limit() {
    // Test limiting a large box by a smaller box inside it
    // aBox1: (-5, -5, -5) to (5, 5, 5)
    // aBox2: (0, 0, 0) to (4, 4, 4)
    let mut aBox1 = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(5.0, 5.0, 5.0));
    let mut aBox2 = CompactBox3::<f64>::new(Xyz::new(2.0, 2.0, 2.0), Xyz::new(2.0, 2.0, 2.0));

    let isLimited: bool = aBox1.limit(&aBox2);
    assert!(isLimited);

    // After limiting, aBox1's min corner should align with aBox2's min corner
    let mut aMin = aBox1.corner_min();
    let mut aMax = aBox1.corner_max();
    assert!((aMin.x() - 0.0).abs() <= 1e-7);
    assert!((aMin.y() - 0.0).abs() <= 1e-7);
    assert!((aMin.z() - 0.0).abs() <= 1e-7);
    assert!((aMax.x() - 5.0).abs() <= 1e-7);
    assert!((aMax.y() - 5.0).abs() <= 1e-7);
    assert!((aMax.z() - 5.0).abs() <= 1e-7);

    // Test with non-intersecting boxes
    let mut aBox3 = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));
    let mut aBox4 = CompactBox3::<f64>::new(Xyz::new(10.0, 10.0, 10.0), Xyz::new(1.0, 1.0, 1.0));
    let isLimited2: bool = aBox3.limit(&aBox4);
    assert!(!(isLimited2));
}

#[test]
fn Bnd_B3dTest__IsOutPoint() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));

    assert!(!(aBox.is_out_point(Xyz::new(0.0, 0.0, 0.0))));
    assert!(!(aBox.is_out_point(Xyz::new(0.5, 0.5, 0.5))));
    assert!(aBox.is_out_point(Xyz::new(2.0, 0.0, 0.0)));
    assert!(aBox.is_out_point(Xyz::new(0.0, 2.0, 0.0)));
    assert!(aBox.is_out_point(Xyz::new(0.0, 0.0, 2.0)));
}

#[test]
fn Bnd_B3dTest__IsOutSphere() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));

    // Sphere at (0, 0, 0) with small radius - should intersect
    assert!(!(aBox.is_out_sphere(Xyz::new(0.0, 0.0, 0.0), 0.5, false)));

    // Sphere far away - should not intersect
    assert!(aBox.is_out_sphere(Xyz::new(10.0, 10.0, 10.0), 1.0, false));
}

#[test]
fn Bnd_B3dTest__IsOutBox() {
    let mut aBox1 = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));
    let mut aBox2 = CompactBox3::<f64>::new(Xyz::new(0.5, 0.5, 0.5), Xyz::new(1.0, 1.0, 1.0));
    let mut aBox3 = CompactBox3::<f64>::new(Xyz::new(5.0, 5.0, 5.0), Xyz::new(1.0, 1.0, 1.0));

    assert!(!(aBox1.is_out_box(&aBox2)));
    assert!(aBox1.is_out_box(&aBox3));
}

#[test]
fn Bnd_B3dTest__IsOutLine() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));

    // Line passing through box
    let mut aLine1 = Axis1::new(
        Point3::new(-2.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    assert!(!(aBox.is_out_line(aLine1, false, 0.0)));

    // Line not intersecting box
    let mut aLine2 = Axis1::new(
        Point3::new(-2.0, 5.0, 5.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    assert!(aBox.is_out_line(aLine2, false, 0.0));
}

#[test]
fn Bnd_B3dTest__IsOutPlane() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));

    // Plane passing through box
    let mut aPlane1 = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    assert!(!(aBox.is_out_plane(aPlane1)));

    // Plane not intersecting box
    let mut aPlane2 = Axis3::from_normal(
        Point3::new(0.0, 0.0, 5.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    assert!(aBox.is_out_plane(aPlane2));
}

#[test]
fn Bnd_B3dTest__IsInBox() {
    let mut aBox1 = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(0.5, 0.5, 0.5));
    let mut aBox2 = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(2.0, 2.0, 2.0));

    assert!(aBox1.is_in_box(&aBox2));
    assert!(!(aBox2.is_in_box(&aBox1)));
}

#[test]
fn Bnd_B3dTest__Transformed() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(1.0, 1.0, 1.0), Xyz::new(1.0, 1.0, 1.0));

    let mut aTrsf = Transform3::default();
    aTrsf = Transform3::translation([2.0, 3.0, 4.0]);

    let mut aTransformedBox = aBox.transformed(aTrsf);

    let mut aMin = aTransformedBox.corner_min();
    let mut aMax = aTransformedBox.corner_max();

    assert!((aMin.x() - 2.0).abs() <= 1e-7);
    assert!((aMin.y() - 3.0).abs() <= 1e-7);
    assert!((aMin.z() - 4.0).abs() <= 1e-7);
    assert!((aMax.x() - 4.0).abs() <= 1e-7);
    assert!((aMax.y() - 5.0).abs() <= 1e-7);
    assert!((aMax.z() - 6.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B3dTest__TransformedWithRotation() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(1.0, 0.0, 0.0), Xyz::new(0.5, 0.5, 0.5));

    let mut aTrsf = Transform3::default();
    aTrsf = Transform3::rotation(
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        std::f64::consts::PI / 2.0,
    )
    .unwrap();

    let mut aTransformedBox = aBox.transformed(aTrsf);

    // After 90 degree rotation around Z-axis, the center should be at (0, 1, 0)
    let mut aMin = aTransformedBox.corner_min();
    let mut aMax = aTransformedBox.corner_max();

    // The rotated box should have its center near (0, 1, 0)
    // and half-size should remain approximately the same (with some expansion for rotation)
    assert!(((aMin.x() + aMax.x()) / 2.0 - 0.0).abs() <= 1e-10);
    assert!(((aMin.y() + aMax.y()) / 2.0 - 1.0).abs() <= 1e-10);
    assert!(((aMin.z() + aMax.z()) / 2.0 - 0.0).abs() <= 1e-10);
}

#[test]
fn Bnd_B3dTest__SetCenterAndHSize() {
    let mut aBox = CompactBox3::<f64>::default();

    aBox.set_center(Xyz::new(5.0, 10.0, 15.0));
    aBox.set_half_size(Xyz::new(2.0, 3.0, 4.0));

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();

    assert!((aMin.x() - 3.0).abs() <= 1e-7);
    assert!((aMin.y() - 7.0).abs() <= 1e-7);
    assert!((aMin.z() - 11.0).abs() <= 1e-7);
    assert!((aMax.x() - 7.0).abs() <= 1e-7);
    assert!((aMax.y() - 13.0).abs() <= 1e-7);
    assert!((aMax.z() - 19.0).abs() <= 1e-7);
}

#[test]
fn Bnd_B3fTest__FloatPrecision() {
    let mut aBox = CompactBox3::<f32>::new(Xyz::new(1.0, 2.0, 3.0), Xyz::new(0.5, 0.5, 0.5));

    assert!(!(aBox.is_void()));

    let mut aMin = aBox.corner_min();
    let mut aMax = aBox.corner_max();

    assert!((aMin.x() - 0.5).abs() <= 1e-5);
    assert!((aMin.y() - 1.5).abs() <= 1e-5);
    assert!((aMin.z() - 2.5).abs() <= 1e-5);
    assert!((aMax.x() - 1.5).abs() <= 1e-5);
    assert!((aMax.y() - 2.5).abs() <= 1e-5);
    assert!((aMax.z() - 3.5).abs() <= 1e-5);
}

#[test]
fn Bnd_B3dTest__IsOutRay() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(1.0, 1.0, 1.0), Xyz::new(1.0, 1.0, 1.0));

    // Ray starting before the box and pointing towards it
    let mut aRay1 = Axis1::new(
        Point3::new(-1.0, 1.0, 1.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    assert!(!(aBox.is_out_line(aRay1, true, 0.0)));

    // Ray starting inside the box
    let mut aRay2 = Axis1::new(
        Point3::new(1.0, 1.0, 1.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    assert!(!(aBox.is_out_line(aRay2, true, 0.0)));

    // Ray pointing away from the box
    let mut aRay3 = Axis1::new(
        Point3::new(-1.0, 1.0, 1.0),
        Direction3::new(-1.0, 0.0, 0.0).unwrap(),
    );
    assert!(aBox.is_out_line(aRay3, true, 0.0));
}

#[test]
fn Bnd_B3dTest__IsOutLineWithOverthickness() {
    let mut aBox = CompactBox3::<f64>::new(Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 1.0, 1.0));

    // Line passing just outside the box, but within overthickness
    let mut aLine = Axis1::new(
        Point3::new(-2.0, 1.2, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );

    // Without overthickness, should be out
    assert!(aBox.is_out_line(aLine, false, 0.0));

    // With sufficient overthickness, should intersect
    assert!(!(aBox.is_out_line(aLine, false, 0.3)));
}
