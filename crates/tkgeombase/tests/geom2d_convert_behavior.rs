//! Regression tests of the native behaviour the Geom2dConvert port preserves
//! (docs/tkgeombase-progress.md); the native outcomes are those of
//! tools/oracle/tkgeombase_probe_geom2dconvert.cxx.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dOffsetCurve};
use rustycad_tkgeombase::geom2d_convert::{
    self as g2c, ApproxCurve, BSplineCurveKnotSplitting, HArrayOfBSplineCurves, PPoint,
};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Xy};

fn one_based<T: Default>(n: i32) -> Array1<'static, T> {
    Array1::with_bounds(1, n).unwrap()
}

// A degree-`deg` Bezier-like B-spline on [0, 1] with the given poles and weights.
fn spline(points: &[(f64, f64)], weights: &[f64]) -> Handle<Geom2dBSplineCurve> {
    let n = points.len() as i32;
    let mut poles = one_based::<Point2>(n);
    let mut w = one_based::<f64>(n);
    for (i, (p, wi)) in (1..).zip(points.iter().zip(weights)) {
        poles[i] = Point2::new(p.0, p.1);
        w[i] = *wi;
    }
    let mut knots = one_based::<f64>(2);
    (knots[1], knots[2]) = (0.0, 1.0);
    let mut mults = one_based::<i32>(2);
    (mults[1], mults[2]) = (n, n);
    Handle::new(Geom2dBSplineCurve::with_weights(&poles, &w, &knots, &mults, n - 1, false).unwrap())
}

#[test]
fn approx_curve_dump_prints_the_native_text() {
    let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_circ2d(
        rustycad_tkmath::gp::Circle2::from_x_axis(rustycad_tkmath::gp::ox2d(), 2.0, true).unwrap(),
    ))
    .cast();
    let offset: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&circle, 0.5, false).unwrap()).cast();
    let approx = ApproxCurve::from_curve(&offset, 1e-4, Shape::C2, 16, 14).unwrap();
    assert!(approx.has_result());
    let mut text = String::new();
    approx.dump(&mut text).unwrap();
    let head = "******* Dump of ApproxCurve *******\n******* Error   ";
    assert!(text.starts_with(head) && text.ends_with('\n'), "{text}");
    // The error is printed as `%g`: six significant digits.
    let printed: f64 = text[head.len()..text.len() - 1].parse().unwrap();
    let error: f64 = approx.max_error();
    assert!((printed - error).abs() <= 1e-5 * error, "{text}");
}

#[test]
fn ppoint_comparisons_are_both_false_for_nan_parameters() {
    let nan = PPoint::new(f64::NAN, Xy::new(0.0, 0.0), Xy::new(1.0, 0.0));
    let other = PPoint::new(0.0, Xy::new(0.0, 0.0), Xy::new(1.0, 0.0));
    assert!(!(nan == other));
    assert!(!(nan != other));
    assert_eq!(PPoint::default().parameter(), f64::MAX);
    let near = PPoint::new(1e-10, Xy::new(5.0, 5.0), Xy::new(0.0, 0.0));
    assert!(near == other, "parameters within PConfusion are equal");
}

#[test]
fn split_by_parameters_ignores_the_parametric_tolerance() {
    let curve = spline(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)], &[1.0, 1.0, 1.0]);
    let tight = g2c::split_bspline_curve(&curve, 0.8, 0.2, 1e-12, true).unwrap();
    let loose = g2c::split_bspline_curve(&curve, 0.8, 0.2, 10.0, true).unwrap();
    assert_eq!(tight.first_parameter(), loose.first_parameter());
    // A non-periodic curve given decreasing bounds is reversed.
    assert!(tight.pole(1).unwrap().distance(curve.value(0.8).unwrap()) < 1e-12);
}

#[test]
fn split_by_knots_reports_equal_and_out_of_range_indices() {
    let curve = spline(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)], &[1.0, 1.0, 1.0]);
    let equal = g2c::split_bspline_curve_knots(&curve, 1, 1, true)
        .err()
        .unwrap();
    assert_eq!(equal.exception_type(), "Standard_DomainError");
    let outside = g2c::split_bspline_curve_knots(&curve, 1, 3, true)
        .err()
        .unwrap();
    assert_eq!(outside.exception_type(), "Standard_OutOfRange");
}

#[test]
fn knot_splitting_rejects_a_negative_range() {
    let curve = spline(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)], &[1.0, 1.0, 1.0]);
    let error = BSplineCurveKnotSplitting::new(&curve, -1).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_RangeError");
    let split = BSplineCurveKnotSplitting::new(&curve, 0).unwrap();
    assert_eq!(split.nb_splits(), 2);
    assert_eq!(
        split.split_value(3).err().unwrap().exception_type(),
        "Standard_RangeError"
    );
}

#[test]
fn concatenation_normalizes_the_weights_of_the_callers_curves() {
    let first = spline(
        &[(0.0, 0.0), (1.0, 1.0), (2.0, 1.0), (3.0, 0.0)],
        &[2.0, 2.0, 1.0, 2.0],
    );
    let second = spline(&[(3.0, 0.0), (4.0, -1.0), (5.0, 0.0)], &[3.0, 3.0, 3.0]);
    let mut curves: Array1<'static, Handle<Geom2dBSplineCurve>> =
        Array1::with_bounds(0, 1).unwrap();
    (curves[0], curves[1]) = (first.clone(), second.clone());
    let mut tolerances: Array1<'static, f64> = Array1::with_bounds(0, 0).unwrap();
    tolerances[0] = 1e-7;
    let mut result: HArrayOfBSplineCurves = Handle::default();
    let mut closed = false;
    g2c::concat_g1(&mut curves, &tolerances, &mut result, &mut closed, 0.0).unwrap();
    // The first curve does not have four equal end weights; the second one does, and
    // its weights (all equal, so it is no longer rational) are divided in place.
    assert_eq!(first.weight(1).unwrap(), 2.0);
    assert!(!second.is_rational());
    assert_eq!(second.weight(2).unwrap(), 1.0);
}

#[test]
fn separated_curves_raise_the_native_continuity_failure() {
    let first = spline(&[(0.0, 0.0), (1.0, 1.0)], &[1.0, 1.0]);
    let second = spline(&[(1.5, 1.0), (2.0, 0.0)], &[1.0, 1.0]);
    let mut curves: Array1<'static, Handle<Geom2dBSplineCurve>> =
        Array1::with_bounds(0, 1).unwrap();
    (curves[0], curves[1]) = (first, second);
    let mut tolerances: Array1<'static, f64> = Array1::with_bounds(0, 0).unwrap();
    tolerances[0] = 1e-7;
    let mut result: HArrayOfBSplineCurves = Handle::default();
    let mut indices = Handle::default();
    let mut closed = false;
    let error = g2c::concat_c1(
        &mut curves,
        &tolerances,
        &mut indices,
        &mut result,
        &mut closed,
        0.0,
    )
    .err()
    .unwrap();
    assert_eq!(error.exception_type(), "Standard_Failure");
    assert_eq!(error.message(), "Courbes non jointives");
}

#[test]
fn untrimmed_lines_are_not_convertible() {
    let line: Handle<dyn Geom2dCurve> = Handle::new(rustycad_tkg2d::geom2d::Geom2dLine::new(
        rustycad_tkmath::gp::ox2d(),
    ))
    .cast();
    let error = g2c::curve_to_bspline_curve(&line, ParameterisationType::TgtThetaOver2)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_DomainError");
}
