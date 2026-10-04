use super::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkg3d::geom::GeomTrimmedCurve;
#[test]
fn native_edge_projection_utilities_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_edge_utilities.expected").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let actual = observe_utilities(
            expected[0] as usize,
            expected[1] as usize,
            expected[2] != 0.,
        );
        assert_eq!(actual.len(), expected.len() - 3);
        for (index, (actual, expected)) in actual.iter().zip(&expected[3..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "column {index}: {line}; actual {actual}"
            );
        }
    }
}
fn observe_utilities(kind: usize, orientation: usize, cached: bool) -> Vec<f64> {
    let (mut fixture, edge) = edge_fixture(kind);
    fixture.face.set_orientation(
        [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ][orientation],
    );
    let context = Context::default();
    let context = if cached { Some(&context) } else { None };
    let parameter = algo_tools2d::intermediate_point(fixture.range.0, fixture.range.1);
    let mut output = vec![parameter];
    let mut curve = Handle::default();
    let mut tolerance = 9.;
    algo_tools2d::curve_on_surface_simple(
        &edge,
        &fixture.face,
        &mut curve,
        &mut tolerance,
        context,
    )
    .unwrap();
    output.push(tolerance);
    append_point(&mut output, curve.value(parameter).unwrap());
    let mut point = Point2::new(9., 8.);
    algo_tools2d::point_on_surface(&edge, &fixture.face, parameter, &mut point, context).unwrap();
    append_point(&mut output, point);
    let bounded = Handle::new(
        GeomTrimmedCurve::new(&fixture.curve, fixture.range.0, fixture.range.1, true, true)
            .unwrap(),
    )
    .cast();
    let adjusted =
        algo_tools2d::adjust_pcurve_for_curve(&fixture.face, &bounded, &curve, context).unwrap();
    append_point(&mut output, adjusted.value(parameter).unwrap());
    append_stored(&mut output, &edge, &fixture, parameter, context);
    output
}
fn append_point(output: &mut Vec<f64>, point: Point2) {
    output.extend([point.x(), point.y()]);
}
fn append_stored(
    output: &mut Vec<f64>,
    edge: &Edge,
    fixture: &ProjectionFixture,
    parameter: f64,
    context: Option<&Context>,
) {
    for _ in 0..2 {
        algo_tools2d::build_pcurve_for_edge_on_face(edge, &fixture.face, context).unwrap();
    }
    let stored = BRepTool::curve_on_face(edge, &fixture.face).unwrap();
    output.extend([
        stored.first,
        stored.last,
        BRepTool::edge_tolerance(edge).unwrap(),
    ]);
    append_point(output, stored.pcurve.value(parameter).unwrap());
    let (mut u_iso, mut v_iso) = (true, true);
    algo_tools2d::is_edge_isoline(edge, &fixture.face, &mut u_iso, &mut v_iso).unwrap();
    output.extend([f64::from(u_iso), f64::from(v_iso)]);
}

#[test]
fn native_isoline_flags_ignore_direction_sign() {
    let directions = [
        (1., 0.),
        (-1., 0.),
        (0., 1.),
        (0., -1.),
        (1., 1.),
        (-1., 1.),
        (1., -1.),
        (-1., -1.),
    ];
    for line in include_str!("../../../../tools/oracle/tkbo_isoline.expected").lines() {
        let expected: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (fixture, edge) = edge_fixture(1);
        let (x, y) = directions[expected[0]];
        let curve = Handle::new(Geom2dLine::new(Axis2d::new(
            Point2::new(0., 0.),
            Direction2::new(x, y).unwrap(),
        )))
        .cast();
        BRepBuilder::default()
            .update_edge_pcurve_on_face(&edge, &curve, &fixture.face, 1e-7)
            .unwrap();
        let (mut u_iso, mut v_iso) = (true, true);
        algo_tools2d::is_edge_isoline(&edge, &fixture.face, &mut u_iso, &mut v_iso).unwrap();
        assert_eq!(
            (usize::from(u_iso), usize::from(v_iso)),
            (expected[1], expected[2]),
            "{line}"
        );
    }
}
