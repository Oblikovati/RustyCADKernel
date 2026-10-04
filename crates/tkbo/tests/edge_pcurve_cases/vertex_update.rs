use super::*;
use rustycad_tkbo::{bop_tools, int_tools::Curve};
use rustycad_tkbrep::{brep::BRepTool, topods::Vertex};
use rustycad_tkmath::gp::Point3;
fn parsed(line: &str) -> Vec<f64> {
    line.split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
}
fn check(line: &str, actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-14, "{line}: actual {actual}");
    }
}
#[test]
fn native_vertex_sphere_updates_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_vertex_update_pair.expected").lines() {
        let row = parsed(line);
        let mut first = Vertex::default();
        let mut second = Vertex::default();
        let builder = BRepBuilder::default();
        builder
            .make_vertex_at(
                &mut first,
                Point3::default(),
                [1e-7, 0.01, 1.][row[2] as usize],
            )
            .unwrap();
        builder
            .make_vertex_at(
                &mut second,
                Point3::new([0., 1e-7, 0.01][row[1] as usize], 0., 0.),
                1e-7,
            )
            .unwrap();
        bop_tools::update_vertex_pair(&first, &second).unwrap();
        let point = BRepTool::pnt(&first).unwrap();
        check(
            line,
            &[
                BRepTool::vertex_tolerance(&first).unwrap(),
                BRepTool::vertex_tolerance(&second).unwrap(),
                point.x(),
                point.y(),
                point.z(),
            ],
            &row[3..],
        );
    }
}
#[test]
fn native_edge_and_intersection_curve_updates_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_vertex_update.expected").lines() {
        let row = parsed(line);
        let actual = curve_update(&row);
        check(line, &actual, &row[4..]);
    }
}
fn curve_update(row: &[f64]) -> [f64; 4] {
    let (fixture, edge) = edge_fixture(row[0] as usize);
    let parameter = fixture.range.0 + (fixture.range.1 - fixture.range.0) * row[2] / 2.;
    let point = fixture.curve.value(parameter).unwrap();
    let point = Point3::new(point.x(), point.y(), point.z() + 0.01);
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, [1e-7, 0.01, 1.][row[3] as usize])
        .unwrap();
    if row[1] == 0. {
        bop_tools::update_vertex_edge(&edge, parameter, &vertex).unwrap();
    } else {
        let mut curve = Curve::default();
        curve.set_curve(&fixture.curve);
        bop_tools::update_vertex_curve(&curve, parameter, &vertex).unwrap();
    }
    let point = BRepTool::pnt(&vertex).unwrap();
    [
        BRepTool::vertex_tolerance(&vertex).unwrap(),
        point.x(),
        point.y(),
        point.z(),
    ]
}
