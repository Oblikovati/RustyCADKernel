use super::*;

use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Builder, Edge, Face, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tkmath::toploc::Location;

const NATIVE: &str =
    include_str!("../../../../tools/oracle/tkshhealing_wire_self_intersection_aggregate.expected");

#[test]
fn nonadjacent_aggregate_matches_native_status_ranges_and_explicit_context() {
    let face = plane();
    let data = crossing_wire_data(&face);
    let original = data.wire().unwrap();
    let fixer = aggregate_fixer(&data, &face, true);
    let changed = fixer.fix_self_intersection().unwrap();
    let observed = aggregate_observation(&fixer, &original, changed).unwrap();
    let expected = expected_row(0);
    assert_eq!(observed, expected);
}

#[test]
fn default_aggregate_matches_native_pass_order_and_statuses() {
    let face = plane();
    let data = crossing_wire_data(&face);
    let original = data.wire().unwrap();
    let fixer = aggregate_fixer(&data, &face, false);
    let changed = fixer.fix_self_intersection().unwrap();
    let observed = aggregate_observation(&fixer, &original, changed).unwrap();
    let expected = expected_row(1);
    assert_eq!(observed, expected);
}

#[test]
fn remove_loop_mode_one_rechecks_after_removal_and_runs_closed_pass() {
    let face = plane();
    let data = removable_loop_wire_data(&face);
    let original = data.wire().unwrap();
    let fixer = remove_loop_mode_one_fixer(&data, &face);
    let changed = fixer.fix_self_intersection().unwrap();
    let observed = aggregate_observation(&fixer, &original, changed).unwrap();
    assert_eq!(observed, expected_row(2));
}

#[test]
fn clear_modes_restores_defaults_and_clear_statuses_clears_all_recorded_statuses() {
    let face = plane();
    let data = crossing_wire_data(&face);
    let fixer = aggregate_fixer(&data, &face, false);
    assert!(fixer.fix_self_intersection().unwrap());
    set_nondefault_modes(&fixer);
    fixer.clear_modes();
    let mut observed = mode_values(&fixer);
    observed.push(f64::from(
        fixer.status_self_intersection(ShapeExtendStatus::Done),
    ));
    fixer.clear_statuses();
    observed.extend(status_values_after_clear(&fixer));
    assert_eq!(observed, expected_row(3));
}

fn set_nondefault_modes(fixer: &ShapeFixWire) {
    fixer.set_topology_mode(true);
    fixer.set_geometry_mode(false);
    fixer.set_closed_mode(false);
    fixer.set_preference_pcurve_mode(false);
    fixer.set_fix_gaps_by_ranges(true);
    set_nondefault_edge_modes(fixer);
    set_nondefault_aggregate_modes(fixer);
}

fn set_nondefault_edge_modes(fixer: &ShapeFixWire) {
    fixer.set_remove_loop_mode(1);
    fixer.set_fix_reversed_2d_mode(1);
    fixer.set_fix_remove_pcurve_mode(1);
    fixer.set_fix_remove_curve_3d_mode(1);
    fixer.set_fix_add_pcurve_mode(1);
    fixer.set_fix_add_curve_3d_mode(1);
    fixer.set_fix_seam_mode(1);
    fixer.set_fix_shifted_mode(1);
    fixer.set_fix_same_parameter_mode(1);
    fixer.set_fix_vertex_tolerance_mode(1);
}

fn set_nondefault_aggregate_modes(fixer: &ShapeFixWire) {
    fixer.set_fix_notched_edges_mode(1);
    fixer.set_fix_self_intersecting_edge_mode(1);
    fixer.set_fix_intersecting_edges_mode(1);
    fixer.set_fix_nonadjacent_intersecting_edges_mode(1);
    fixer.set_fix_tail_mode(1);
    fixer.set_fix_reorder_mode(1);
    fixer.set_fix_small_mode(1);
    fixer.set_fix_connected_mode(1);
    fixer.set_fix_edge_curves_mode(1);
    fixer.set_fix_degenerated_mode(1);
    fixer.set_fix_self_intersection_mode(1);
    fixer.set_fix_lacking_mode(1);
    fixer.set_fix_gaps_3d_mode(1);
    fixer.set_fix_gaps_2d_mode(1);
}

fn mode_values(fixer: &ShapeFixWire) -> Vec<f64> {
    let mut values = geometry_mode_values(fixer);
    values.extend(aggregate_mode_values(fixer));
    values
}

fn geometry_mode_values(fixer: &ShapeFixWire) -> Vec<f64> {
    vec![
        f64::from(fixer.topology_mode()),
        f64::from(fixer.geometry_mode()),
        f64::from(fixer.closed_mode()),
        f64::from(fixer.preference_pcurve_mode()),
        f64::from(fixer.fix_gaps_by_ranges()),
        f64::from(fixer.remove_loop_mode()),
        f64::from(fixer.fix_reversed_2d_mode()),
        f64::from(fixer.fix_remove_pcurve_mode()),
        f64::from(fixer.fix_remove_curve_3d_mode()),
        f64::from(fixer.fix_add_pcurve_mode()),
        f64::from(fixer.fix_add_curve_3d_mode()),
        f64::from(fixer.fix_seam_mode()),
        f64::from(fixer.fix_shifted_mode()),
        f64::from(fixer.fix_same_parameter_mode()),
        f64::from(fixer.fix_vertex_tolerance_mode()),
    ]
}

