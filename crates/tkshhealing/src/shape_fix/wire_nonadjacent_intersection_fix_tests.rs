use super::*;
use crate::shape_analysis::ShapeAnalysisWire;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2, Point3};

const EXPECTED: &str =
    include_str!("../../../../tools/oracle/tkshhealing_wire_nonadjacent_intersection.expected");

#[test]
fn explicit_nonadjacent_pair_increases_nearest_endpoint_tolerance() {
    let (analyzer, vertices) = nonadjacent_pair_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(2.0);

    assert!(fixer.fix_nonadjacent_intersecting_edges(1, 3).unwrap());
    assert_eq!(
        fixer.last_fix_status(ShapeExtendStatus::Done1),
        expected_flag("pair", "done1")
    );
    assert_eq!(
        fixer.last_fix_status(ShapeExtendStatus::Fail2),
        expected_flag("pair", "fail2")
    );
    assert_pair_endpoint_tolerances(&vertices);
}

#[test]
fn explicit_pair_respects_maximum_tolerance_without_mutation() {
    let (analyzer, vertices) = nonadjacent_pair_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(0.5);

    assert!(fixer.fix_nonadjacent_intersecting_edges(1, 3).unwrap());
    assert_eq!(
        fixer.last_fix_status(ShapeExtendStatus::Done1),
        expected_flag("limited", "done1")
    );
    assert_eq!(
        fixer.last_fix_status(ShapeExtendStatus::Fail2),
        expected_flag("limited", "fail2")
    );
    assert_close(
        BRepTool::vertex_tolerance(&vertices[3]).unwrap(),
        expected("limited", "tolerance"),
    );
}

#[test]
fn explicit_pair_without_crossing_preserves_native_failure_status() {
    let (analyzer, _) = nonadjacent_pair_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);

    assert_eq!(
        fixer.fix_nonadjacent_intersecting_edges(1, 2).unwrap(),
        expected_flag("no_crossing", "no_crossing")
    );
    assert_eq!(
        fixer.last_fix_status(ShapeExtendStatus::Done1),
        expected_flag("no_crossing", "done1")
    );
    assert_eq!(
        fixer.last_fix_status(ShapeExtendStatus::Fail1),
        expected_flag("no_crossing", "fail1")
    );
}

fn nonadjacent_pair_fixture() -> (Handle<ShapeAnalysisWire>, [Vertex; 5]) {
    let face = planar_face();
    let edges = fixture_edges(&face);
    let analyzer = fixture_analyzer(&face, &edges);
    let vertices = fixture_vertices(&edges);
    (analyzer, vertices)
}

fn fixture_edges(face: &Face) -> [Edge; 3] {
    [
        first_crossing_edge(face),
        unrelated_edge(face),
        third_crossing_edge(face),
    ]
}

fn first_crossing_edge(face: &Face) -> Edge {
    edge(
        face,
        Point3::new(-1.0, 0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        2.0,
    )
}

fn unrelated_edge(face: &Face) -> Edge {
    edge(
        face,
        Point3::new(5.0, 5.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        1.0,
    )
}

fn third_crossing_edge(face: &Face) -> Edge {
    edge(
        face,
        Point3::new(0.0, -1.0, 0.0),
        Direction2::new(0.0, 1.0).unwrap(),
        2.0,
    )
}

fn fixture_analyzer(face: &Face, edges: &[Edge; 3]) -> Handle<ShapeAnalysisWire> {
    let data = Handle::new(ShapeExtendWireData::new());
    for item in edges {
        data.add_edge(item, 0).unwrap();
    }
    Handle::new(ShapeAnalysisWire::from_wire_data(&data, face, 1e-7).unwrap())
}

fn fixture_vertices(edges: &[Edge; 3]) -> [Vertex; 5] {
    let analysis = ShapeAnalysisEdge::new();
    [
        analysis.first_vertex(&edges[0]),
        analysis.last_vertex(&edges[0]),
        analysis.first_vertex(&edges[1]),
        analysis.first_vertex(&edges[2]),
        analysis.last_vertex(&edges[2]),
    ]
}

fn planar_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn edge(face: &Face, start: Point3, direction: Direction2, last: f64) -> Edge {
    let end = Point3::new(
        start.x() + direction.x() * last,
        start.y() + direction.y() * last,
        0.0,
    );
    let builder = BRepBuilder::default();
    let mut result = Edge::default();
    builder.make_edge(&mut result).unwrap();
    add_vertices(&builder, &mut result, start, end);
    set_pcurve(&builder, &result, face, start, direction, last);
    result
}

fn add_vertices(builder: &BRepBuilder, edge: &mut Edge, start: Point3, end: Point3) {
    let first = vertex(start);
    let last_vertex = vertex(end);
    builder
        .add(edge, &first.oriented(Orientation::Forward))
        .unwrap();
    builder
        .add(edge, &last_vertex.oriented(Orientation::Reversed))
        .unwrap();
}

fn set_pcurve(
    builder: &BRepBuilder,
    edge: &Edge,
    face: &Face,
    start: Point3,
    direction: Direction2,
    last: f64,
) {
    let origin = Point2::new(start.x(), start.y());
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(origin, direction))).cast();
    builder
        .update_edge_pcurve_on_face(edge, &pcurve, face, 1e-7)
        .unwrap();
    builder.range_on_face(edge, face, 0.0, last).unwrap();
}

fn vertex(point: Point3) -> Vertex {
    let mut result = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut result, point, 1e-7)
        .unwrap();
    result
}

fn expected(row_label: &str, field_label: &str) -> f64 {
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(row_label))
        .unwrap();
    let mut fields = row.split_whitespace();
    while fields.next() != Some(field_label) {}
    fields.next().unwrap().parse().unwrap()
}

fn expected_flag(row_label: &str, field_label: &str) -> bool {
    expected(row_label, field_label) == 1.0
}

fn assert_pair_endpoint_tolerances(vertices: &[Vertex; 5]) {
    for index in [0, 1, 2, 4] {
        assert_close(
            BRepTool::vertex_tolerance(&vertices[index]).unwrap(),
            expected("pair", "first"),
        );
    }
    assert_close(
        BRepTool::vertex_tolerance(&vertices[3]).unwrap(),
        expected("pair", "third"),
    );
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}
