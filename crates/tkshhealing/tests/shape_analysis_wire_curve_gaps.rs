//! Native sampled curve-gap distances, retained failure metrics and aggregate status.
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis1, Axis2d, Axis3, Direction3, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};

const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_curve_gaps.expected");

#[test]
fn curve_gap_measurements_and_statuses_match_native() {
    inspect("coincident", 0., 1., false);
    inspect("offset", 3., 1., false);
    inspect("different_ranges", 0., 2., false);
    inspect("reversed", 3., 2., true);
    inspect("below_precision", 1e-8, 1., false);
}

fn inspect(name: &str, offset: f64, span: f64, reverse: bool) {
    let face = planar_face();
    let mut edge = edge_on(&face, offset, span);
    if reverse {
        edge.set_orientation(Orientation::Reversed);
    }
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    let mut observed = vec![
        f64::from(analyzer.check_curve_gap_at(0).unwrap()),
        analyzer.max_distance_3d(),
    ];
    BRepBuilder::default()
        .update_edge_curve(&edge, &Handle::default(), 1e-7)
        .unwrap();
    observed.extend([
        f64::from(analyzer.check_curve_gap_at(1).unwrap()),
        f64::from(analyzer.last_check_status(ShapeExtendStatus::Fail1)),
        analyzer.max_distance_3d(),
    ]);
    wire.add_edge(&edge_on(&face, 2., 1.), 0).unwrap();
    append_aggregate(&analyzer, &mut observed);
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(observed, expected, "native curve-gap case {name}");
}

fn append_aggregate(analyzer: &ShapeAnalysisWire, observed: &mut Vec<f64>) {
    observed.extend([
        f64::from(analyzer.check_curve_gaps().unwrap()),
        f64::from(analyzer.status_curve_gaps(ShapeExtendStatus::Done1)),
        f64::from(analyzer.status_curve_gaps(ShapeExtendStatus::Fail1)),
        analyzer.max_distance_3d(),
    ]);
    analyzer.clear_statuses();
    observed.extend([
        f64::from(analyzer.status_curve_gaps(ShapeExtendStatus::Ok)),
        analyzer.max_distance_3d(),
    ]);
}

fn planar_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn edge_on(face: &Face, offset: f64, span: f64) -> Edge {
    let edge = support::edge_at_points(Point3::default(), Point3::new(1., 0., 0.));
    let builder = BRepBuilder::default();
    let axis = Axis1::new(
        Point3::new(0., 0., offset),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
    builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
    builder.range(&edge, 0., 1., true).unwrap();
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, face, 0., span).unwrap();
    edge
}

#[test]
fn unloaded_curve_checks_return_false_and_reset_status() {
    let analyzer = ShapeAnalysisWire::new();
    assert!(!analyzer.check_curve_gap_at(0).unwrap());
    assert!(analyzer.last_check_status(ShapeExtendStatus::Ok));
    assert!(!analyzer.check_curve_gaps().unwrap());
    assert!(analyzer.status_curve_gaps(ShapeExtendStatus::Ok));
    assert!(!analyzer.check_edge_curves().unwrap());
    assert!(analyzer.status_edge_curves(ShapeExtendStatus::Ok));
}

#[test]
fn edge_curve_consistency_statuses_match_native() {
    inspect_consistency("consistency_coincident", 0., 1., false);
    inspect_consistency("consistency_offset", 3., 1., false);
    inspect_consistency("consistency_ranges", 0., 2., false);
    inspect_consistency("consistency_missing", 0., 1., true);
}

fn inspect_consistency(name: &str, offset: f64, span: f64, missing: bool) {
    let face = planar_face();
    let edge = edge_on(&face, offset, span);
    if missing {
        BRepBuilder::default()
            .update_edge_curve(&edge, &Handle::default(), 1e-7)
            .unwrap();
    }
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    let observed = consistency_observation(&analyzer);
    let row = EXPECTED
        .lines()
        .find(|row| row.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(observed, expected, "native consistency case {name}");
}

fn consistency_observation(analyzer: &ShapeAnalysisWire) -> Vec<f64> {
    use ShapeExtendStatus::*;
    let mut observed = vec![f64::from(analyzer.check_edge_curves().unwrap())];
    for status in [
        Done1, Done2, Done3, Done4, Done5, Done6, Done7, Fail1, Fail2, Fail3, Fail4, Fail5, Fail6,
        Fail7,
    ] {
        observed.push(f64::from(analyzer.status_edge_curves(status)));
    }
    observed.extend([
        f64::from(analyzer.last_check_status(Done1)),
        f64::from(analyzer.last_check_status(Fail1)),
        analyzer.min_distance_3d(),
        analyzer.min_distance_2d(),
    ]);
    analyzer.clear_statuses();
    observed.push(f64::from(analyzer.status_edge_curves(Ok)));
    observed
}
