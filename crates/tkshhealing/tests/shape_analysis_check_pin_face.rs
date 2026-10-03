//! Native-backed cases for `ShapeAnalysis_CheckSmallFace::CheckPinFace`.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::{PinFaceEdgeMap, ShapeAnalysisCheckSmallFace};
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;

const ORACLE: &str = include_str!("../../../tools/oracle/shape_analysis_check_pin_face.expected");

#[test]
fn pin_face_keeps_the_native_nonmatching_edge_gate() {
    let native = ORACLE
        .split_whitespace()
        .map(|item| item.parse::<i32>().ok());
    let expected: Vec<i32> = native.flatten().collect();
    assert_eq!(expected, [0, 0, 0]);
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    let mut edge_pairs = PinFaceEdgeMap::default();
    let matched = analyzer
        .check_pin_face(&face(), &mut edge_pairs, 0.02)
        .unwrap();
    assert!(!matched);
    assert!(edge_pairs.is_empty());
    assert_eq!(analyzer.status_pin_face(ShapeExtendStatus::Done), None);
    assert_eq!(analyzer.status_pin_edges(ShapeExtendStatus::Done), None);
}

#[test]
fn pin_face_without_a_wire_returns_a_shape_specific_failure() {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1.0e-7)
        .unwrap();
    let failure = ShapeAnalysisCheckSmallFace::new()
        .check_pin_face(&face, &mut PinFaceEdgeMap::default(), 0.02)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
}

fn face() -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1.0e-7)
        .unwrap();
    let curve: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(Axis2::new(Point3::default(), Direction3::Z).unwrap(), 1.0).unwrap(),
    )
    .cast();
    let shared = vertex(Point3::new(1.0, 0.0, 0.0));
    let first = vertex(curve.value(-0.1).unwrap());
    let second = vertex(curve.value(-0.2).unwrap());
    let first_edge = circular_edge(&curve, &first, &shared, -0.1, 0.0);
    let second_edge = circular_edge(&curve, &second, &shared, -0.1, 0.0);
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let mut wire_shape = wire.into_shape();
    Builder
        .add(&mut wire_shape, &first_edge.into_shape())
        .unwrap();
    Builder
        .add(&mut wire_shape, &second_edge.into_shape())
        .unwrap();
    let mut face_shape = face.into_shape();
    Builder.add(&mut face_shape, &wire_shape).unwrap();
    topods::face(&face_shape).unwrap()
}

fn circular_edge(
    curve: &Handle<dyn GeomCurve>,
    first: &Vertex,
    last: &Vertex,
    first_parameter: f64,
    last_parameter: f64,
) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, curve, 0.02)
        .unwrap();
    builder
        .range(&edge, first_parameter, last_parameter, true)
        .unwrap();
    let mut shape = edge.into_shape();
    Builder
        .add(&mut shape, &first.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut shape, &last.oriented(Orientation::Reversed))
        .unwrap();
    topods::edge(&shape).unwrap()
}

fn vertex(point: Point3) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, 0.02)
        .unwrap();
    vertex
}
