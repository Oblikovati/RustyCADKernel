use super::*;
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Axis2d, Axis3, Direction2, Direction3, Point3};

const EXPECTED: &str =
    include_str!("../../../../tools/oracle/tkshhealing_wire_fix_intersections.expected");
const TOOL_EDGES_EXPECTED: &str =
    include_str!("../../../../tools/oracle/tkshhealing_intersection_tool_edges.expected");

struct ContextToleranceFixture {
    analyzer: Handle<ShapeAnalysisWire>,
    first_end: Vertex,
    second_start: Vertex,
    first: Edge,
    second: Edge,
}

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

#[test]
fn adjacent_crossing_context_replaces_edges_before_raising_local_tolerance() {
    let fixture = context_edge_tolerance_fixture();
    let reshape = Handle::new(crate::shape_build::ShapeBuildReShape::new());
    let fixer = ShapeFixWire::new();
    fixer.init(&fixture.analyzer);
    fixer.set_context(&reshape);
    fixer.set_max_tolerance(2.0);

    assert!(fixer.fix_intersecting_edges_at(2).unwrap());
    assert_context_statuses(&fixer);
    assert_original_shared_vertices(&fixture);
    assert_replaced_edge_tolerances(&reshape, &fixture);
}

#[test]
fn split_tool_preserves_native_face_range_on_tiny_trimmed_line_cut() {
    let face = planar_face();
    let edge = tiny_trimmed_line_edge(&face);
    let cut = 1.0e9 - 1.0e-4;
    let mut cut_line = true;

    let changed =
        super::super::wire_split_tool::cut_edge(&edge, 0.0, cut, &face, &mut cut_line).unwrap();

    assert_eq!(changed, expected_cut_flag("changed"));
    assert_eq!(cut_line, expected_cut_flag("cut_line"));
    assert_close(
        BRepTool::range_on_face(&edge, &face).unwrap().1,
        expected_cut_range("face_last"),
    );
    let edge_range = BRepTool::range(&edge).unwrap();
    assert_close(edge_range.0, expected_cut_range("edge_first"));
    assert_close(edge_range.1, expected_cut_range("edge_last"));
}

#[test]
fn split_tool_validates_same_parameter_cut_against_bounded_curve_domain() {
    let face = planar_face();
    let edge = bounded_curve_cut_edge(&face);
    let mut cut_line = false;

    let changed =
        super::super::wire_split_tool::cut_edge(&edge, 0.0, 1.5, &face, &mut cut_line).unwrap();

    assert!(changed);
    assert!(!cut_line);
    assert_eq!(
        BRepTool::range_on_face(&edge, &face).unwrap(),
        (0.0, expected_validated_range("face_last"))
    );
    assert_eq!(
        BRepTool::range(&edge).unwrap(),
        (0.0, expected_validated_range("edge_last"))
    );
    assert!(BRepTool::same_range(&edge).unwrap());
    assert!(BRepTool::same_parameter(&edge).unwrap());
}

