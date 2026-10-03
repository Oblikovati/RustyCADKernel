//! Native tolerance and status behavior for `ShapeFix_Wire::FixLacking`.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{self, Builder, Edge, Face, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
    shape_fix::{FixRoot, ShapeFixWire},
};

const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_fix_lacking.expected");

#[test]
fn tolerance_increase_and_status_match_native() {
    let (analyzer, vertex) = narrow_gap_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(0.01);
    assert!(fixer.fix_lacking_at(1, false).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Fail));
    assert_eq!(BRepTool::vertex_tolerance(&vertex).unwrap(), 0.0005005);
    assert!(EXPECTED.lines().any(|row| row.contains("single=1 done1=1")));
}

#[test]
fn aggregate_status_tracks_native_no_op_after_vertex_tolerance_is_repaired() {
    let (analyzer, _) = narrow_gap_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(0.01);
    assert!(fixer.fix_lacking_at(1, false).unwrap());
    assert!(!fixer.fix_lacking(false).unwrap());
    assert!(!fixer.status_lacking(ShapeExtendStatus::Done));
    assert!(EXPECTED.lines().any(|row| row == "aggregate=0 count=1"));
}

#[test]
fn topology_mode_inserts_closed_edge_when_existing_vertices_are_off_surface() {
    let (analyzer, _, wire) = topology_gap_wire(1e-8);
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_topology_mode(true);
    fixer.set_max_tolerance(1e-4);
    assert!(fixer.fix_lacking_at(1, false).unwrap());
    assert_eq!(wire.nb_edges(), 2);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done4));
    assert!(
        EXPECTED
            .lines()
            .any(|row| row == "topology=1 count=2 done2=1 done4=1 fail3=0")
    );
}

#[test]
fn topological_backtracking_curve_matches_native_add_edge_status() {
    let (analyzer, _, wire) = topology_gap_wire(0.001);
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_topology_mode(true);
    fixer.set_max_tolerance(1e-4);
    assert!(fixer.fix_lacking_at(1, false).unwrap());
    assert_eq!(wire.nb_edges(), 2);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done4));
    assert!(
        EXPECTED
            .lines()
            .any(|row| row == "long=1 count=2 done1=0 done2=1 done4=1 fail1=0")
    );
}

#[test]
fn midpoint_at_shared_vertex_inserts_degenerated_edge() {
    let face = plane_face();
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(0.0005, 0., 0.), 1e-7)
        .unwrap();
    let edge = make_edge(&face, &vertex, None);
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap());
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(1e-4);
    assert!(fixer.fix_lacking_at(1, false).unwrap());
    assert_eq!(wire.nb_edges(), 2);
    let has_degenerate = (1..=wire.nb_edges())
        .any(|index| BRepTool::degenerated(&wire.edge(index).unwrap()).unwrap());
    assert!(has_degenerate);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done3));
    assert!(
        EXPECTED
            .lines()
            .any(|row| row == "degenerated=1 count=2 done1=0 done2=1 done3=1 fail2=0")
    );
}

#[test]
fn unsuccessful_curve_bend_falls_back_to_native_tolerance_growth() {
    let (analyzer, vertex, wire) = topology_gap_wire(0.001);
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.set_max_tolerance(0.01);
    assert!(fixer.fix_lacking_at(1, false).unwrap());
    assert_eq!(wire.nb_edges(), 1);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Done5));
    assert_eq!(BRepTool::vertex_tolerance(&vertex).unwrap(), 0.0005005);
    assert!(
        EXPECTED
            .lines()
            .any(|row| row.starts_with("bent=1 count=1 done1=1"))
    );
}

#[test]
fn analyzer_failure_is_transferred_without_inventing_a_repair_failure() {
    let face = plane_face();
    let edge = edge_without_vertices(&face);
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap());
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(!fixer.fix_lacking_at(1, false).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Fail1));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Fail2));
    assert!(
        EXPECTED
            .lines()
            .any(|row| row == "failure=0 fail1=1 fail2=0")
    );
}

#[test]
fn maximum_tolerance_failure_matches_native_status() {
    let face = plane_face();
    let vertex = make_vertex();
    let edge = edge_with_span(&face, &vertex, None, 1.0);
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap());
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(!fixer.fix_lacking_at(1, false).unwrap());
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Fail1));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Fail2));
    assert!(
        EXPECTED
            .lines()
            .any(|row| row == "maximum_tolerance=0 fail1=0 fail2=1")
    );
}

fn narrow_gap_wire() -> (Handle<ShapeAnalysisWire>, Vertex) {
    let face = plane_face();
    let vertex = make_vertex();
    let edge = make_edge(&face, &vertex, None);
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    (Handle::new(analyzer), vertex)
}

fn topology_gap_wire(
    curve_range: f64,
) -> (
    Handle<ShapeAnalysisWire>,
    Vertex,
    Handle<ShapeExtendWireData>,
) {
    let face = plane_face();
    let vertex = make_vertex();
    let edge = make_edge(&face, &vertex, Some(curve_range));
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    (Handle::new(analyzer), vertex, wire)
}

fn plane_face() -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn make_vertex() -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::default(), 1e-7)
        .unwrap();
    vertex
}

fn make_edge(face: &Face, vertex: &Vertex, curve_range: Option<f64>) -> Edge {
    edge_with_span(face, vertex, curve_range, 0.001)
}

fn edge_with_span(
    face: &Face,
    vertex: &Vertex,
    curve_range: Option<f64>,
    pcurve_span: f64,
) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let mut shape = edge.clone().into_shape();
    for orientation in [Orientation::Forward, Orientation::Reversed] {
        Builder
            .add(&mut shape, &vertex.oriented(orientation))
            .unwrap();
    }
    let edge = topods::edge(&shape).unwrap();
    if let Some(range) = curve_range {
        let curve: Handle<dyn GeomCurve> =
            Handle::new(GeomLine::new(rustycad_tkmath::gp::Axis1::default())).cast();
        builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
        builder.range(&edge, 0., range, true).unwrap();
    }
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::new(1., 0.).unwrap(),
    )))
    .cast();
    builder
        .update_edge_pcurve_on_face(&edge, &line, face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, face, 0., pcurve_span).unwrap();
    edge
}

fn edge_without_vertices(face: &Face) -> Edge {
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder.make_edge(&mut edge).unwrap();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::new(1., 0.).unwrap(),
    )))
    .cast();
    builder
        .update_edge_pcurve_on_face(&edge, &line, face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, face, 0., 0.001).unwrap();
    edge
}
