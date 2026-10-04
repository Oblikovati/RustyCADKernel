use super::{ProjectionFixture, fixture};
use rustycad_tkbo::{
    bop_tools::{algo_tools2d, algo_tools3d},
    int_tools::Context,
};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Edge};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis2d, Direction2, Direction3, Point2};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
#[test]
fn native_lookup_projection_and_normals_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_edge_pcurve.expected").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = observe(
            expected[0] as usize,
            expected[1] as usize,
            expected[2] != 0.,
        );
        assert_eq!(actual.len() + 3, expected.len());
        for (index, (actual, expected)) in actual.iter().zip(&expected[3..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "column {index}: {line}; actual {actual}"
            );
        }
    }
}
fn observe(kind: usize, orientation: usize, cached: bool) -> Vec<f64> {
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
    let mut curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(9., 8.),
        Direction2::new(1., 0.).unwrap(),
    )))
    .cast();
    let mut range = (9., 8.);
    let mut tolerance = 9.;
    let simple = algo_tools2d::has_curve_on_surface_simple(&edge, &fixture.face).unwrap();
    let found = algo_tools2d::has_curve_on_surface(
        &edge,
        &fixture.face,
        &mut curve,
        &mut range,
        &mut tolerance,
    )
    .unwrap();
    let mut output = vec![
        f64::from(simple),
        f64::from(found),
        f64::from(curve.is_null()),
        range.0,
        range.1,
        tolerance,
    ];
    algo_tools2d::curve_on_surface(
        &edge,
        &fixture.face,
        &mut curve,
        &mut range,
        &mut tolerance,
        context,
    )
    .unwrap();
    append_curve(&mut output, &curve, range, tolerance);
    tolerance = 9.;
    algo_tools2d::make_2d(
        &edge,
        &fixture.face,
        &mut curve,
        &mut range,
        &mut tolerance,
        context,
    )
    .unwrap();
    append_curve(&mut output, &curve, range, tolerance);
    append_normals(&mut output, &fixture, &edge, context);
    output
}
fn append_curve(
    output: &mut Vec<f64>,
    curve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    tolerance: f64,
) {
    let point = curve.value(0.5 * (range.0 + range.1)).unwrap();
    output.extend([range.0, range.1, tolerance, point.x(), point.y()]);
}
fn append_normals(
    output: &mut Vec<f64>,
    fixture: &ProjectionFixture,
    edge: &Edge,
    context: Option<&Context>,
) {
    let mut normal = Direction3::default();
    algo_tools3d::get_normal_to_face_on_edge(edge, &fixture.face, &mut normal, context).unwrap();
    output.extend([normal.x(), normal.y(), normal.z()]);
    algo_tools3d::get_normal_to_face_on_edge_at(edge, &fixture.face, 1., &mut normal, context)
        .unwrap();
    output.extend([normal.x(), normal.y(), normal.z()]);
    output.push(algo_tools2d::intermediate_point_on_edge(edge).unwrap());
}
fn edge_fixture(kind: usize) -> (ProjectionFixture, Edge) {
    let fixture = fixture(if kind < 5 {
        kind
    } else if kind == 5 {
        0
    } else {
        1
    });
    let edge =
        BRepBuilderAPIMakeEdge::from_curve_params(&fixture.curve, fixture.range.0, fixture.range.1)
            .unwrap()
            .edge()
            .unwrap();
    if kind == 5 {
        BRepBuilder::default().range(&edge, 0., 0., false).unwrap();
    }
    if kind == 6 {
        store_pcurve(&fixture, &edge);
    }
    (fixture, edge)
}
fn store_pcurve(fixture: &ProjectionFixture, edge: &Edge) {
    let mut stored = Handle::default();
    let mut tolerance = 1e-7;
    algo_tools2d::make_pcurve_on_face(
        &fixture.face,
        &fixture.curve,
        fixture.range,
        &mut stored,
        &mut tolerance,
        None,
    )
    .unwrap();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(edge, &stored, &fixture.face, 1e-7)
        .unwrap();
    builder
        .range_on_face(edge, &fixture.face, fixture.range.0, fixture.range.1)
        .unwrap();
}

mod near_edge;

mod point_in_face;

mod orient_edge;

mod approx_normal;

mod surface_normal;

mod seam;
