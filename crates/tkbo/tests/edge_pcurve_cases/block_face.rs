use super::*;
use rustycad_tkbo::{bop_tools, int_tools::Range};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::Vector3;
#[test]
fn native_edge_block_face_membership_and_projector_state_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_block_face.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let actual = block_observation(&row);
        assert_eq!(actual.len(), row.len() - 4, "{line}");
        for (index, (actual, expected)) in actual.iter().zip(&row[4..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-10,
                "{line}: column {index}, actual {actual}"
            );
        }
    }
}
fn block_observation(row: &[f64]) -> Vec<f64> {
    let fixture = fixture(row[0] as usize);
    let context = Context::default();
    let edge = block_edge(&fixture, row[2] as usize, row[3] != 0.);
    let range = block_range(fixture.range, row[1] as usize);
    let result = bop_tools::is_block_in_on_face(&range, &fixture.face, &edge, &context).unwrap();
    let cached = context.proj_ps(&fixture.face).unwrap();
    let projector = cached.read();
    let mut actual = vec![f64::from(result), f64::from(projector.is_done())];
    if projector.is_done() {
        let (u, v) = projector.lower_distance_parameters().unwrap();
        actual.extend([projector.lower_distance().unwrap(), u, v]);
    }
    actual
}
fn block_edge(fixture: &ProjectionFixture, shift: usize, large_tolerance: bool) -> Edge {
    let curve: Handle<dyn GeomCurve> = fixture.curve.copy().unwrap().cast();
    curve
        .translate(Vector3::new(0., 0., [0., 1e-6, 1.][shift]))
        .unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_curve_params(&curve, fixture.range.0, fixture.range.1)
        .unwrap()
        .edge()
        .unwrap();
    BRepBuilder::default()
        .update_edge_tolerance(&edge, if large_tolerance { 0.01 } else { 1e-7 })
        .unwrap();
    edge
}
fn block_range(range: (f64, f64), mode: usize) -> Range {
    let (first, last) = range;
    let span = last - first;
    match mode {
        1 => Range::new(first + span * 0.4, first + span * 0.4 + span * 0.2),
        2 => Range::new(last + 10., last + 11.),
        _ => Range::new(first, last),
    }
}
