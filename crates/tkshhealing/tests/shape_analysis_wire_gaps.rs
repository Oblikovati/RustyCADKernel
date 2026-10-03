//! Pinned native gap metrics, statuses and parameter-space tolerances.
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Axis2d, Direction2, Direction3, Point2, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_gaps.expected");

#[test]
fn boundary_gaps_match_native_metric_and_status_snapshots() {
    for (mode, row) in EXPECTED.lines().enumerate() {
        let analyzer = fixture_analyzer(mode);
        let observed = observations(&analyzer, mode);
        let expected: Vec<f64> = row
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(observed, expected, "native boundary gap case {row}");
    }
}

fn observations(analyzer: &ShapeAnalysisWire, mode: usize) -> Vec<f64> {
    let mut observed = Vec::new();
    for index in [2, 1, 0, -1] {
        append_pair(&mut observed, analyzer, index);
    }
    remove_first_representation(analyzer, mode);
    append_pair(&mut observed, analyzer, 2);
    observed.push(f64::from(analyzer.check_gaps_3d().unwrap()));
    append_snapshot(&mut observed, analyzer, true, true);
    observed.push(f64::from(analyzer.check_gaps_2d().unwrap()));
    append_snapshot(&mut observed, analyzer, true, false);
    analyzer.clear_statuses();
    observed.extend([
        f64::from(analyzer.status_gaps_3d(ShapeExtendStatus::Ok)),
        f64::from(analyzer.status_gaps_2d(ShapeExtendStatus::Ok)),
    ]);
    observed
}

fn append_pair(observed: &mut Vec<f64>, analyzer: &ShapeAnalysisWire, index: i32) {
    observed.push(f64::from(analyzer.check_gap_3d_at(index).unwrap()));
    append_snapshot(observed, analyzer, false, true);
    observed.push(f64::from(analyzer.check_gap_2d_at(index).unwrap()));
    append_snapshot(observed, analyzer, false, false);
}

fn remove_first_representation(analyzer: &ShapeAnalysisWire, mode: usize) {
    if mode == 10 {
        BRepBuilder::default()
            .update_edge_curve(
                &analyzer.wire_data().edge(1).unwrap(),
                &Handle::default(),
                1e-7,
            )
            .unwrap();
    }
    if mode == 11 {
        BRepBuilder::default()
            .update_edge_pcurve_on_face(
                &analyzer.wire_data().edge(1).unwrap(),
                &Handle::default(),
                &analyzer.face(),
                1e-7,
            )
            .unwrap();
    }
}

fn append_snapshot(
    observed: &mut Vec<f64>,
    analyzer: &ShapeAnalysisWire,
    aggregate: bool,
    three: bool,
) {
    for flag in [ShapeExtendStatus::Done1, ShapeExtendStatus::Fail1] {
        let value = match (aggregate, three) {
            (false, _) => analyzer.last_check_status(flag),
            (true, true) => analyzer.status_gaps_3d(flag),
            (true, false) => analyzer.status_gaps_2d(flag),
        };
        observed.push(f64::from(value));
    }
    observed.extend(if three {
        [analyzer.min_distance_3d(), analyzer.max_distance_3d()]
    } else {
        [analyzer.min_distance_2d(), analyzer.max_distance_2d()]
    });
}

fn fixture_analyzer(mode: usize) -> ShapeAnalysisWire {
    let face = match mode {
        7 => Face::default(),
        4 | 6 | 11 => support::cylindrical_face(0.25),
        _ => support::planar_face(),
    };
    let gap = match mode {
        0 => 0.,
        1 => 0.00001,
        _ => 0.0002,
    };
    let first = fixture_edge(&face, 0., mode);
    let mut second = fixture_edge(&face, 1. + gap, mode);
    if mode == 3 {
        second = Edge::unchecked(second.reversed());
    }
    let wire = Handle::new(ShapeExtendWireData::new());
    if mode != 8 {
        wire.add_edge(&first, 0).unwrap();
        wire.add_edge(&second, 0).unwrap();
    }
    ShapeAnalysisWire::from_wire_data(&wire, &face, if mode == 9 { f64::NAN } else { 0.0001 })
        .unwrap()
}

fn fixture_edge(face: &Face, start: f64, mode: usize) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    if mode != 5 {
        attach_line_3d(&edge, start);
    }
    if mode != 6 && mode != 12 && !face.is_null() {
        let axis = Axis2d::new(Point2::new(start, 0.), Direction2::new(1., 0.).unwrap());
        let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(axis)).cast();
        builder
            .update_edge_pcurve_on_face(&edge, &curve, face, 1e-7)
            .unwrap();
        builder.range_on_face(&edge, face, 0., 1.).unwrap();
    }
    edge
}

fn attach_line_3d(edge: &Edge, start: f64) {
    let line = GeomLine::from_point(
        Point3::new(start, 0., 0.),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let curve: Handle<dyn GeomCurve> = Handle::new(line).cast();
    BRepBuilder::default()
        .update_edge_curve(edge, &curve, 1e-7)
        .unwrap();
    BRepBuilder::default().range(edge, 0., 1., true).unwrap();
}
