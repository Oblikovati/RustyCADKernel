//! Native missing-connection statuses, tolerances, orientation and metric state.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Builder, Edge, Face, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};

const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_lacking.expected");

#[test]
fn missing_connection_failure_branches_match_native() {
    inspect("missing_vertices", 0, 1., 0., false, false);
    inspect("disconnected_vertices", 2, 1., 0., false, false);
    inspect("missing_pcurve", 1, -1., 0., false, false);
}

#[test]
fn missing_connection_tolerance_and_reversal_match_native() {
    inspect("small_gap", 1, 1e-8, 0., false, false);
    inspect("large_gap", 1, 1., 0., false, false);
    inspect("reversed_gap", 1, 1., 0., true, false);
    inspect("small_caller_tolerance", 1, 1e-8, 1e-9, false, false);
}

#[test]
fn missing_connection_aggregate_statuses_match_native() {
    inspect("aggregate_gap", 1, 1., 0., false, true);
    inspect("aggregate_missing_pcurve", 1, -1., 0., false, true);
}

fn inspect(name: &str, vertices: u8, span: f64, tolerance: f64, reverse: bool, doubled: bool) {
    let face = planar_face();
    let edge = fixture_edge(&face, vertices, span, reverse);
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    if doubled {
        wire.add_edge(&edge, 0).unwrap();
    }
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    let found = analyzer.check_lacking_at(0, tolerance).unwrap().0;
    assert_eq!(analyzer.check_lacking_edge(0, tolerance).unwrap(), found);
    let mut observed = observed_flags(&analyzer, found);
    observed.push(analyzer.max_distance_2d());
    observed.push(analyzer.max_distance_3d());
    append_aggregate_observation(&analyzer, &mut observed);
    assert_native_row(name, &observed);
}

fn observed_flags(analyzer: &ShapeAnalysisWire, found: bool) -> Vec<f64> {
    let statuses = [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
    ];
    let mut observed = vec![f64::from(found)];
    observed.extend(statuses.map(|status| f64::from(analyzer.last_check_status(status))));
    observed
}

fn append_aggregate_observation(analyzer: &ShapeAnalysisWire, observed: &mut Vec<f64>) {
    observed.push(f64::from(analyzer.check_lacking().unwrap()));
    observed.push(f64::from(analyzer.status_lacking(ShapeExtendStatus::Done)));
    observed.push(f64::from(analyzer.status_lacking(ShapeExtendStatus::Fail)));
    analyzer.check_connected_at(1, 0.).unwrap();
    observed.push(analyzer.max_distance_3d());
    analyzer.clear_statuses();
    observed.push(f64::from(analyzer.status_lacking(ShapeExtendStatus::Ok)));
}

fn assert_native_row(name: &str, observed: &[f64]) {
    let row = EXPECTED
        .lines()
        .find(|row| row.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(observed, expected, "native missing-connection row {name}");
}

fn planar_face() -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn fixture_edge(face: &Face, vertices: u8, span: f64, reverse: bool) -> Edge {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    if vertices > 0 {
        attach_vertices(&edge, vertices == 2);
    }
    if span >= 0. {
        attach_pcurve(&edge, face, span);
    }
    if reverse {
        edge.reverse();
    }
    edge
}

fn attach_vertices(edge: &Edge, disconnected: bool) {
    let mut shape = edge.clone().into_shape();
    for (x, orientation) in [
        (0., Orientation::Forward),
        (f64::from(disconnected), Orientation::Reversed),
    ] {
        let mut vertex = Vertex::default();
        BRepBuilder::default()
            .make_vertex_at(&mut vertex, Point3::new(x, 0., 0.), 1e-7)
            .unwrap();
        Builder
            .add(&mut shape, &vertex.oriented(orientation))
            .unwrap();
    }
}

fn attach_pcurve(edge: &Edge, face: &Face, span: f64) {
    let frame = Axis2d::new(Point2::default(), Direction2::new(1., 0.).unwrap());
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(frame)).cast();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(edge, &curve, face, 1e-7)
        .unwrap();
    builder.range_on_face(edge, face, 0., span).unwrap();
}

#[test]
fn failed_gap_checks_discard_previous_distance() {
    use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
    use rustycad_tkmath::gp::Axis1;
    let face = planar_face();
    let edge = fixture_edge(&face, 1, 1., false);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    let builder = BRepBuilder::default();
    builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
    builder.range(&edge, 0., 2., true).unwrap();
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    analyzer.check_gap_3d_at(1).unwrap();
    let prior = analyzer.max_distance_3d();
    builder
        .update_edge_curve(&edge, &Handle::default(), 1e-7)
        .unwrap();
    let found = analyzer.check_gaps_3d().unwrap();
    assert_native_row(
        "failed_gaps_discard_stale_distance",
        &[
            prior,
            f64::from(found),
            analyzer.min_distance_3d(),
            analyzer.max_distance_3d(),
        ],
    );
}

#[test]
fn empty_wire_with_a_valid_face_has_no_gap_failure() {
    let wire = Handle::new(ShapeExtendWireData::new());
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &planar_face(), 1e-7).unwrap();
    let found = analyzer.check_gap_2d_at(1).unwrap();
    assert_native_row(
        "empty_wire_on_face",
        &[
            f64::from(found),
            f64::from(analyzer.last_check_status(ShapeExtendStatus::Ok)),
        ],
    );
}
