use super::*;

use crate::shape_analysis::ShapeAnalysisWire;
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::topods::{Builder, Edge, Face, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_lib::to_3d;
use rustycad_tkmath::gp::Axis2;
use rustycad_tkmath::gp::{Axis3, Point2, Point3};

const EXPECTED: &str =
    include_str!("../../../../tools/oracle/tkshhealing_wire_self_intersection.expected");

#[test]
fn tolerance_repair_matches_native_after_geometry_removal_is_disabled() {
    let (analyzer, first, last) = loop_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_geometry_mode(false);
    fixer.set_max_tolerance(2.0);
    assert!(fixer.fix_self_intersecting_edge(1).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Fail2));
    assert_close(
        BRepTool::vertex_tolerance(&first).unwrap(),
        expected("tol1"),
    );
    assert_close(BRepTool::vertex_tolerance(&last).unwrap(), expected("tol2"));
}

#[test]
fn projected_loop_removal_matches_native_done4_path() {
    let (analyzer, face, _first, _last) = removable_loop_fixture();
    let mut points_2d = Sequence::default();
    let mut points_3d = Sequence::default();
    analyzer
        .check_self_intersecting_edge_into(1, &mut points_2d, &mut points_3d)
        .unwrap();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(2.0);
    let edge = analyzer.wire_data().edge(1).unwrap();
    assert!(
        fixer
            .try_remove_loop(&edge, &face, points_2d.value(1).unwrap(), 0.1, 1e-7, false)
            .unwrap()
    );
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done4));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Done1));
    let edge = analyzer.wire_data().edge(1).unwrap();
    assert!(
        !BRepTool::curve_on_face(&edge, &face)
            .unwrap()
            .pcurve
            .is_null()
    );
}

#[test]
fn edge_fix_rechecks_after_native_loop_removal() {
    let (analyzer, _face, _first, _last) = removable_loop_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(2.0);
    assert!(fixer.fix_self_intersecting_edge(1).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done4));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Done1));
    let mut points_2d = Sequence::default();
    let mut points_3d = Sequence::default();
    analyzer
        .check_self_intersecting_edge_into(1, &mut points_2d, &mut points_3d)
        .unwrap();
    assert!(!analyzer.last_check_status(ShapeExtendStatus::Done));
}

#[test]
fn mode_one_replaces_crossing_edge_with_two_edges() {
    let (analyzer, _face, _first, _last) = removable_loop_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_remove_loop_mode(1);
    assert!(fixer.fix_self_intersecting_edge(1).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done4));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done8));
    let edge_count = fixer.wire_data().unwrap().nb_edges();
    assert_eq!(edge_count, expected_integer("remove_loop_mode1", "edges"));
}

#[test]
fn mode_zero_removes_the_loop_from_pcurve_and_3d_curve() {
    let (analyzer, _face, _first, _last) = removable_loop_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_remove_loop_mode(0);
    assert!(fixer.fix_self_intersecting_edge(1).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done4));
    assert_eq!(
        fixer.wire_data().unwrap().nb_edges(),
        expected_integer("remove_loop_mode0", "edges")
    );
    let edge = fixer.wire_data().unwrap().edge(1).unwrap();
    assert_close(
        BRepTool::edge_tolerance(&edge).unwrap(),
        expected_real("remove_loop_mode0", "tolerance"),
    );
}

#[test]
fn mode_zero_reconstructs_a_spatial_3d_loop() {
    let (analyzer, edge) = spatial_loop_fixture();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_remove_loop_mode(0);
    assert!(fixer.fix_self_intersecting_edge(1).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done4));
    let (curve, first, last) = BRepTool::curve(&edge).unwrap();
    assert_close(first, 0.0);
    assert_close(last, 1.0);
    for (field, fraction) in [("zquarter", 0.25), ("zhalf", 0.5), ("zthreequarter", 0.75)] {
        let point = curve.value(first + fraction * (last - first)).unwrap();
        assert_close(point.z(), expected_real("remove_loop_mode0_spatial", field));
    }
}

