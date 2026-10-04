use super::*;
use rustycad_tkbo::{bop_tools, int_tools::Curve};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
#[test]
fn native_intersection_pcurve_installation_matches() {
    for line in include_str!("../../../../tools/oracle/tkbo_make_section_pcurve.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let actual = installation(&row);
        assert_eq!(actual.len(), row.len() - 4, "{line}: {actual:?}");
        for (index, (actual, expected)) in actual.iter().zip(&row[4..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-11,
                "{line}: column {index}, actual {actual}"
            );
        }
    }
}
fn installation(row: &[f64]) -> Vec<f64> {
    let (mut fixture, edge) = edge_fixture(row[0] as usize);
    let mut second = super::fixture(row[0] as usize);
    let orientations = [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ];
    fixture.face.set_orientation(orientations[row[3] as usize]);
    second
        .face
        .set_orientation(orientations[(row[3] as usize + 1) % 4]);
    let curve = intersection_curve(&fixture, row[2] != 0.);
    let builder = BRepBuilder::default();
    builder.same_parameter(&edge, false).unwrap();
    builder.same_range(&edge, false).unwrap();
    let mask = row[1] as usize;
    let result = bop_tools::make_pcurve(
        &edge,
        &fixture.face,
        &second.face,
        &curve,
        mask & 1 != 0,
        mask & 2 != 0,
        Some(&Context::default()),
    );
    let mut actual = vec![
        if result.is_ok() { 0. } else { -1. },
        BRepTool::edge_tolerance(&edge).unwrap(),
        f64::from(BRepTool::same_parameter(&edge).unwrap()),
        f64::from(BRepTool::same_range(&edge).unwrap()),
    ];
    append_face(&mut actual, &edge, &fixture.face);
    append_face(&mut actual, &edge, &second.face);
    actual
}
fn intersection_curve(fixture: &ProjectionFixture, supplied: bool) -> Curve {
    let mut curve = Curve::default();
    curve.set_curve(&fixture.curve);
    if !supplied {
        return curve;
    }
    let mut pcurve = Handle::default();
    let mut tolerance = 1e-7;
    algo_tools2d::make_pcurve_on_face(
        &fixture.face,
        &fixture.curve,
        fixture.range,
        &mut pcurve,
        &mut tolerance,
        None,
    )
    .unwrap();
    curve.set_first_curve2d(&pcurve);
    curve.set_second_curve2d(&pcurve);
    curve
}
fn append_face(actual: &mut Vec<f64>, edge: &Edge, face: &Face) {
    let found = BRepTool::curve_on_face(edge, face).unwrap();
    actual.extend([f64::from(found.pcurve.is_null()), found.first, found.last]);
    if !found.pcurve.is_null() {
        let uv = found
            .pcurve
            .value((found.first + found.last) * 0.5)
            .unwrap();
        actual.extend([uv.x(), uv.y()]);
    }
}
