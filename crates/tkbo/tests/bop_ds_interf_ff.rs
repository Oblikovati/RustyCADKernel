//! Regression coverage for BOPDS face/face dynamic result storage.
use rustycad_tkbo::bop_ds::Point;
use rustycad_tkbo::bop_ds::{BopdsCurve, InterfFF};
use rustycad_tkernel::ncollection::DynamicArray;
use rustycad_tkmath::gp::{Point2, Point3};

#[test]
fn face_face_record_reserves_without_changing_lengths_and_stores_results() {
    let mut record = InterfFF::default();
    assert_eq!(record.indices(), (-1, -1));
    assert!(!record.tangent_faces());
    assert_eq!(array_bounds(record.curves()), (0, -1));
    assert_eq!(array_bounds(record.points()), (0, -1));

    record.init(3, 2);
    assert_eq!(record.curves().length(), 0);
    assert_eq!(record.points().length(), 0);
    record.set_indices(7, 11);
    record.set_tangent_faces(true);
    add_curve(&mut record, 0, 0.125);
    add_curve(&mut record, 4, 0.875);
    add_point(&mut record, 0, [1.0, 2.0, 3.0], [4.0, 5.0], [6.0, 7.0]);
    add_point(
        &mut record,
        2,
        [-1.0, -2.0, -3.0],
        [-4.0, -5.0],
        [-6.0, -7.0],
    );
    record.init(1, 1);

    assert_eq!(record.indices(), (7, 11));
    assert!(record.tangent_faces());
    assert_eq!(array_bounds(record.curves()), (0, 4));
    assert_eq!(record.curves().value(0).unwrap().tolerance(), 0.125);
    assert_eq!(record.curves().value(1).unwrap().tolerance(), 0.0);
    assert_eq!(record.curves().value(4).unwrap().tolerance(), 0.875);
    assert_eq!(array_bounds(record.points()), (0, 2));
    assert_eq!(
        record.points().value(0).unwrap().pnt(),
        &Point3::new(1.0, 2.0, 3.0)
    );
    assert_eq!(record.points().value(2).unwrap().index(), 12);
    assert_eq!(
        record.points().value(2).unwrap().pnt2d1(),
        &Point2::new(-4.0, -5.0)
    );
}

fn add_curve(record: &mut InterfFF, index: i32, tolerance: f64) {
    let mut curve = BopdsCurve::default();
    curve.set_tolerance(tolerance);
    record.change_curves().set_value(index, curve).unwrap();
}

fn add_point(
    record: &mut InterfFF,
    index: i32,
    point: [f64; 3],
    first_uv: [f64; 2],
    second_uv: [f64; 2],
) {
    let mut point_record = Point::default();
    point_record.set_pnt(Point3::new(point[0], point[1], point[2]));
    point_record.set_pnt2d1(Point2::new(first_uv[0], first_uv[1]));
    point_record.set_pnt2d2(Point2::new(second_uv[0], second_uv[1]));
    point_record.set_index(index + 10);
    record
        .change_points()
        .set_value(index, point_record)
        .unwrap();
}

fn array_bounds<T>(array: &DynamicArray<T>) -> (i32, i32) {
    (array.lower(), array.upper())
}
