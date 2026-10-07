//! Native parity for focused `ShapeUpgrade_UnifySameDomain::Build` paths.
#[path = "shape_upgrade_unify_same_domain/bsplines.rs"]
mod bsplines;
#[path = "shape_upgrade_unify_same_domain/circles.rs"]
mod circles;
mod common;
#[path = "shape_upgrade_unify_same_domain/controls.rs"]
mod controls;
#[path = "shape_upgrade_unify_same_domain/global_context.rs"]
mod global_context;
#[path = "shape_upgrade_unify_same_domain/nested.rs"]
mod nested;
#[path = "shape_upgrade_unify_same_domain/pcurve_circle.rs"]
mod pcurve_circle;
#[path = "shape_upgrade_unify_same_domain/pcurve_concat.rs"]
mod pcurve_concat;
#[path = "shape_upgrade_unify_same_domain/pcurve_seam.rs"]
mod pcurve_seam;
use common::face_count;
#[path = "support/unify_internal_paths.rs"]
mod unify_internal_paths;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeWire, BRepBuilderAPISewing,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use unify_internal_paths::{
    face_with_internal_path, internal_edge_count, internal_wire_count, sew_coplanar_faces,
    unify_faces_with_internal_edges,
};

#[test]
fn two_collinear_edges_on_a_wire_merge_to_one_native_line() {
    let input = collinear_wire();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);

    unifier.build().unwrap();

    assert_native_line_merge(&input, &unifier.shape(), "linear");
}

#[test]
fn three_collinear_edges_merge_as_one_native_chain() {
    let input = line_chain(&[0., 1., 2., 3.]);
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    assert_native_line_merge(&input, &unifier.shape(), "linear_three");
}

#[test]
fn reverse_insertion_order_uses_native_wire_chain_order() {
    let input = reverse_order_wire();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    assert_native_line_merge(&input, &unifier.shape(), "reverse_order");
}

#[test]
fn reversed_edge_occurrences_follow_native_wire_orientation() {
    let input = reversed_occurrences_wire();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    assert_native_line_merge(&input, &unifier.shape(), "reversed_occurrences");
}

