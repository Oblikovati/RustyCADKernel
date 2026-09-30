use super::*;
use crate::brep_mesh_data::DiscretePCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;
fn samples(values: &[f64], orientation: Orientation) -> PCurveHandle {
    let curve: PCurveHandle = Handle::new(DiscretePCurve::new(
        Default::default(),
        orientation,
        &Handle::default(),
    ))
    .cast();
    for (index, x) in values.iter().enumerate() {
        curve.add_point(Point2::new(*x, 0.0), index as f64);
    }
    curve
}
#[test]
fn internal_curves_bypass_missing_samples() {
    let curve = samples(&[], Orientation::Internal);
    assert!(connect(&curve, &curve, &curve).is_ok());
    let forward = samples(&[], Orientation::Forward);
    assert!(connect(&forward, &forward, &forward).is_err());
}
#[test]
fn one_edge_closes_first_sample_onto_last() {
    let curve = samples(&[1.0, 2.0], Orientation::Forward);
    connect(&curve, &curve, &curve).unwrap();
    assert_eq!(curve.get_point(0).unwrap(), Point2::new(2.0, 0.0));
}
#[test]
fn nearest_ties_select_second_endpoint_and_first_pair() {
    let first = samples(&[0.0, 0.0], Orientation::Forward);
    let second = samples(&[-1.0, 1.0], Orientation::Forward);
    let (a, b, distance) =
        closest_pair(&endpoints(&first).unwrap(), &endpoints(&second).unwrap()).unwrap();
    assert!(a == PointRef::new(first.downgrade(), 0));
    assert!(b == PointRef::new(second.downgrade(), 1));
    assert_eq!(distance, 1.0);
}
#[test]
fn two_edge_writes_preserve_shared_single_sample_slot() {
    let first = samples(&[0.0], Orientation::Forward);
    let second = samples(&[1.0, 2.0], Orientation::Forward);
    connect(&first, &second, &first).unwrap();
    assert_eq!(first.get_point(0).unwrap().x(), 1.0);
}
#[test]
fn three_edge_adjustment_uses_unassigned_opposite_endpoint() {
    let previous = samples(&[-1.0, -2.0], Orientation::Forward);
    let current = samples(&[0.0, 10.0], Orientation::Forward);
    let next = samples(&[-0.5, 12.0], Orientation::Forward);
    connect(&previous, &current, &next).unwrap();
    assert_eq!(current.get_point(0).unwrap().x(), -0.5);
    assert_eq!(current.get_point(1).unwrap().x(), -1.0);
}