#[test]
fn intersection_tool_split_edge_matches_native_ranges_vertices_and_flags() {
    let face = planar_face();
    let edge = edge_with_pcurve_and_line(
        &face,
        Point3::new(0.0, 0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        2.0,
        Point3::new(0.0, 0.0, 0.0),
        Direction3::X,
    );
    let split = vertex_at(Point3::new(1.0, 0.0, 0.0));
    let tool = super::super::wire_intersection_tool::ShapeFixIntersectionTool::new(
        &Handle::new(crate::shape_build::ShapeBuildReShape::new()),
        1.0e-7,
    );
    let (mut first, mut second) = (Edge::default(), Edge::default());

    assert!(
        tool.split_edge(&edge, 1.0, &split, &face, &mut first, &mut second, 1.0e-7)
            .unwrap()
    );
    assert_close(
        BRepTool::range(&first).unwrap().0,
        expected_tool_edge("first_first"),
    );
    assert_close(
        BRepTool::range(&first).unwrap().1,
        expected_tool_edge("first_last"),
    );
    assert_close(
        BRepTool::range(&second).unwrap().0,
        expected_tool_edge("second_first"),
    );
    assert_close(
        BRepTool::range(&second).unwrap().1,
        expected_tool_edge("second_last"),
    );
    assert!(
        ShapeAnalysisEdge::new()
            .last_vertex(&first)
            .is_same(&ShapeAnalysisEdge::new().first_vertex(&second))
    );
    assert!(!BRepTool::same_range(&first).unwrap());
    assert!(!BRepTool::same_parameter(&first).unwrap());
}

#[test]
fn intersection_tool_rejected_split_preserves_output_edges_and_shares_context() {
    let face = planar_face();
    let edge = edge_with_pcurve(
        &face,
        Point3::default(),
        Point3::new(2.0, 0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        2.0,
    );
    let endpoint = ShapeAnalysisEdge::new().first_vertex(&edge);
    let sentinel = edge_with_pcurve(
        &face,
        Point3::new(5.0, 0.0, 0.0),
        Point3::new(6.0, 0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        1.0,
    );
    let context = Handle::new(crate::shape_build::ShapeBuildReShape::new());
    let tool = super::super::wire_intersection_tool::ShapeFixIntersectionTool::with_max_tolerance(
        &context, 1.0e-7, 0.25,
    );
    let (mut first, mut second) = (sentinel.clone(), sentinel.clone());

    assert_eq!(tool.context().as_ptr(), context.as_ptr());
    assert!(
        !tool
            .split_edge(
                &edge,
                1.0,
                &endpoint,
                &face,
                &mut first,
                &mut second,
                1.0e-7
            )
            .unwrap()
    );
    assert!(first.is_same(&sentinel));
    assert!(second.is_same(&sentinel));
}

#[test]
fn intersection_tool_cut_edge_preserves_native_cut_line_output() {
    let face = planar_face();
    let edge = edge_with_pcurve_and_line(
        &face,
        Point3::default(),
        Direction2::new(1.0, 0.0).unwrap(),
        2.0,
        Point3::default(),
        Direction3::X,
    );
    let tool = super::super::wire_intersection_tool::ShapeFixIntersectionTool::new(
        &Handle::new(crate::shape_build::ShapeBuildReShape::new()),
        1.0e-7,
    );
    let mut cut_line = true;

    assert!(
        tool.cut_edge(&edge, 0.0, 0.75, &face, &mut cut_line)
            .unwrap()
    );
    assert!(cut_line);
    let range = BRepTool::range(&edge).unwrap();
    assert_close(range.0, expected_tool_edge("cut_first"));
    assert_close(range.1, expected_tool_edge("cut_last"));
}

#[test]
fn intersection_tool_trimmed_line_cut_keeps_native_tiny_spatial_remainder() {
    let face = planar_face();
    let edge = tiny_trimmed_line_edge(&face);
    let tool = super::super::wire_intersection_tool::ShapeFixIntersectionTool::new(
        &Handle::new(crate::shape_build::ShapeBuildReShape::new()),
        1.0e-7,
    );
    let mut cut_line = false;

    assert!(
        tool.cut_edge(&edge, 0.0, 1.0e9 - 1.0e-4, &face, &mut cut_line)
            .unwrap()
    );
    assert!(cut_line);
    let (_, first, last) = BRepTool::curve(&edge).unwrap();
    assert_close(first, expected_tool_edge("trimmed_first"));
    assert_close(last, expected_tool_edge("trimmed_last"));
}

fn bounded_curve_cut_edge(face: &Face) -> Edge {
    let edge = edge_with_pcurve(
        face,
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        2.0,
    );
    install_bounded_curve_representations(&edge, face);
    edge
}

fn install_bounded_curve_representations(edge: &Edge, face: &Face) {
    let axis = Axis1::new(Point3::default(), Direction3::X);
    let line = Handle::new(GeomLine::new(axis)).cast();
    let curve = Handle::new(GeomTrimmedCurve::new(&line, 0.0, 1.0, true, true).unwrap())
        .cast::<dyn GeomCurve>();
    let builder = BRepBuilder::default();
    builder.update_edge_curve(edge, &curve, 1e-7).unwrap();
    builder.range(edge, 0.0, 2.0, true).unwrap();
    builder.range_on_face(edge, face, 0.0, 2.0).unwrap();
    builder.same_range(edge, true).unwrap();
    builder.same_parameter(edge, true).unwrap();
}

fn tiny_trimmed_line_edge(face: &Face) -> Edge {
    // Native probe inserts the 3D representation first: BRep_Tool::Range uses list order.
    let mut edge = Edge::default();
    let curve = Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast();
    BRepBuilder::default()
        .make_edge_with_curve(&mut edge, &curve, 1e-7)
        .unwrap();
    install_tiny_trimmed_line_representations(&edge, face);
    edge
}

fn install_tiny_trimmed_line_representations(edge: &Edge, face: &Face) {
    let base = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    let trimmed = Handle::new(Geom2dTrimmedCurve::new(&base, 0.0, 1.0e9, true, true).unwrap())
        .cast::<dyn Geom2dCurve>();
    let curve_3d = Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X)))
        .cast::<dyn GeomCurve>();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(edge, &trimmed, face, 1e-7)
        .unwrap();
    builder.update_edge_curve(edge, &curve_3d, 1e-7).unwrap();
    builder.range_on_face(edge, face, 0.0, 1.0e9).unwrap();
    builder.range(edge, 0.0, 1.0, true).unwrap();
    builder.same_parameter(edge, false).unwrap();
}

