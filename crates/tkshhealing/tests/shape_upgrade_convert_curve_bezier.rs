//! Native-backed Compute/Build checks for ShapeUpgrade curve-to-Bezier converters.
use rustycad_tkernel::{
    ncollection::{Array1, HSequence},
    standard::Handle,
};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction2, Direction3, Point2, Point3};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{
        ShapeUpgradeConvertCurve2dToBezier, ShapeUpgradeConvertCurve3dToBezier,
        ShapeUpgradeSplitCurve3d, SplitCurve,
    },
};

struct NativeCase {
    status: [bool; 3],
    values: Vec<f64>,
    params: Vec<f64>,
    curves: Vec<(String, Vec<f64>)>,
}

fn expected_case(name: &str) -> NativeCase {
    let block = include_str!("../../../tools/oracle/tkshhealing_convert_curve_bezier.expected")
        .split("\n\n")
        .find(|block| block.starts_with(name))
        .unwrap();
    let rows: Vec<&str> = block.lines().collect();
    let status = parse_values(rows[0])
        .iter()
        .map(|v| *v != 0.)
        .collect::<Vec<_>>();
    let curves = rows[3..].iter().map(|row| parse_curve(row)).collect();
    NativeCase {
        status: [status[0], status[1], status[2]],
        values: parse_values(rows[1]),
        params: parse_values(rows[2]),
        curves,
    }
}

fn parse_values(row: &str) -> Vec<f64> {
    row.split_whitespace()
        .filter_map(|item| item.parse().ok())
        .collect()
}

fn parse_curve(row: &str) -> (String, Vec<f64>) {
    let mut fields = row.split_whitespace();
    let name = fields.next().unwrap().to_owned();
    let values = fields.filter_map(|field| field.parse().ok()).collect();
    (name, values)
}

fn assert_sequence(actual: Vec<f64>, expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() <= 1e-14, "{actual} != {expected}");
    }
}

fn assert_native_case(
    name: &str,
    status: [bool; 3],
    values: Vec<f64>,
    params: Vec<f64>,
    curves: Vec<(String, Vec<f64>)>,
) {
    let expected = expected_case(name);
    assert_eq!(status, expected.status, "{name} status");
    assert_sequence(values, &expected.values);
    assert_sequence(params, &expected.params);
    assert_eq!(curves.len(), expected.curves.len(), "{name} result count");
    for ((actual_type, actual_values), (expected_type, expected_values)) in
        curves.iter().zip(expected.curves)
    {
        assert_eq!(actual_type, &expected_type, "{name} result type");
        assert_sequence(actual_values.clone(), &expected_values);
    }
}

fn status3(tool: &ShapeUpgradeConvertCurve3dToBezier) -> [bool; 3] {
    [
        tool.status_flag(ShapeExtendStatus::Done1),
        tool.status_flag(ShapeExtendStatus::Done2),
        tool.status_flag(ShapeExtendStatus::Done3),
    ]
}

fn status2(tool: &ShapeUpgradeConvertCurve2dToBezier) -> [bool; 3] {
    [
        tool.status_flag(ShapeExtendStatus::Done1),
        tool.status_flag(ShapeExtendStatus::Done2),
        tool.status_flag(ShapeExtendStatus::Done3),
    ]
}

fn output3(tool: &ShapeUpgradeConvertCurve3dToBezier) -> Vec<(String, Vec<f64>)> {
    tool.curves().array1().iter().map(observe_curve3).collect()
}

fn observe_curve3(handle: &Handle<dyn GeomCurve>) -> (String, Vec<f64>) {
    let curve = handle.get().unwrap();
    let kind = curve.dynamic_type().name().to_owned();
    if kind == "Geom_Line" {
        return (kind, Vec::new());
    }
    let first = curve.value(curve.first_parameter()).unwrap();
    let last = curve.value(curve.last_parameter()).unwrap();
    (
        kind,
        vec![
            first.x(),
            first.y(),
            first.z(),
            last.x(),
            last.y(),
            last.z(),
        ],
    )
}

fn output2(tool: &ShapeUpgradeConvertCurve2dToBezier) -> Vec<(String, Vec<f64>)> {
    tool.curves()
        .array1()
        .iter()
        .map(|handle| {
            let curve = handle.get().unwrap();
            let first = curve.value(curve.first_parameter()).unwrap();
            let last = curve.value(curve.last_parameter()).unwrap();
            (
                curve.dynamic_type().name().to_owned(),
                vec![first.x(), first.y(), last.x(), last.y()],
            )
        })
        .collect()
}

fn points3() -> [Point3; 3] {
    [
        Point3::new(0., 0., 0.),
        Point3::new(1., 2., 0.),
        Point3::new(2., 0., 0.),
    ]
}

fn points2() -> [Point2; 3] {
    [
        Point2::new(0., 0.),
        Point2::new(1., 2.),
        Point2::new(2., 0.),
    ]
}

fn linear_points2() -> [Point2; 3] {
    [
        Point2::new(0., 0.),
        Point2::new(2., 0.),
        Point2::new(4., 0.),
    ]
}

