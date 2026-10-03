//! Native tail overlap, edge splitting, orientation and retained output parts.
mod support;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::Edge,
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_tail.expected");

#[test]
fn tail_overlap_and_parts_match_pinned_native() {
    for (mode, row) in EXPECTED.lines().enumerate() {
        inspect_tail_case(mode, row);
    }
}

fn inspect_tail_case(mode: usize, row: &str) {
    let (analyzer, first, second) = fixture_analyzer(mode);
    let sentinel = fixture_edge(30., 1., 0., 5., true);
    let mut parts: [Edge; 4] = std::array::from_fn(|_| sentinel.clone());
    let max_sine = if mode == 6 { -1. } else { 0.1 };
    analyzer.check_small_at(1, 100.).unwrap();
    let result = analyzer
        .check_tail(&first, &second, max_sine, 0.01, 1e-3, &mut parts)
        .unwrap();
    let mut observed = vec![f64::from(result)];
    for part in &parts {
        append_part(&mut observed, part, [&first, &second, &sentinel]);
    }
    for flag in [
        ShapeExtendStatus::Ok,
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
    ] {
        observed.push(f64::from(analyzer.last_check_status(flag)));
    }
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(observed, expected, "native tail case {row}");
}

fn append_part(observed: &mut Vec<f64>, part: &Edge, original: [&Edge; 3]) {
    let (first, last) = BRepTool::range(part).unwrap();
    observed.extend(original.map(|edge| f64::from(part.is_same(edge))));
    observed.extend([
        part.orientation() as u8 as f64,
        first,
        last,
        f64::from(BRepTool::same_parameter(part).unwrap()),
        f64::from(BRepTool::same_range(part).unwrap()),
    ]);
    append_part_vertices(observed, part);
}

fn append_part_vertices(observed: &mut Vec<f64>, part: &Edge) {
    let (first, last) = rustycad_tkbrep::topexp::vertices(part, false);
    for vertex in [first, last] {
        let point = BRepTool::pnt(&vertex).unwrap();
        observed.extend([
            point.x(),
            point.y(),
            point.z(),
            BRepTool::vertex_tolerance(&vertex).unwrap(),
        ]);
    }
}

fn fixture_analyzer(mode: usize) -> (ShapeAnalysisWire, Edge, Edge) {
    let face = support::surfaces::planar_face();
    let (first, second) = fixture_pair(mode);
    let wire = Handle::new(ShapeExtendWireData::new());
    if mode != 10 {
        wire.add_edge(&first, 0).unwrap();
        wire.add_edge(&second, 0).unwrap();
    }
    (
        ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap(),
        first,
        second,
    )
}

fn fixture_pair(mode: usize) -> (Edge, Edge) {
    if mode >= 11 {
        return curved_pair(mode == 12);
    }
    if mode == 3 {
        return (
            Edge::unchecked(fixture_edge(2., -1., 0., 2., true).reversed()),
            Edge::unchecked(fixture_edge(1., 1., 0., 1., true).reversed()),
        );
    }
    let first = fixture_edge(0., 1., 0., if mode == 9 { 0. } else { 2. }, mode != 7);
    let origin = if mode == 4 {
        3.
    } else if mode == 9 {
        0.
    } else {
        2.
    };
    let direction = if matches!(mode, 5 | 6) { 0.2 } else { 0. };
    let length = if mode == 1 {
        1.
    } else if mode == 2 {
        3.
    } else {
        2.
    };
    let second = fixture_edge(origin, -1., direction, length, true);
    if mode == 8 {
        BRepBuilder::default().degenerated(&first, true).unwrap();
    }
    (first, second)
}

fn fixture_edge(origin: f64, dx: f64, dy: f64, length: f64, geometry: bool) -> Edge {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(origin, 0., 0.),
        Direction3::new(dx, dy, 0.).unwrap(),
    ))
    .cast();
    curve_edge(&curve, 0., length, geometry)
}

fn curve_edge(curve: &Handle<dyn GeomCurve>, first: f64, last: f64, geometry: bool) -> Edge {
    let edge = support::edge_at_points(curve.value(first).unwrap(), curve.value(last).unwrap());
    if geometry {
        BRepBuilder::default()
            .update_edge_curve(&edge, curve, 1e-7)
            .unwrap();
        BRepBuilder::default()
            .range(&edge, first, last, true)
            .unwrap();
    }
    edge
}

fn curved_pair(shorter: bool) -> (Edge, Edge) {
    let poles = [
        Point3::new(0., 0., 0.),
        Point3::new(1., 2., 0.),
        Point3::new(2., 0., 0.),
    ];
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast();
    let first = curve_edge(&curve, 0., 1., true);
    let second = curve_edge(&curve, if shorter { 0.5 } else { 0. }, 1., true);
    (first, Edge::unchecked(second.reversed()))
}
