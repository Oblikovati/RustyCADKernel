use super::*;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3};

#[test]
fn same_circle_arcs_merge_to_the_native_open_arc() {
    assert_native_circle_merge(
        &circle_wire(&[0., std::f64::consts::FRAC_PI_2, std::f64::consts::PI]),
        "circle",
    );
}

#[test]
fn three_same_circle_arcs_merge_as_one_native_open_arc() {
    assert_native_circle_merge(
        &circle_wire(&[
            0.,
            std::f64::consts::FRAC_PI_3,
            2. * std::f64::consts::FRAC_PI_3,
            std::f64::consts::PI,
        ]),
        "circle_three",
    );
}

#[test]
fn reverse_insertion_order_keeps_the_native_circle_chain_direction() {
    assert_native_circle_merge(&reverse_order_circle_wire(), "circle_reverse_order");
}

#[test]
fn reversed_circle_edge_occurrences_keep_native_endpoints() {
    assert_native_circle_merge(&reversed_circle_wire(), "circle_reversed_occurrences");
}

#[test]
fn two_circle_arcs_form_the_native_closed_circle_edge() {
    assert_native_circle_merge(
        &circle_wire(&[0., std::f64::consts::PI, std::f64::consts::TAU]),
        "circle_closed",
    );
}

#[test]
fn near_equal_circle_radii_merge_using_the_first_native_support() {
    let frame = Axis2::default();
    let first = BRepBuilderAPIMakeEdge::from_circ_params(
        Circle3::new(frame, 2.0).unwrap(),
        0.0,
        std::f64::consts::FRAC_PI_2,
    )
    .unwrap()
    .edge()
    .unwrap();
    let second = BRepBuilderAPIMakeEdge::from_circ_params(
        Circle3::new(frame, 2.0 + 5e-8).unwrap(),
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    )
    .unwrap()
    .edge()
    .unwrap();
    let mut wire = BRepBuilderAPIMakeWire::new();
    wire.add_edge(&first).unwrap();
    wire.add_edge(&second).unwrap();
    let input = wire.wire().unwrap().into_shape();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);

    unifier.build().unwrap();

    let edges = edge_shapes(&unifier.shape());
    assert_eq!(edges.len(), 1);
    let curve = BRepAdaptorCurve::with_edge(&edges[0]).unwrap();
    assert_eq!(curve.circle().unwrap().radius(), 2.0);
    assert_eq!(curve.first_parameter().unwrap(), 0.0);
    assert_eq!(curve.last_parameter().unwrap(), std::f64::consts::PI);
}

