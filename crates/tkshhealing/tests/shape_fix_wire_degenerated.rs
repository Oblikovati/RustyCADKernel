//! Native parity for invalid marked-degenerate edge removal.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisWire;
use rustycad_tkshhealing::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkshhealing::shape_fix::ShapeFixWire;

const NATIVE_RESULT: &str =
    include_str!("../../../tools/oracle/tkshhealing_wire_degenerated.expected");

#[test]
fn invalid_marked_degenerated_edge_is_removed_with_native_status() {
    let (analyzer, data) = invalid_marked_degenerated_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_degenerated_at(1).unwrap());
    assert_eq!(data.nb_edges(), 0);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done3));
    assert!(analyzer.last_check_status(ShapeExtendStatus::Fail2));
    assert_eq!(
        NATIVE_RESULT.lines().next(),
        Some("changed=1 count=0 done3=1 fail2=1")
    );
}

#[test]
fn closed_edge_at_cone_apex_is_replaced_as_degenerated() {
    let (analyzer, data) = cone_apex_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_degenerated_at(1).unwrap());
    let repaired = data.edge(1).unwrap();
    assert!(BRepTool::degenerated(&repaired).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert_eq!(
        BRepTool::range_on_face(&repaired, &analyzer.face()).unwrap(),
        (0.0, 2.0 * std::f64::consts::PI)
    );
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| line == "replace=1 count=1 degenerated=1 done2=1")
    );
}

#[test]
fn bulk_degenerated_fix_tracks_single_edge_replacement_status() {
    let (analyzer, data) = cone_apex_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_degenerated().unwrap());
    assert_eq!(data.nb_edges(), 1);
    assert!(BRepTool::degenerated(&data.edge(1).unwrap()).unwrap());
    assert!(fixer.status_degenerated(ShapeExtendStatus::Done2));
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| line == "bulk=1 count=1 done2=1")
    );
}

#[test]
fn unready_fix_preserves_the_analyzer_check_status() {
    let (analyzer, _) = invalid_marked_degenerated_wire();
    analyzer.check_degenerated_at(1).unwrap();
    assert!(analyzer.last_check_status(ShapeExtendStatus::Fail2));
    analyzer.set_face(&Face::default()).unwrap();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(!fixer.fix_degenerated_at(1).unwrap());
    assert!(analyzer.last_check_status(ShapeExtendStatus::Fail2));
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| { line == "unready=0 prior_fail2=1 after_fail2=1" })
    );
}

#[test]
fn duplicate_degenerated_edges_at_wire_wrap_are_collapsed() {
    let (analyzer, data) = cone_apex_wire();
    let edge = data.edge(1).unwrap();
    data.add_edge(&edge, 0).unwrap();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_degenerated().unwrap());
    assert_eq!(data.nb_edges(), 1);
    assert!(BRepTool::degenerated(&data.edge(1).unwrap()).unwrap());
    assert!(fixer.status_degenerated(ShapeExtendStatus::Done2));
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| { line == "duplicate=1 count=1 degenerated=1 status2=1" })
    );
}

fn cone_apex_wire() -> (Handle<ShapeAnalysisWire>, Handle<ShapeExtendWireData>) {
    let face = cone_face();
    let edge = cone_apex_edge(&face);
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&edge, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&data, &face, 1e-7).unwrap());
    (analyzer, data)
}

fn cone_face() -> Face {
    let cone = Handle::new(GeomConicalSurface::new(Axis3::default(), 0.5, 0.0).unwrap());
    let surface: Handle<dyn GeomSurface> = cone.cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn cone_apex_edge(face: &Face) -> Edge {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::default(), 1e-7)
        .unwrap();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    add_edge_vertex(&mut edge, &vertex);
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::new(1.0, 0.0).unwrap(),
    )))
    .cast();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, face, 1e-7)
        .unwrap();
    builder
        .range_on_face(&edge, face, 0.0, 2.0 * std::f64::consts::PI)
        .unwrap();
    edge
}

fn invalid_marked_degenerated_wire() -> (Handle<ShapeAnalysisWire>, Handle<ShapeExtendWireData>) {
    let face = plane_face();
    let edge = marked_degenerated_edge();
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&edge, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&data, &face, 1e-7).unwrap());
    (analyzer, data)
}

fn plane_face() -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn marked_degenerated_edge() -> Edge {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(1.0, 0.0, 0.0), 1e-7)
        .unwrap();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    builder.degenerated(&edge, true).unwrap();
    add_edge_vertex(&mut edge, &vertex);
    edge
}

fn add_edge_vertex(edge: &mut Edge, vertex: &Vertex) {
    let builder = Builder;
    let first = vertex
        .clone()
        .oriented(rustycad_tkg3d::top_abs::Orientation::Forward);
    let last = vertex
        .clone()
        .oriented(rustycad_tkg3d::top_abs::Orientation::Reversed);
    let mut shape = edge.clone().into_shape();
    builder.add(&mut shape, &first).unwrap();
    builder.add(&mut shape, &last).unwrap();
    *edge = topods::edge(&shape).unwrap();
}
