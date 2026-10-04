use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkmath::gp::Vector2;
#[test]
fn native_curve_tolerance_outputs_and_exceptions_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_curve_tolerance.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (fixture, edge) = edge_fixture(row[0] as usize);
        let mode = row[1] as usize;
        if mode != 0 {
            store_test_pcurve(&fixture, &edge, mode);
        }
        let (mut distance, mut parameter) = (99., 88.);
        let result =
            bop_tools::compute_curve_tolerance(&fixture.face, &edge, &mut distance, &mut parameter);
        if row[2] == -1. {
            assert_eq!(
                result.unwrap_err().exception_type(),
                "Standard_NullObject",
                "{line}"
            );
        } else {
            assert_eq!(result.unwrap(), row[2] != 0., "{line}");
        }
        assert!(
            (distance - row[3]).abs() < 1e-12,
            "{line}: distance {distance}"
        );
        assert!(
            (parameter - row[4]).abs() < 1e-12,
            "{line}: parameter {parameter}"
        );
    }
}
fn store_test_pcurve(fixture: &ProjectionFixture, edge: &Edge, mode: usize) {
    algo_tools2d::build_pcurve_for_edge_on_face(edge, &fixture.face, None).unwrap();
    if mode != 2 {
        return;
    }
    let found = BRepTool::curve_on_face(edge, &fixture.face).unwrap();
    let copy: Handle<dyn Geom2dCurve> = found.pcurve.copy().unwrap().cast();
    copy.translate(Vector2::new(0.01, 0.02)).unwrap();
    BRepBuilder::default()
        .update_edge_pcurve_on_face(edge, &copy, &fixture.face, 1e-7)
        .unwrap();
}