fn split3(
    name: &str,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    line_mode: bool,
    circle_mode: bool,
) {
    split3_with_conic(name, curve, range, line_mode, circle_mode, true);
}

fn split3_with_conic(
    name: &str,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    line_mode: bool,
    circle_mode: bool,
    conic_mode: bool,
) {
    let tool = ShapeUpgradeConvertCurve3dToBezier::new();
    tool.set_line_mode(line_mode);
    tool.set_circle_mode(circle_mode);
    tool.set_conic_mode(conic_mode);
    tool.init_curve_range(curve, range.0, range.1).unwrap();
    let values = Handle::new(HSequence::default());
    values.append((range.0 + range.1) / 2.);
    tool.set_split_values(&values).unwrap();
    tool.compute().unwrap();
    tool.build(true).unwrap();
    check3(name, &tool);
}

#[test]
fn native_3d_ellipse_conic_mode_matches() {
    let ellipse: Handle<dyn GeomCurve> =
        Handle::new(GeomEllipse::new(Axis2::default(), 3., 2.).unwrap()).cast();
    for (name, enabled) in [
        ("ellipse-conic-mode-off", false),
        ("ellipse-conic-mode-on", true),
    ] {
        split3_with_conic(name, &ellipse, (0., 1.), true, true, enabled);
    }
}

fn check3(name: &str, tool: &ShapeUpgradeConvertCurve3dToBezier) {
    let values = tool
        .split_values()
        .sequence(|s| s.iter().copied().collect());
    let params = tool
        .split_params()
        .sequence(|s| s.iter().copied().collect());
    assert_native_case(name, status3(tool), values, params, output3(tool));
}

#[test]
fn native_3d_line_modes_bezier_trim_and_circle_arcs_match() {
    let axis = Axis1::new(Point3::default(), Direction3::X);
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
    split3("line", &line, (0., 4.), true, true);
    split3("line-mode-off", &line, (0., 4.), false, true);
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&line, 0., 4., true, true).unwrap()).cast();
    split3("trimmed-line-mode-off", &trimmed, (0., 4.), false, true);
    let points = points3();
    let poles = Array1::from_shared(&points, 1);
    let bezier: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    split3("bezier-full", &bezier, (0., 1.), true, true);
    split3("bezier-trim", &bezier, (0.2, 0.8), true, true);
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 2.).unwrap()).cast();
    split3("circle-mode-off", &circle, (0., 1.), true, false);
    split3("circle-arcs", &circle, (0., 1.), true, true);
}

fn split2(name: &str, curve: &Handle<dyn Geom2dCurve>, range: (f64, f64), split: f64) {
    let converter = ShapeUpgradeConvertCurve2dToBezier::new();
    converter.init_curve_range(curve, range.0, range.1).unwrap();
    let values = Handle::new(HSequence::default());
    values.append(split);
    converter.set_split_values(&values).unwrap();
    converter.compute().unwrap();
    converter.build(true).unwrap();
    check2(name, &converter);
}

fn check2(name: &str, converter: &ShapeUpgradeConvertCurve2dToBezier) {
    let values = converter
        .split_values()
        .sequence(|s| s.iter().copied().collect());
    let params = converter
        .split_params()
        .sequence(|s| s.iter().copied().collect());
    assert_native_case(name, status2(converter), values, params, output2(converter));
}

#[test]
fn native_2d_line_output_matches() {
    let axis =
        rustycad_tkmath::gp::Axis2d::new(Point2::default(), Direction2::new(1., 0.).unwrap());
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(axis)).cast();
    split2("line2d", &line, (0., 4.), 2.);
}

#[test]
fn native_2d_trimmed_bezier_build_matches() {
    let points = points2();
    let poles = Array1::from_shared(&points, 1);
    let bezier: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&poles).unwrap()).cast();
    split2("bezier2d-trim", &bezier, (0.2, 0.8), 0.5);
}

#[test]
fn native_2d_linear_approximation_and_circle_arcs_match() {
    let points = linear_points2();
    let poles = Array1::from_shared(&points, 1);
    let line_bezier: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&poles).unwrap()).cast();
    split2("linear-bezier2d", &line_bezier, (0., 1.), 0.5);

    let axis = rustycad_tkmath::gp::Axis2d::default();
    let circle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(axis, 2., true).unwrap()).cast();
    split2("circle2d-arcs", &circle, (0., 1.), 0.5);
}

#[test]
fn native_rtti_parent_and_virtual_compute_build_share_result_state() {
    let tool = Handle::new(ShapeUpgradeConvertCurve3dToBezier::new());
    let base = tool.cast::<ShapeUpgradeSplitCurve3d>();
    let dispatch = tool.cast::<dyn SplitCurve>();
    let axis = Axis1::new(Point3::default(), Direction3::X);
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
    base.init_curve_range(&line, 0., 4.).unwrap();
    dispatch.compute().unwrap();
    dispatch.build(true).unwrap();
    assert_eq!(
        base.curves().object_address(),
        tool.curves().object_address()
    );
    assert!(tool.status_flag(ShapeExtendStatus::Done1));
}