fn assert_native_line_merge(_input: &Shape, result: &Shape, case: &str) {
    assert!(
        BRepCheckAnalyzer::new(result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let result_edges = edge_shapes(result);
    assert_eq!(result_edges.len(), native_edge_count(case));
    let (result_start, result_end) = edge_endpoints(&result_edges[0]);
    assert_eq!(
        endpoint_coordinates(result_start, result_end),
        native_endpoints(case)
    );
}

fn edge_shapes(shape: &Shape) -> Vec<rustycad_tkbrep::topods::Edge> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .map(|edge| rustycad_tkbrep::topods::edge(&edge).unwrap())
        .collect()
}

fn edge_endpoints(
    edge: &rustycad_tkbrep::topods::Edge,
) -> (
    rustycad_tkbrep::topods::Vertex,
    rustycad_tkbrep::topods::Vertex,
) {
    (
        rustycad_tkbrep::topexp::first_vertex(edge, true),
        rustycad_tkbrep::topexp::last_vertex(edge, true),
    )
}

fn endpoint_coordinates(
    first: rustycad_tkbrep::topods::Vertex,
    last: rustycad_tkbrep::topods::Vertex,
) -> Vec<f64> {
    let first = rustycad_tkbrep::brep::BRepTool::pnt(&first).unwrap();
    let last = rustycad_tkbrep::brep::BRepTool::pnt(&last).unwrap();
    vec![
        first.x(),
        first.y(),
        first.z(),
        last.x(),
        last.y(),
        last.z(),
    ]
}

#[test]
fn non_collinear_edges_remain_separate_like_native() {
    let mut wire = BRepBuilderAPIMakeWire::new();
    wire.add_edge(&line_edge(0., 1.)).unwrap();
    let second =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(1., 0., 0.), Point3::new(1., 1., 0.))
            .unwrap()
            .edge()
            .unwrap();
    wire.add_edge(&second).unwrap();
    let input = wire.wire().unwrap().into_shape();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let count = Explorer::new(&unifier.shape(), ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .count();
    assert_eq!(count, native_edge_count("perpendicular"));
}

#[test]
fn disabled_edge_unification_keeps_both_native_edges() {
    let input = collinear_wire();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, false, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    let count = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .count();
    assert_eq!(count, native_edge_count("disabled"));
}

#[test]
fn planar_face_boundary_line_chain_is_rebuilt_like_native() {
    let input = face_with_split_boundary();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    let expected = native_face_summary();
    assert_eq!(face_count(&result), expected.0);
    assert_eq!(edge_shapes(&result).len(), expected.1);
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

fn face_with_split_boundary() -> Shape {
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(2., 0., 0.),
        Point3::new(2., 2., 0.),
        Point3::new(0., 2., 0.),
        Point3::new(0., 0., 0.),
    ];
    let wire = wire_from_points(&points);
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn wire_from_points(points: &[Point3]) -> Wire {
    let mut wire = BRepBuilderAPIMakeWire::new();
    for pair in points.windows(2) {
        wire.add_edge(
            &BRepBuilderAPIMakeEdge::from_points(pair[0], pair[1])
                .unwrap()
                .edge()
                .unwrap(),
        )
        .unwrap();
    }
    wire.wire().unwrap()
}

fn collinear_wire() -> Shape {
    line_chain(&[0., 1., 2.])
}

fn reverse_order_wire() -> Shape {
    let mut wire = BRepBuilderAPIMakeWire::new();
    for pair in [(2., 3.), (1., 2.), (0., 1.)] {
        wire.add_edge(&line_edge(pair.0, pair.1)).unwrap();
    }
    wire.wire().unwrap().into_shape()
}

fn reversed_occurrences_wire() -> Shape {
    let mut wire = BRepBuilderAPIMakeWire::new();
    wire.add_edge(&line_edge(0., 1.)).unwrap();
    let mut second = line_edge(2., 1.);
    second.reverse();
    wire.add_edge(&second).unwrap();
    let mut third = line_edge(3., 2.);
    third.reverse();
    wire.add_edge(&third).unwrap();
    wire.wire().unwrap().into_shape()
}

fn line_chain(points: &[f64]) -> Shape {
    let mut wire = BRepBuilderAPIMakeWire::new();
    for pair in points.windows(2) {
        wire.add_edge(&line_edge(pair[0], pair[1])).unwrap();
    }
    wire.wire().unwrap().into_shape()
}

fn native_edge_count(case: &str) -> usize {
    include_str!("../../../tools/oracle/tkshhealing_unify_collinear_wire.expected")
        .lines()
        .find(|line| line.starts_with(case))
        .unwrap()
        .split_whitespace()
        .last()
        .unwrap()
        .parse()
        .unwrap()
}

fn native_endpoints(case: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_unify_collinear_wire.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{case} endpoints")))
        .unwrap()
        .split_whitespace()
        .skip(2)
        .map(|value| value.parse().unwrap())
        .collect()
}

fn native_face_summary() -> (usize, usize) {
    let row = include_str!("../../../tools/oracle/tkshhealing_unify_collinear_wire.expected")
        .lines()
        .find(|line| line.starts_with("planar_face"))
        .unwrap();
    let values = row.split_whitespace().collect::<Vec<_>>();
    (values[2].parse().unwrap(), values[4].parse().unwrap())
}

fn line_edge(first_x: f64, last_x: f64) -> rustycad_tkbrep::topods::Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(first_x, 0., 0.), Point3::new(last_x, 0., 0.))
        .unwrap()
        .edge()
        .unwrap()
}

#[test]
fn shared_vertex_internal_path_rebuild_terminates_in_face_merge() {
    let input = sewn_internal_path_pair();
    let result = unify_faces_with_internal_edges(&input);

    assert_internal_path_merge(&result);
}

fn sewn_internal_path_pair() -> Shape {
    let first = face_with_internal_path(
        0.0,
        10.0,
        0.0,
        10.0,
        &[
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(5.0, 5.0, 0.0),
            Point3::new(8.0, 2.0, 0.0),
        ],
    );
    let second = face_with_internal_path(0.0, 10.0, 10.0, 20.0, &[]);
    sew_coplanar_faces(first, second)
}

fn assert_internal_path_merge(result: &Shape) {
    assert_eq!(face_count(result), 1);
    assert_eq!(internal_wire_count(result), 1);
    assert_eq!(internal_edge_count(result), 2);
}
