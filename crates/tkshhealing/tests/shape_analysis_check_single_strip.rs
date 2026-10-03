//! Differential coverage of ShapeAnalysis_CheckSmallFace::CheckSingleStrip.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

const ORACLE: &str =
    include_str!("../../../tools/oracle/shape_analysis_check_single_strip.expected");

#[test]
fn check_single_strip_matches_native_topology_and_status() {
    let (first_vertex, last_vertex) = shared_vertices();
    let first = edge_between(0.0, &first_vertex, &last_vertex);
    let close = edge_between(5e-4, &first_vertex, &last_vertex);
    check_case(
        "two_strip",
        face(&[first.clone(), close.clone()]),
        first.clone(),
        close.clone(),
    );
    check_case(
        "one_edge",
        face(std::slice::from_ref(&first)),
        first.clone(),
        Edge::default(),
    );
    let extra = vertex(Point3::new(1.0, 1.0, 0.0));
    let diagonal = diagonal_edge(&first_vertex, &extra);
    check_case(
        "three_vertices",
        face(&[first.clone(), diagonal]),
        Edge::default(),
        Edge::default(),
    );
    check_case(
        "three_edges",
        face(&[first.clone(), close.clone(), first.clone()]),
        first.clone(),
        close.clone(),
    );
    let far = edge_between(2e-3, &first_vertex, &last_vertex);
    check_case("too_far", face(&[first.clone(), far.clone()]), first, far);
}

fn check_case(name: &str, face: Face, expected_first: Edge, expected_second: Edge) {
    let mut first = Edge::default();
    let mut second = Edge::default();
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    let found = analyzer
        .check_single_strip(&face, &mut first, &mut second, 1e-3)
        .unwrap();
    let values = expected(name);
    assert_eq!(found as i32, values[0] as i32, "{name}");
    assert_eq!(
        analyzer.status_strip(ShapeExtendStatus::Done3) as i32,
        values[1] as i32,
        "{name}"
    );
    assert_eq!(
        first.is_null() as i32,
        values[2] as i32,
        "{name} first null"
    );
    assert_eq!(
        second.is_null() as i32,
        values[3] as i32,
        "{name} second null"
    );
    if !expected_first.is_null() {
        assert!(first.is_same(&expected_first), "{name} first identity");
    }
    if !expected_second.is_null() {
        assert!(second.is_same(&expected_second), "{name} second identity");
    }
}

fn expected(name: &str) -> Vec<f64> {
    ORACLE
        .lines()
        .find(|row| row.starts_with(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|field| field.parse::<f64>().unwrap())
        .collect()
}

fn face(edges: &[Edge]) -> Face {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let mut wire_shape = wire.into_shape();
    for edge in edges {
        Builder
            .add(&mut wire_shape, &edge.oriented(Orientation::Forward))
            .unwrap();
    }
    let mut face = Face::default();
    builder.make_face(&mut face);
    let mut face_shape = face.into_shape();
    Builder.add(&mut face_shape, &wire_shape).unwrap();
    topods::face(&face_shape).unwrap()
}

fn shared_vertices() -> (Vertex, Vertex) {
    (
        vertex(Point3::new(0.0, 0.0, 0.0)),
        vertex(Point3::new(1.0, 0.0, 0.0)),
    )
}

fn vertex(point: Point3) -> Vertex {
    let mut vertex = Vertex::default();
    let builder = BRepBuilder::default();
    builder.make_vertex_at(&mut vertex, point, 3e-3).unwrap();
    vertex
}

fn edge_between(z: f64, first: &Vertex, last: &Vertex) -> Edge {
    let line = curve(Point3::new(0.0, 0.0, z), Direction3::X);
    BRepBuilderAPIMakeEdge::from_curve_vertices_params(&line, first, last, 0.0, 1.0)
        .unwrap()
        .edge()
        .unwrap()
}

fn diagonal_edge(first: &Vertex, last: &Vertex) -> Edge {
    let direction = Direction3::new(1.0, 1.0, 0.0).unwrap();
    let line = curve(Point3::new(0.0, 0.0, 0.0), direction);
    BRepBuilderAPIMakeEdge::from_curve_vertices_params(&line, first, last, 0.0, 2.0_f64.sqrt())
        .unwrap()
        .edge()
        .unwrap()
}

fn curve(location: Point3, direction: Direction3) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(location, direction))).cast()
}