#[test]
fn nonparallel_circle_axes_leave_adjacent_edges_separate() {
    let first_frame = Axis2::default();
    let second_frame = Axis2::with_x_direction(
        Point3::default(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    )
    .unwrap();
    let first = circle_arc(first_frame, 0.0, std::f64::consts::FRAC_PI_2);
    let second = circle_arc(second_frame, 0.0, std::f64::consts::FRAC_PI_2);
    assert_circle_pair_stays_separate(&first, &second);
}

#[test]
fn nonconcentric_circle_arcs_leave_adjacent_edges_separate() {
    let first_frame = Axis2::default();
    let second_frame = Axis2::with_x_direction(
        Point3::new(0.2, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let first = circle_arc_between(
        first_frame,
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(0.1, 3.99_f64.sqrt(), 0.0),
    );
    let second_angle = 3.99_f64.sqrt().atan2(-0.1);
    let second_end_angle = second_angle + std::f64::consts::FRAC_PI_2;
    let second = circle_arc_between(
        second_frame,
        Point3::new(0.1, 3.99_f64.sqrt(), 0.0),
        Point3::new(
            0.2 + 2.0 * second_end_angle.cos(),
            2.0 * second_end_angle.sin(),
            0.0,
        ),
    );
    assert_circle_pair_stays_separate(&first, &second);
}

#[test]
fn radius_difference_beyond_shared_vertex_tolerance_stays_unmerged() {
    let first = BRepBuilderAPIMakeEdge::from_circ_params(
        Circle3::new(Axis2::default(), 2.0).unwrap(),
        0.0,
        std::f64::consts::FRAC_PI_2,
    )
    .unwrap()
    .edge()
    .unwrap();
    let second = BRepBuilderAPIMakeEdge::from_circ_params(
        Circle3::new(Axis2::default(), 2.0 + 5e-7).unwrap(),
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    )
    .unwrap()
    .edge()
    .unwrap();
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    let mut compound = rustycad_tkbrep::topods::Compound::default();
    builder.make_compound(&mut compound);
    builder
        .add(&mut compound, &first.clone().into_shape())
        .unwrap();
    builder
        .add(&mut compound, &second.clone().into_shape())
        .unwrap();
    let input = compound.into_shape();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    assert_eq!(edge_shapes(&unifier.shape()).len(), 2);
}

fn circle_arc(frame: Axis2, first: f64, last: f64) -> rustycad_tkbrep::topods::Edge {
    BRepBuilderAPIMakeEdge::from_circ_params(Circle3::new(frame, 2.0).unwrap(), first, last)
        .unwrap()
        .edge()
        .unwrap()
}

fn circle_arc_between(frame: Axis2, first: Point3, last: Point3) -> rustycad_tkbrep::topods::Edge {
    BRepBuilderAPIMakeEdge::from_circ_points(Circle3::new(frame, 2.0).unwrap(), first, last)
        .unwrap()
        .edge()
        .unwrap()
}

fn assert_circle_pair_stays_separate(
    first: &rustycad_tkbrep::topods::Edge,
    second: &rustycad_tkbrep::topods::Edge,
) {
    let mut wire = BRepBuilderAPIMakeWire::new();
    wire.add_edge(first).unwrap();
    wire.add_edge(second).unwrap();
    let input = wire.wire().unwrap().into_shape();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    assert_eq!(edge_shapes(&unifier.shape()).len(), 2);
}

fn assert_native_circle_merge(input: &Shape, case: &str) {
    let unifier = ShapeUpgradeUnifySameDomain::new(input, true, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    let edges = edge_shapes(&result);
    assert_eq!(edges.len(), native_edge_count(case));
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let (start, end) = edge_endpoints(&edges[0]);
    assert_eq!(endpoint_coordinates(start, end), native_endpoints(case));
    assert_native_circle_sample(&edges[0], case == "circle_closed");
}

fn assert_native_circle_sample(edge: &rustycad_tkbrep::topods::Edge, closed: bool) {
    let curve = BRepAdaptorCurve::with_edge(edge).unwrap();
    assert_eq!(curve.get_type().unwrap(), CurveType::Circle);
    assert_eq!(curve.first_parameter().unwrap(), 0.);
    let last = if closed {
        std::f64::consts::TAU
    } else {
        std::f64::consts::PI
    };
    assert_eq!(curve.last_parameter().unwrap(), last);
    let middle = curve.value(last * 0.5).unwrap();
    if closed {
        assert!((middle.x() - 2.).abs() < 1e-14);
        assert!(middle.y().abs() < 1e-14);
    } else {
        assert!(middle.x().abs() < 1e-14);
        assert!((middle.y() - 2.).abs() < 1e-14);
    }
    assert!(middle.z().abs() < 1e-14);
}

fn circle_wire(breaks: &[f64]) -> Shape {
    circle_wire_segments(
        &breaks
            .windows(2)
            .map(|pair| (pair[0], pair[1]))
            .collect::<Vec<_>>(),
    )
}

fn reverse_order_circle_wire() -> Shape {
    circle_wire_segments(&[
        (std::f64::consts::FRAC_PI_2, std::f64::consts::PI),
        (0., std::f64::consts::FRAC_PI_2),
    ])
}

fn circle_wire_segments(segments: &[(f64, f64)]) -> Shape {
    let circle = Circle3::new(Axis2::default(), 2.0).unwrap();
    let mut wire = BRepBuilderAPIMakeWire::new();
    for (first, last) in segments {
        let edge = BRepBuilderAPIMakeEdge::from_circ_params(circle, *first, *last)
            .unwrap()
            .edge()
            .unwrap();
        wire.add_edge(&edge).unwrap();
    }
    wire.wire().unwrap().into_shape()
}

fn reversed_circle_wire() -> Shape {
    let circle = Circle3::new(Axis2::default(), 2.0).unwrap();
    let mut wire = BRepBuilderAPIMakeWire::new();
    for (first, last) in [
        (0., std::f64::consts::FRAC_PI_2),
        (std::f64::consts::FRAC_PI_2, std::f64::consts::PI),
    ] {
        let mut edge = BRepBuilderAPIMakeEdge::from_circ_params(circle, first, last)
            .unwrap()
            .edge()
            .unwrap();
        edge.reverse();
        wire.add_edge(&edge).unwrap();
    }
    wire.wire().unwrap().into_shape()
}
