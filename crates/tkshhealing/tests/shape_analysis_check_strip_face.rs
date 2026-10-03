//! Differential coverage of ShapeAnalysis_CheckSmallFace::CheckStripFace.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

const ORACLE: &str = include_str!("../../../tools/oracle/shape_analysis_check_strip_face.expected");

#[test]
fn check_strip_face_matches_native_direct_and_fallback_paths() {
    let (first_vertex, last_vertex) = shared_vertices();
    let shared_first = shared_edge(0.0, &first_vertex, &last_vertex);
    let shared_second = shared_edge(5e-4, &first_vertex, &last_vertex);
    check_case(
        "single_path",
        face(&[shared_first.clone(), shared_second.clone()]),
        shared_first.clone(),
        shared_second.clone(),
    );
    let loose_first = loose_edge(0.0);
    let loose_second = loose_edge(5e-4);
    check_case(
        "fallback_path",
        face(&[loose_first.clone(), loose_second.clone()]),
        loose_first.clone(),
        loose_second,
    );
    check_case(
        "one_edge",
        face(std::slice::from_ref(&loose_first)),
        loose_first,
        Edge::default(),
    );
}

fn check_case(name: &str, face: Face, expected_first: Edge, expected_second: Edge) {
    let mut first = Edge::default();
    let mut second = Edge::default();
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    let found = analyzer
        .check_strip_face(&face, &mut first, &mut second, 1e-3)
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
        assert!(first.is_same(&expected_first), "{name} first");
    }
    if !expected_second.is_null() {
        assert!(second.is_same(&expected_second), "{name} second");
    }
}

fn expected(name: &str) -> Vec<f64> {
    ORACLE
        .lines()
        .find(|row| row.starts_with(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse::<f64>().unwrap())
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
        vertex(Point3::new(0., 0., 0.)),
        vertex(Point3::new(1., 0., 0.)),
    )
}

fn vertex(point: Point3) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, 3e-3)
        .unwrap();
    vertex
}

fn shared_edge(z: f64, first: &Vertex, last: &Vertex) -> Edge {
    let line = curve(Point3::new(0., 0., z));
    BRepBuilderAPIMakeEdge::from_curve_vertices_params(&line, first, last, 0., 1.)
        .unwrap()
        .edge()
        .unwrap()
}

fn loose_edge(z: f64) -> Edge {
    let line = curve(Point3::new(0., 0., z));
    BRepBuilderAPIMakeEdge::from_curve_params(&line, 0., 1.)
        .unwrap()
        .edge()
        .unwrap()
}

fn curve(point: Point3) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(point, Direction3::X))).cast()
}
