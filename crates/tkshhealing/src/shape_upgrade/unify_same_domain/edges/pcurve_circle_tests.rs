use super::{extend_range, mapped_range, oriented_circle, reparameterized_circle, same_circle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib::ElementaryCurve;
use rustycad_tkmath::gp::{Axis22d, Circle2, Direction2, Point2};

#[test]
fn native_circle_support_match_uses_center_and_radius_tolerance() {
    let circle = circle_at(Point2::default(), 2.0);
    let near = circle_at(Point2::new(1.0e-8, 0.0), 2.0 + 1.0e-8);
    let far = circle_at(Point2::new(1.0e-4, 0.0), 2.0);
    assert!(same_circle(circle, near));
    assert!(!same_circle(circle, far));
}

#[test]
fn native_circle_union_maps_and_sorts_second_arc_endpoints() {
    let first = circle_at(Point2::default(), 2.0);
    let second = circle_at(Point2::new(1.0e-8, 0.0), 2.0 + 1.0e-8);
    let points = [second.value(1.8), second.value(0.9)];
    let mapped = mapped_range(first, points).unwrap();
    assert!((mapped.0 - 0.899_999_996_083_365_6).abs() < 1.0e-14);
    assert!((mapped.1 - 1.799_999_995_130_762).abs() < 1.0e-14);
}

#[test]
fn native_circle_union_extends_last_range_for_forward_edges() {
    let range = extend_range((0.2, 1.1), (0.9, 1.8), Orientation::Forward);
    assert_eq!(range, (0.2, 1.8));
}

#[test]
fn native_circle_union_extends_first_range_for_reversed_edges() {
    let range = extend_range((1.1, 2.0), (0.2, 0.9), Orientation::Reversed);
    assert_eq!(range, (0.2, 2.0));
}

#[test]
fn native_reversed_circle_curve_maps_range_and_endpoint_values() {
    let circle = circle_at(Point2::default(), 2.0);
    let merged =
        oriented_circle(circle, (0.2, 1.799_999_995_130_762), Orientation::Reversed).unwrap();
    assert!((merged.range.0 - 4.483185312048825).abs() < 1.0e-14);
    assert!((merged.range.1 - 6.083185307179586).abs() < 1.0e-14);
    let curve = merged.curve.get().unwrap();
    assert_point(
        curve.eval_d0(merged.range.0).unwrap(),
        (-0.4544041799023818, 1.9476952639689926),
    );
    assert_point(
        curve.eval_d0(merged.range.1).unwrap(),
        (1.960133155682483, 0.39733866159012327),
    );
}

#[test]
fn native_circle_reparameterization_rotates_axis_to_preserve_edge_points() {
    let circle = circle_at(Point2::default(), 2.0);
    let adjusted = reparameterized_circle(circle, 1.6);
    assert_point(
        adjusted.value(0.4),
        (-0.832_293_673_094_284_9, 1.8185948536513632),
    );
    assert_point(
        circle.value(2.0),
        (-0.832_293_673_094_284_8, 1.8185948536513634),
    );
}

fn circle_at(location: Point2, radius: f64) -> Circle2 {
    let axis = Axis22d::new(location, Direction2::X, true).unwrap();
    Circle2::new(axis, radius).unwrap()
}

fn assert_point(point: Point2, expected: (f64, f64)) {
    assert!((point.x() - expected.0).abs() < 1.0e-14);
    assert!((point.y() - expected.1).abs() < 1.0e-14);
}
