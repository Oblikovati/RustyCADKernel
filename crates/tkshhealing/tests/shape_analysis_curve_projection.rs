//! Projection API contracts supplement unchanged native differential comparisons.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::{
    CurveProjection, ShapeAnalysisCurve, adjust_by_period, adjust_to_period,
};
fn line() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast()
}
fn linear_bezier() -> Handle<dyn GeomCurve> {
    let poles = [Point3::default(), Point3::new(1.0, 0.0, 0.0)];
    Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}
#[test]
fn line_projection_overloads_return_same_distance_point_and_parameter() {
    let curve = line();
    let p = Point3::new(3.0, 4.0, 0.0);
    let tool = ShapeAnalysisCurve;
    let expected = (4.0, Point3::new(3.0, 0.0, 0.0), 3.0);
    assert_eq!(tool.project_full(&curve, p, 1e-7, true).unwrap(), expected);
    assert_eq!(
        tool.project(&curve, p, 1e-7, (0.0, 10.0), true).unwrap(),
        expected
    );
    assert_eq!(
        tool.next_project(2.0, &curve, p, 1e-7, (0.0, 10.0), false)
            .unwrap(),
        expected
    );
}
#[test]
fn adaptor_projection_overloads_share_the_native_search() {
    let curve = line();
    let adaptor = GeomAdaptorCurve::with_range(&curve, 0.0, 10.0).unwrap();
    let p = Point3::new(3.0, 4.0, 0.0);
    let tool = ShapeAnalysisCurve;
    let expected = (4.0, Point3::new(3.0, 0.0, 0.0), 3.0);
    assert_eq!(
        tool.project_adaptor(&adaptor, p, 1e-7, true).unwrap(),
        expected
    );
    assert_eq!(
        tool.project_act_adaptor(&adaptor, p, 1e-7).unwrap(),
        expected
    );
    assert_eq!(
        tool.next_project_adaptor(2.0, &adaptor, p, 1e-7).unwrap(),
        expected
    );
}
#[test]
fn bounded_endpoint_adjustment_uses_confusion_when_disabled() {
    let curve = linear_bezier();
    let point = Point3::new(1e-5, 0.0, 0.0);
    let tool = ShapeAnalysisCurve;
    let snapped = tool.project(&curve, point, 0.1, (0.0, 1.0), true).unwrap();
    assert_eq!(snapped.2, 0.0);
    let projected = tool.project(&curve, point, 0.1, (0.0, 1.0), false).unwrap();
    assert!(projected.2 > 0.0);
    let local = tool
        .next_project(0.5, &curve, point, 0.1, (0.0, 1.0), true)
        .unwrap();
    assert_eq!(local.2, 0.0);
}
#[test]
fn output_projection_records_retain_values_before_invalid_curve() {
    let mut output = CurveProjection {
        point: Point3::new(3.0, 4.0, 5.0),
        parameter: 7.0,
    };
    let original = output;
    let error = ShapeAnalysisCurve
        .project_full_into(
            &Handle::default(),
            Point3::default(),
            1e-7,
            true,
            &mut output,
        )
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert_eq!(output, original);
}
#[test]
fn period_shifts_keep_floor_for_large_multiples() {
    assert_eq!(adjust_by_period(1e20, 0.0, 6.0), -1e20);
    assert_eq!(adjust_by_period(3.0, 0.0, 6.0), 0.0);
    assert_eq!(adjust_by_period(7.0, 0.0, 1e-110), 7.0);
    assert_eq!(adjust_by_period(-7.0, 0.0, -6.0), 6.0);
    assert_eq!(adjust_to_period(7.0, 0.0, 6.0), -6.0);
}
