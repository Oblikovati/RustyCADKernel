//! Differential coverage of ShapeAnalysis_CheckSmallFace::CheckPinEdges.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Builder, Edge, Vertex, edge};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;

const ORACLE: &str = include_str!("../../../tools/oracle/shape_analysis_check_pin_edges.expected");

#[test]
fn check_pin_edges_matches_native_projection_and_derivative_branches() {
    assert_case("circle", circular_pin_edges(), true);
    assert_case("line", linear_pin_edges(), false);
}

fn assert_case(name: &str, edges: (Edge, Edge), expected: bool) {
    let native = ORACLE.lines().find(|row| row.starts_with(name)).unwrap();
    let value = native
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<i32>()
        .unwrap();
    assert_eq!(value, expected as i32, "invalid pinned fixture {name}");
    let result = ShapeAnalysisCheckSmallFace::new()
        .check_pin_edges(&edges.0, &edges.1, 100.0, 200.0, 1e-3)
        .unwrap();
    assert_eq!(result, expected, "{name}");
}

fn circular_pin_edges() -> (Edge, Edge) {
    let tau = std::f64::consts::TAU;
    let circle: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(Axis2::new(Point3::default(), Direction3::Z).unwrap(), 1.0).unwrap(),
    )
    .cast();
    let common = vertex(circle.value(tau).unwrap());
    let first = vertex(circle.value(tau - 0.1).unwrap());
    let second = vertex(circle.value(tau - 0.2).unwrap());
    (
        arc(&circle, &first, &common, tau - 0.1, tau),
        arc(&circle, &second, &common, tau - 0.2, tau),
    )
}

fn linear_pin_edges() -> (Edge, Edge) {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    let common = vertex(Point3::default());
    let first = vertex(Point3::new(-0.1, 0.0, 0.0));
    let second = vertex(Point3::new(-0.2, 0.0, 0.0));
    (
        arc(&line, &first, &common, -0.1, 0.0),
        arc(&line, &second, &common, -0.2, 0.0),
    )
}

fn arc(
    curve: &Handle<dyn GeomCurve>,
    first: &Vertex,
    last: &Vertex,
    first_parameter: f64,
    last_parameter: f64,
) -> Edge {
    let builder = BRepBuilder::default();
    let mut result = Edge::default();
    builder
        .make_edge_with_curve(&mut result, curve, 1e-3)
        .unwrap();
    builder
        .range(&result, first_parameter, last_parameter, true)
        .unwrap();
    let mut shape = result.into_shape();
    Builder
        .add(&mut shape, &first.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut shape, &last.oriented(Orientation::Reversed))
        .unwrap();
    edge(&shape).unwrap()
}

fn vertex(point: Point3) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, 1e-3)
        .unwrap();
    vertex
}