fn spatial_loop_fixture() -> (Handle<ShapeAnalysisWire>, Edge) {
    let face = planar_face();
    let (first, last) = loop_vertices();
    let poles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(3.0, 3.0, 10.0),
        Point3::new(-3.0, 3.0, 10.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let curve_3d: Handle<dyn GeomCurve> =
        Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast();
    let curve_2d = loop_curve();
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &curve_3d, 1e-7)
        .unwrap();
    Builder
        .add(&mut edge, &first.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut edge, &last.oriented(Orientation::Reversed))
        .unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &curve_2d, &face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, &face, 0.0, 1.0).unwrap();
    builder.range(&edge, 0.0, 1.0, true).unwrap();
    builder.same_range(&edge, false).unwrap();
    builder.same_parameter(&edge, false).unwrap();
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    (Handle::new(analyzer), edge)
}

fn loop_fixture() -> (Handle<ShapeAnalysisWire>, Vertex, Vertex) {
    let face = planar_face();
    let (first, last) = loop_vertices();
    let edge = loop_edge(&face, &first, &last);
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    (Handle::new(analyzer), first, last)
}

fn removable_loop_fixture() -> (Handle<ShapeAnalysisWire>, Face, Vertex, Vertex) {
    let face = planar_face();
    let (first, last) = loop_vertices();
    let edge = loop_edge_with_3d(&face, &first, &last);
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    (Handle::new(analyzer), face, first, last)
}

fn loop_vertices() -> (Vertex, Vertex) {
    let builder = BRepBuilder::default();
    let mut first = Vertex::default();
    let mut last = Vertex::default();
    builder
        .make_vertex_at(&mut first, Point3::default(), 1e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last, Point3::new(1.0, 0.0, 0.0), 1e-7)
        .unwrap();
    (first, last)
}

fn planar_face() -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn loop_edge(face: &Face, first: &Vertex, last: &Vertex) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    Builder
        .add(&mut edge, &first.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut edge, &last.oriented(Orientation::Reversed))
        .unwrap();
    let poles = [
        Point2::new(0.0, 0.0),
        Point2::new(3.0, 3.0),
        Point2::new(-3.0, 3.0),
        Point2::new(1.0, 0.0),
    ];
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast();
    builder
        .update_edge_pcurve_on_face(&edge, &curve, face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, face, 0.0, 1.0).unwrap();
    edge
}

fn loop_edge_with_3d(face: &Face, first: &Vertex, last: &Vertex) -> Edge {
    let curve_2d = loop_curve();
    let curve_3d: Handle<dyn GeomCurve> = to_3d(Axis2::default(), &curve_2d).unwrap();
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &curve_3d, 1e-7)
        .unwrap();
    Builder
        .add(&mut edge, &first.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut edge, &last.oriented(Orientation::Reversed))
        .unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &curve_2d, face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, face, 0.0, 1.0).unwrap();
    builder.range(&edge, 0.0, 1.0, true).unwrap();
    builder.same_range(&edge, false).unwrap();
    builder.same_parameter(&edge, false).unwrap();
    edge
}

fn loop_curve() -> Handle<dyn Geom2dCurve> {
    let poles = [
        Point2::new(0.0, 0.0),
        Point2::new(3.0, 3.0),
        Point2::new(-3.0, 3.0),
        Point2::new(1.0, 0.0),
    ];
    Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}

fn expected(label: &str) -> f64 {
    expected_real("fix", label)
}

fn expected_real(row_name: &str, field_name: &str) -> f64 {
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(row_name))
        .unwrap();
    let mut values = row.split_whitespace();
    while values.next() != Some(field_name) {}
    values.next().unwrap().parse().unwrap()
}

fn expected_integer(row_name: &str, field_name: &str) -> i32 {
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(row_name))
        .unwrap();
    let mut values = row.split_whitespace();
    while values.next() != Some(field_name) {}
    values.next().unwrap().parse().unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-14, "{actual} != {expected}");
}