fn context_edge_tolerance_fixture() -> ContextToleranceFixture {
    let face = planar_face();
    let first = context_first_edge(&face);
    let second = context_second_edge(&face);
    let (first_end, second_start) = move_shared_vertices(&first, &second);
    ContextToleranceFixture {
        analyzer: context_analyzer(&face, &first, &second),
        first_end,
        second_start,
        first,
        second,
    }
}

fn context_first_edge(face: &Face) -> Edge {
    edge_with_pcurve_and_line(
        face,
        Point3::new(-1.0, 0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        2.0,
        Point3::new(-1.0, 0.0, 0.0),
        Direction3::X,
    )
}

fn context_second_edge(face: &Face) -> Edge {
    edge_with_pcurve_and_line(
        face,
        Point3::new(0.0, -0.5, 0.0),
        Direction2::new(0.0, 1.0).unwrap(),
        3.0,
        Point3::new(0.0, -0.5, 0.0),
        Direction3::Y,
    )
}

fn move_shared_vertices(first: &Edge, second: &Edge) -> (Vertex, Vertex) {
    let analysis = ShapeAnalysisEdge::new();
    let first_end = analysis.last_vertex(first);
    let second_start = analysis.first_vertex(second);
    let builder = BRepBuilder::default();
    for vertex in [&first_end, &second_start] {
        builder
            .update_vertex_point(vertex, Point3::new(1.0, 0.0, 0.0), 1e-7)
            .unwrap();
    }
    (first_end, second_start)
}

fn context_analyzer(face: &Face, first: &Edge, second: &Edge) -> Handle<ShapeAnalysisWire> {
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(first, 0).unwrap();
    wire.add_edge(second, 0).unwrap();
    Handle::new(ShapeAnalysisWire::from_wire_data(&wire, face, 1e-7).unwrap())
}

fn assert_context_statuses(fixer: &ShapeFixWire) {
    assert_eq!(
        fixer.last_fix_status(ShapeExtendStatus::Done1),
        expected_context_flag("done1")
    );
    assert_eq!(
        fixer.last_fix_status(ShapeExtendStatus::Done6),
        expected_context_flag("done6")
    );
}

fn assert_original_shared_vertices(fixture: &ContextToleranceFixture) {
    assert_close(
        BRepTool::vertex_tolerance(&fixture.first_end).unwrap(),
        1e-7,
    );
    assert_close(
        BRepTool::vertex_tolerance(&fixture.second_start).unwrap(),
        1e-7,
    );
}

fn assert_replaced_edge_tolerances(
    reshape: &Handle<crate::shape_build::ShapeBuildReShape>,
    fixture: &ContextToleranceFixture,
) {
    let applied_first = applied_edge(reshape, &fixture.first);
    let applied_second = applied_edge(reshape, &fixture.second);
    assert_replacement_flags(
        &fixture.first,
        &fixture.second,
        &applied_first,
        &applied_second,
    );
    assert_edge_and_shared_tolerances(reshape, fixture, &applied_second);
}

fn assert_replacement_flags(
    first: &Edge,
    second: &Edge,
    applied_first: &Edge,
    applied_second: &Edge,
) {
    assert_eq!(
        !first.is_same(applied_first),
        expected_context_flag("edge1_replaced")
    );
    assert_eq!(
        !second.is_same(applied_second),
        expected_context_flag("edge2_replaced")
    );
}

fn assert_edge_and_shared_tolerances(
    reshape: &Handle<crate::shape_build::ShapeBuildReShape>,
    fixture: &ContextToleranceFixture,
    applied_second: &Edge,
) {
    assert_close(
        BRepTool::edge_tolerance(applied_second).unwrap(),
        expected_context_float("edge2_tol"),
    );
    let applied_shared = applied_vertex(reshape, &fixture.first_end);
    assert_close(
        BRepTool::vertex_tolerance(&applied_shared).unwrap(),
        expected_context_float("shared_tol"),
    );
}

fn edge_with_pcurve_and_line(
    face: &Face,
    start: Point3,
    direction: Direction2,
    last: f64,
    curve_start: Point3,
    curve_direction: Direction3,
) -> Edge {
    let end = Point3::new(
        start.x() + direction.x() * last,
        start.y() + direction.y() * last,
        start.z(),
    );
    let edge = edge_with_pcurve(face, start, end, direction, last);
    let axis = Axis1::new(curve_start, curve_direction);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
    let builder = BRepBuilder::default();
    builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
    builder.range(&edge, 0.0, last, true).unwrap();
    edge
}

fn applied_edge(reshape: &Handle<crate::shape_build::ShapeBuildReShape>, original: &Edge) -> Edge {
    let applied = reshape
        .apply(
            &original.clone().into_shape(),
            rustycad_tkg3d::top_abs::ShapeEnum::Edge,
        )
        .unwrap();
    rustycad_tkbrep::topods::edge(&applied).unwrap()
}

fn applied_vertex(
    reshape: &Handle<crate::shape_build::ShapeBuildReShape>,
    original: &Vertex,
) -> Vertex {
    let applied = reshape
        .apply(
            &original.clone().into_shape(),
            rustycad_tkg3d::top_abs::ShapeEnum::Vertex,
        )
        .unwrap();
    rustycad_tkbrep::topods::vertex(&applied).unwrap()
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

fn expected_context_float(label: &str) -> f64 {
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with("adjacent_context_edge_tolerance "))
        .unwrap();
    let mut fields = row.split_whitespace();
    while fields.next() != Some(label) {}
    fields.next().unwrap().parse().unwrap()
}

fn expected_context_flag(label: &str) -> bool {
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with("adjacent_context_edge_tolerance "))
        .unwrap();
    let mut fields = row.split_whitespace();
    while fields.next() != Some(label) {}
    fields.next().unwrap() == "1"
}

fn expected_cut_flag(label: &str) -> bool {
    let row =
        include_str!("../../../../tools/oracle/tkshhealing_wire_split_tool_early_range.expected")
            .trim();
    let mut fields = row.split_whitespace();
    while fields.next() != Some(label) {}
    fields.next().unwrap() == "1"
}

fn expected_cut_range(label: &str) -> f64 {
    let row =
        include_str!("../../../../tools/oracle/tkshhealing_wire_split_tool_early_range.expected")
            .trim();
    let mut fields = row.split_whitespace();
    while fields.next() != Some(label) {}
    fields.next().unwrap().parse().unwrap()
}

fn expected_validated_range(label: &str) -> f64 {
    let row = include_str!(
        "../../../../tools/oracle/tkshhealing_wire_split_tool_validated_range.expected"
    )
    .trim();
    let mut fields = row.split_whitespace();
    while fields.next() != Some(label) {}
    fields.next().unwrap().parse().unwrap()
}

fn expected_tool_edge(label: &str) -> f64 {
    TOOL_EDGES_EXPECTED
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{label}=")))
        .unwrap()
        .parse()
        .unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-14, "{actual} != {expected}");
}
