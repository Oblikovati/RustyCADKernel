use super::*;
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point3};

const EXPECTED: &str =
    include_str!("../../../../tools/oracle/tkshhealing_wire_fix_intersections.expected");

#[test]
fn adjacent_crossing_tolerance_matches_native_non_topological_repair() {
    let (analyzer, first_end, second_start) = adjacent_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(2.0);
    assert!(fixer.fix_intersecting_edges_at(2).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Done6));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Fail2));
    assert_close(
        BRepTool::vertex_tolerance(&first_end).unwrap(),
        expected("first_tol"),
    );
    assert_close(
        BRepTool::vertex_tolerance(&second_start).unwrap(),
        expected("second_tol"),
    );
}

#[test]
fn adjacent_crossing_topology_mode_cuts_edge_ranges() {
    let (analyzer, _, _) = adjacent_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_topology_mode(true);
    assert!(fixer.fix_intersecting_edges_at(2).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Done1));
    let wire = fixer.wire_data().unwrap();
    assert_eq!(
        BRepTool::range_on_face(&wire.edge(1).unwrap(), &analyzer.face()).unwrap(),
        (0.0, expected_field("range1", 1)),
    );
    assert_eq!(
        BRepTool::range_on_face(&wire.edge(2).unwrap(), &analyzer.face()).unwrap(),
        (expected_field("range2", 0), expected_field("range2", 1)),
    );
}

#[test]
fn adjacent_crossing_uses_context_vertex_copies() {
    let (analyzer, first_end, _) = adjacent_fixture();
    let reshape = Handle::new(crate::shape_build::ShapeBuildReShape::new());
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_context(&reshape);
    fixer.set_topology_mode(true);
    assert!(fixer.fix_intersecting_edges_at(2).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert_close(
        BRepTool::pnt(&first_end).unwrap().x(),
        expected_field("original_x", 0),
    );
    let applied = reshape
        .apply(
            &first_end.clone().into_shape(),
            rustycad_tkg3d::top_abs::ShapeEnum::Vertex,
        )
        .unwrap();
    let moved = rustycad_tkbrep::topods::vertex(&applied).unwrap();
    assert_close(
        BRepTool::pnt(&moved).unwrap().x(),
        expected_field("applied_x", 0),
    );
    assert_close(
        BRepTool::vertex_tolerance(&moved).unwrap(),
        expected_field("applied_tolerance", 0),
    );
}

fn adjacent_fixture() -> (Handle<ShapeAnalysisWire>, Vertex, Vertex) {
    let face = planar_face();
    let first = edge_with_pcurve(
        &face,
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        2.0,
    );
    let second = edge_with_pcurve(
        &face,
        Point3::new(0.0, -2.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Direction2::new(0.0, 1.0).unwrap(),
        3.0,
    );
    let analysis = ShapeAnalysisEdge::new();
    let first_end = analysis.last_vertex(&first);
    let second_start = analysis.first_vertex(&second);
    let builder = BRepBuilder::default();
    builder
        .update_vertex_point(&first_end, Point3::new(1.0, 0.0, 0.0), 1e-7)
        .unwrap();
    builder
        .update_vertex_point(&second_start, Point3::new(1.0, 0.0, 0.0), 1e-7)
        .unwrap();
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&first, 0).unwrap();
    wire.add_edge(&second, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    (Handle::new(analyzer), first_end, second_start)
}

fn planar_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn edge_with_pcurve(
    face: &Face,
    start: Point3,
    end: Point3,
    direction: Direction2,
    last: f64,
) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let first = vertex_at(start);
    let last_vertex = vertex_at(end);
    builder
        .add(&mut edge, &first.oriented(Orientation::Forward))
        .unwrap();
    builder
        .add(&mut edge, &last_vertex.oriented(Orientation::Reversed))
        .unwrap();
    let origin = rustycad_tkmath::gp::Point2::new(start.x(), start.y());
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(origin, direction))).cast();
    builder
        .update_edge_pcurve_on_face(&edge, &curve, face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, face, 0.0, last).unwrap();
    edge
}

fn vertex_at(point: Point3) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, 1e-7)
        .unwrap();
    vertex
}

fn expected(label: &str) -> f64 {
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with("adjacent_tolerance "))
        .unwrap();
    let mut fields = row.split_whitespace();
    while fields.next() != Some(label) {}
    fields.next().unwrap().parse().unwrap()
}

fn expected_field(label: &str, value_index: usize) -> f64 {
    let row = EXPECTED
        .lines()
        .find(|line| line.split_whitespace().any(|field| field == label))
        .unwrap();
    let mut fields = row.split_whitespace();
    while fields.next() != Some(label) {}
    fields.nth(value_index).unwrap().parse().unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-14, "{actual} != {expected}");
}
