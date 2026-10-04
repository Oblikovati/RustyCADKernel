use super::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::Vector3;
#[test]
fn native_existing_pcurve_attachment_matches() {
    for line in include_str!("../../../../tools/oracle/tkbo_attach_existing.expected").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let actual = observe_attachment(
            expected[0] as usize,
            expected[1] as usize,
            expected[2] as usize,
        );
        assert_eq!(actual.len(), expected.len() - 3, "{line}");
        for (column, (actual, expected)) in actual.iter().zip(&expected[3..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-10,
                "column {column}: {line}; actual {actual}"
            );
        }
    }
}
fn attachment_edge(fixture: &ProjectionFixture, mode: usize) -> Edge {
    let curve: Handle<dyn GeomCurve> = fixture.curve.copy().unwrap().cast();
    let mut range = fixture.range;
    if mode == 1 {
        curve.reverse().unwrap();
        range = (
            curve.reversed_parameter(range.1).unwrap(),
            curve.reversed_parameter(range.0).unwrap(),
        );
    }
    if mode == 4 {
        curve.translate(Vector3::new(0., 0., 1e-4)).unwrap();
    }
    if mode == 2 {
        curve.translate(Vector3::new(1., 2., 3.)).unwrap();
    }
    BRepBuilderAPIMakeEdge::from_curve_params(&curve, range.0, range.1)
        .unwrap()
        .edge()
        .unwrap()
}
fn observe_attachment(kind: usize, mode: usize, orientation: usize) -> Vec<f64> {
    let (mut fixture, mut old) = edge_fixture(kind);
    if mode != 3 {
        algo_tools2d::build_pcurve_for_edge_on_face(&old, &fixture.face, None).unwrap();
    }
    let mut edge = attachment_edge(&fixture, mode);
    let orientation = [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ][orientation];
    old.set_orientation(orientation);
    edge.set_orientation(orientation);
    fixture.face.set_orientation(orientation);
    let status =
        algo_tools2d::attach_existing_pcurve(&old, &edge, &fixture.face, &Context::default())
            .unwrap();
    let stored = BRepTool::curve_on_face(&edge, &fixture.face).unwrap();
    let mut output = vec![
        f64::from(status),
        f64::from(stored.pcurve.is_null()),
        BRepTool::edge_tolerance(&edge).unwrap(),
        f64::from(BRepTool::same_parameter(&edge).unwrap()),
        f64::from(BRepTool::same_range(&edge).unwrap()),
    ];
    if !stored.pcurve.is_null() {
        let point = stored
            .pcurve
            .value(0.5 * (stored.first + stored.last))
            .unwrap();
        output.extend([stored.first, stored.last, point.x(), point.y()]);
    }
    let mut vertex_tolerance = 0.0_f64;
    for shape in rustycad_tkbrep::topexp::Explorer::new(
        &edge,
        rustycad_tkg3d::top_abs::ShapeEnum::Vertex,
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap()
    {
        vertex_tolerance = vertex_tolerance.max(
            BRepTool::vertex_tolerance(&rustycad_tkbrep::topods::Vertex::unchecked(shape)).unwrap(),
        );
    }
    output.push(vertex_tolerance);
    output
}

#[test]
fn native_existing_seam_attachment_matches() {
    for line in include_str!("../../../../tools/oracle/tkbo_attach_seam.expected").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut face = fixture(expected[0] as usize).face;
        let (mut old, _) = super::seam::original_edges(&face);
        let mut edge = super::seam::split_edge(&old, &face, 3);
        let orientations = [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ];
        let orientation = orientations[expected[1] as usize];
        old.set_orientation(orientation);
        edge.set_orientation(orientation);
        face.set_orientation(orientation);
        let status =
            algo_tools2d::attach_existing_pcurve(&old, &edge, &face, &Context::default()).unwrap();
        let mut actual = vec![
            f64::from(status),
            f64::from(BRepTool::is_closed_on_face(&edge, &face).unwrap()),
        ];
        face.set_orientation(Orientation::Forward);
        for side in &orientations[..2] {
            edge.set_orientation(*side);
            let stored = BRepTool::curve_on_face(&edge, &face).unwrap();
            let point = stored
                .pcurve
                .value(0.5 * (stored.first + stored.last))
                .unwrap();
            actual.extend([stored.first, stored.last, point.x(), point.y()]);
        }
        assert_eq!(actual.len(), expected.len() - 2);
        for (column, (actual, expected)) in actual.iter().zip(&expected[2..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-10,
                "column {column}: {line}; actual {actual}"
            );
        }
    }
}