fn aggregate_mode_values(fixer: &ShapeFixWire) -> Vec<f64> {
    vec![
        f64::from(fixer.fix_notched_edges_mode()),
        f64::from(fixer.fix_self_intersecting_edge_mode()),
        f64::from(fixer.fix_intersecting_edges_mode()),
        f64::from(fixer.fix_nonadjacent_intersecting_edges_mode()),
        f64::from(fixer.fix_tail_mode()),
        f64::from(fixer.fix_reorder_mode()),
        f64::from(fixer.fix_small_mode()),
        f64::from(fixer.fix_connected_mode()),
        f64::from(fixer.fix_edge_curves_mode()),
        f64::from(fixer.fix_degenerated_mode()),
        f64::from(fixer.fix_self_intersection_mode()),
        f64::from(fixer.fix_lacking_mode()),
        f64::from(fixer.fix_gaps_3d_mode()),
        f64::from(fixer.fix_gaps_2d_mode()),
    ]
}

fn status_values_after_clear(fixer: &ShapeFixWire) -> Vec<f64> {
    let statuses = [
        fixer.last_fix_status(ShapeExtendStatus::Done),
        fixer.status_reorder(ShapeExtendStatus::Done),
        fixer.status_small(ShapeExtendStatus::Done),
        fixer.status_connected(ShapeExtendStatus::Done),
        fixer.status_edge_curves(ShapeExtendStatus::Done),
        fixer.status_degenerated(ShapeExtendStatus::Done),
        fixer.status_closed(ShapeExtendStatus::Done),
        fixer.status_self_intersection(ShapeExtendStatus::Done),
        fixer.status_lacking(ShapeExtendStatus::Done),
        fixer.status_gaps_3d(ShapeExtendStatus::Done),
        fixer.status_gaps_2d(ShapeExtendStatus::Done),
        fixer.status_notches(ShapeExtendStatus::Done),
        fixer.status_fix_tails(ShapeExtendStatus::Done),
        fixer.status_removed_segment(),
    ];
    statuses.into_iter().map(f64::from).collect()
}

fn expected_row(index: usize) -> Vec<f64> {
    NATIVE
        .lines()
        .nth(index)
        .unwrap()
        .split_whitespace()
        .map(parse_number)
        .collect()
}

fn aggregate_fixer(
    data: &Handle<ShapeExtendWireData>,
    face: &Face,
    isolate_nonadjacent: bool,
) -> ShapeFixWire {
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(data, face, 1.0e-7).unwrap());
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    if isolate_nonadjacent {
        fixer.set_fix_self_intersecting_edge_mode(0);
        fixer.set_fix_intersecting_edges_mode(0);
        fixer.set_fix_nonadjacent_intersecting_edges_mode(1);
    }
    fixer
}

fn remove_loop_mode_one_fixer(data: &Handle<ShapeExtendWireData>, face: &Face) -> ShapeFixWire {
    let fixer = aggregate_fixer(data, face, true);
    fixer.set_fix_self_intersecting_edge_mode(1);
    fixer.set_remove_loop_mode(1);
    fixer
}

fn removable_loop_wire_data(face: &Face) -> Handle<ShapeExtendWireData> {
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&removable_loop_edge(face), 0).unwrap();
    data
}

fn removable_loop_edge(face: &Face) -> Edge {
    let curve_2d = loop_pcurve();
    let curve_3d: Handle<dyn GeomCurve> =
        rustycad_tkgeombase::geom_lib::to_3d(Axis2::default(), &curve_2d).unwrap();
    let mut edge = make_loop_edge_with_vertices(&curve_3d);
    finish_loop_edge(face, &mut edge, &curve_2d);
    edge
}

fn loop_pcurve() -> Handle<dyn Geom2dCurve> {
    let poles = [
        Point2::new(0.0, 0.0),
        Point2::new(3.0, 3.0),
        Point2::new(-3.0, 3.0),
        Point2::new(1.0, 0.0),
    ];
    Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}

fn make_loop_edge_with_vertices(curve: &Handle<dyn GeomCurve>) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, curve, 1.0e-7)
        .unwrap();
    let (first, last) = loop_vertices();
    Builder
        .add(&mut edge, &first.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut edge, &last.oriented(Orientation::Reversed))
        .unwrap();
    edge
}

fn finish_loop_edge(face: &Face, edge: &mut Edge, pcurve: &Handle<dyn Geom2dCurve>) {
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(edge, pcurve, face, 1.0e-7)
        .unwrap();
    builder.range_on_face(edge, face, 0.0, 1.0).unwrap();
    builder.range(edge, 0.0, 1.0, true).unwrap();
    builder.same_range(edge, false).unwrap();
    builder.same_parameter(edge, false).unwrap();
}

fn loop_vertices() -> (Vertex, Vertex) {
    let builder = BRepBuilder::default();
    let mut first = Vertex::default();
    let mut last = Vertex::default();
    builder
        .make_vertex_at(&mut first, Point3::default(), 1.0e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last, Point3::new(1.0, 0.0, 0.0), 1.0e-7)
        .unwrap();
    (first, last)
}

fn aggregate_observation(
    fixer: &ShapeFixWire,
    original: &rustycad_tkbrep::topods::Wire,
    changed: bool,
) -> Result<Vec<f64>, rustycad_tkernel::standard::Failure> {
    let data = fixer.wire_data()?;
    let face = fixer.face();
    let mut observed = aggregate_status_values(fixer, &data.nb_edges(), changed);
    append_face_ranges(&data, &face, &mut observed)?;
    observed.push(applied_edge_count(fixer, original)?);
    Ok(observed)
}

fn aggregate_status_values(fixer: &ShapeFixWire, edge_count: &i32, changed: bool) -> Vec<f64> {
    let mut observed = vec![
        f64::from(changed),
        f64::from(*edge_count),
        f64::from(fixer.status_removed_segment()),
    ];
    observed.extend(
        ShapeExtendStatus::ALL.map(|status| f64::from(fixer.status_self_intersection(status))),
    );
    observed
}

fn append_face_ranges(
    data: &Handle<ShapeExtendWireData>,
    face: &Face,
    observed: &mut Vec<f64>,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    for index in 1..=data.nb_edges() {
        let edge = data.edge(index)?;
        let (first, last) = BRepTool::range_on_face(&edge, face)?;
        observed.extend([first, last]);
    }
    Ok(())
}

fn applied_edge_count(
    fixer: &ShapeFixWire,
    original: &rustycad_tkbrep::topods::Wire,
) -> Result<f64, rustycad_tkernel::standard::Failure> {
    let context = fixer.context();
    let applied = context.apply(&original.clone().into_shape(), ShapeEnum::Shape)?;
    let edge_count = Explorer::new(&applied, ShapeEnum::Edge, ShapeEnum::Shape)?.count();
    Ok(edge_count as f64)
}

fn crossing_wire_data(face: &Face) -> Handle<ShapeExtendWireData> {
    let data = Handle::new(ShapeExtendWireData::new());
    for (start, direction) in horizontal_crossing_lines()
        .into_iter()
        .chain(vertical_crossing_lines())
    {
        let uv = Point2::new(start.x(), start.y());
        let uv_direction = Direction2::new(direction.x(), direction.y()).unwrap();
        data.add_edge(&line_edge(face, start, direction, uv, uv_direction), 0)
            .unwrap();
    }
    data
}

fn horizontal_crossing_lines() -> [(Point3, Direction3); 2] {
    [
        (
            Point3::new(-1.0, 0.0, 0.0),
            Direction3::new(1.0, 0.0, 0.0).unwrap(),
        ),
        (
            Point3::new(3.0, 3.0, 0.0),
            Direction3::new(1.0, 0.0, 0.0).unwrap(),
        ),
    ]
}

fn vertical_crossing_lines() -> [(Point3, Direction3); 2] {
    [
        (
            Point3::new(0.0, -1.0, 0.0),
            Direction3::new(0.0, 1.0, 0.0).unwrap(),
        ),
        (
            Point3::new(-4.0, -4.0, 0.0),
            Direction3::new(0.0, 1.0, 0.0).unwrap(),
        ),
    ]
}

fn line_edge(
    face: &Face,
    start: Point3,
    direction: Direction3,
    uv: Point2,
    uv_direction: Direction2,
) -> Edge {
    let edge = edge_with_3d_line(start, direction);
    set_line_pcurve(face, &edge, uv, uv_direction);
    edge
}

fn edge_with_3d_line(start: Point3, direction: Direction3) -> Edge {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(start, direction))).cast();
    let mut edge = Edge::default();
    ShapeBuildEdge
        .make_edge_on_curve_range(&mut edge, &curve, &Location::default(), 0.0, 2.0)
        .unwrap();
    update_endpoint_tolerances(&edge);
    edge
}

fn update_endpoint_tolerances(edge: &Edge) {
    let analysis = ShapeAnalysisEdge::new();
    let builder = BRepBuilder::default();
    let first = analysis.first_vertex(edge);
    let last = analysis.last_vertex(edge);
    builder.update_vertex_tolerance(&first, 1.0e-7).unwrap();
    builder.update_vertex_tolerance(&last, 1.0e-7).unwrap();
}

fn set_line_pcurve(face: &Face, edge: &Edge, uv: Point2, direction: Direction2) {
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(uv, direction))).cast();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(edge, &pcurve, face, 1.0e-7)
        .unwrap();
    builder.range_on_face(edge, face, 0.0, 2.0).unwrap();
    builder.same_range(edge, true).unwrap();
    builder.same_parameter(edge, true).unwrap();
}

fn plane() -> Face {
    let surface = Handle::new(GeomPlane::new(Axis3::default()));
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface.cast(), 1.0e-7)
        .unwrap();
    face
}

fn parse_number(value: &str) -> f64 {
    value.parse().unwrap()
}
