//! Full legacy declarations previously waiting for TKGeomBase/TKGeomAlgo dependencies.
//! Original OCCT inputs and assertions at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::{
    ncollection::{Array1, List},
    precision,
    standard::Handle,
};
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d,
    geom2d::{
        Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
        Geom2dGeometry, Geom2dLine, Geom2dOffsetCurve, Geom2dTrimmedCurve,
    },
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkgeomalgo::{
    gcc_ent::Position,
    geom2d_api::{ExtremaCurveCurve, InterCurveCurve, PointsToBSpline, ProjectPointOnCurve},
    geom2d_gcc::{Circ2d2TanOn, Circ2d2TanRad, QualifiedCurve},
};
use rustycad_tkgeombase::{
    gcpnts::AbscissaPoint,
    geom2d_convert::{self, ApproxCurve},
};
use rustycad_tkmath::{
    convert::ParameterisationType,
    geom_abs::Shape,
    gp::{Axis2d, Axis22d, Circle2, Direction2, Point2, Vector2},
};
use std::f64::consts::PI;
fn legacy_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual}; expected {expected} within {tolerance}"
    );
}
fn legacy_circle(center: Point2, radius: f64) -> Handle<dyn Geom2dCurve> {
    let frame = Axis22d::with_directions(center, Direction2::X, Direction2::Y).unwrap();
    Handle::new(Geom2dCircle::new(frame, radius).unwrap()).cast()
}
fn legacy_ellipse(center: Point2) -> Handle<dyn Geom2dCurve> {
    Handle::new(
        Geom2dEllipse::from_axis(Axis2d::new(center, Direction2::X), 2.0, 1.0, true).unwrap(),
    )
    .cast()
}
fn legacy_extrema(
    first: &Handle<dyn Geom2dCurve>,
    second: &Handle<dyn Geom2dCurve>,
) -> ExtremaCurveCurve {
    ExtremaCurveCurve::new(
        first,
        second,
        first.first_parameter(),
        first.last_parameter(),
        second.first_parameter(),
        second.last_parameter(),
    )
    .unwrap()
}
#[test]
// OCC24889 preserves the original decimal boundary, which differs from FRAC_PI_2.
#[allow(clippy::approx_constant)]
fn Geom2dAPI_InterCurveCurve_Test__OCC24889_IntersectionParameterWithinLimits() {
    // OCC24889: intersection parameters must remain within both trimmed domains.
    let first = legacy_circle(Point2::new(25.0, -25.0), 155.0);
    let second = legacy_circle(Point2::new(25.0, 25.0), 155.0);
    let first: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dTrimmedCurve::new(&first, 1.57079632679490, 2.97959469729228, true, true).unwrap(),
    )
    .cast();
    let second: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dTrimmedCurve::new(&second, 3.30359060633978, 4.71238898038469, true, true).unwrap(),
    )
    .cast();
    let tool = InterCurveCurve::between(&first, &second, precision::confusion()).unwrap();
    assert!(tool.nb_points() > 0);
    let point = tool.intersector().point(1).unwrap();
    let (u, v) = (point.param_on_first(), point.param_on_second());
    assert!(u >= first.first_parameter());
    assert!(u <= first.last_parameter());
    assert!(v >= second.first_parameter());
    assert!(v <= second.last_parameter());
    assert!(
        first
            .value(u)
            .unwrap()
            .square_distance(second.value(v).unwrap())
            < 1e-14
    );
}
#[test]
fn Geom2dAPI_InterCurveCurve_Test__FClassesBug_25635_1_TangentEllipses() {
    let first = legacy_ellipse(Point2::default());
    let second = legacy_ellipse(Point2::new(4.0, 0.0));
    let extrema = legacy_extrema(&first, &second);
    assert!(extrema.nb_extrema() > 0);
    legacy_near(extrema.distance(1).unwrap(), 0.0, 7e-5);
}
#[test]
fn Geom2dAPI_InterCurveCurve_Test__FClassesBug_25635_2_TangentCircles() {
    let first = legacy_circle(Point2::default(), 2.0);
    let second = legacy_circle(Point2::new(4.0, 0.0), 2.0);
    let extrema = legacy_extrema(&first, &second);
    assert!(extrema.nb_extrema() >= 4);
    assert!(extrema.distance(2).unwrap() <= precision::p_confusion());
    let (first, second) = extrema.points(2).unwrap();
    assert!(first.is_equal(second, precision::p_confusion()));
}
fn legacy_qualified(curve: &Handle<dyn Geom2dCurve>) -> QualifiedCurve {
    QualifiedCurve::new(
        &Geom2dAdaptorCurve::with_curve(curve).unwrap(),
        Position::Unqualified,
    )
    .unwrap()
}
fn legacy_bezier(middle_y: f64) -> Handle<dyn Geom2dCurve> {
    let poles = [
        Point2::default(),
        Point2::new(0.0, middle_y),
        Point2::new(6.0, 0.0),
    ];
    Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}
fn tangent_solutions_at(
    first: &Handle<dyn Geom2dCurve>,
    second: &Handle<dyn Geom2dCurve>,
    u: f64,
) -> i32 {
    let (point, tangent) = first.d1(u).unwrap();
    let normal = Direction2::from_vector(Vector2::new(-tangent.y(), tangent.x())).unwrap();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(point, normal)).cast();
    let q1 = legacy_qualified(first);
    let q2 = legacy_qualified(second);
    let g1 = ProjectPointOnCurve::with_curve(point, first)
        .unwrap()
        .lower_distance_parameter()
        .unwrap();
    let g3 = ProjectPointOnCurve::with_curve(point, &line)
        .unwrap()
        .lower_distance_parameter()
        .unwrap();
    Circ2d2TanOn::new(
        &q1,
        &q2,
        &Geom2dAdaptorCurve::with_curve(&line).unwrap(),
        1e-9,
        [g1, g1, g3],
    )
    .unwrap()
    .nb_solutions()
}
#[test]
fn Geom2dGcc_Circ2d2TanOn_Test__OCC27357_NoExceptions() {
    // OCC27357: each of the 100 sub-algorithm calls must complete without exception.
    let first = legacy_bezier(1.0);
    let second = legacy_bezier(1.5);
    let mut dummy = List::default();
    for i in 0..100 {
        dummy.append(tangent_solutions_at(&first, &second, f64::from(i) / 99.0));
    }
}
#[test]
fn Geom2dGcc_Circ2d2TanRad_Test__OCC24303_CircleTangentToTwoEllipses() {
    let first = legacy_ellipse(Point2::default());
    let second = legacy_ellipse(Point2::new(4.0, 0.0));
    let theoretical =
        Circle2::from_x_axis(Axis2d::new(Point2::new(5.0, 0.0), Direction2::X), 3.0, true).unwrap();
    let calculator = Circ2d2TanRad::new(
        &legacy_qualified(&first),
        &legacy_qualified(&second),
        3.0,
        1e-9,
    )
    .unwrap();
    let count = calculator.nb_solutions();
    assert!(count > 0);
    for index in 1..=count {
        legacy_near(calculator.this_solution(index).unwrap().radius(), 3.0, 1e-6);
    }
    if count > 0 {
        assert!(
            theoretical
                .location()
                .distance(calculator.this_solution(1).unwrap().location())
                < 10.0
        );
    }
}
fn legacy_interpolated(points: &[Point2]) -> Handle<Geom2dBSplineCurve> {
    let interpolation =
        PointsToBSpline::with_points(&Array1::from_shared(points, 1), 3, 8, Shape::C2, 1e-6)
            .unwrap();
    assert!(interpolation.is_done());
    interpolation.curve().unwrap()
}
fn legacy_collapsed_offset() -> Geom2dOffsetCurve {
    Geom2dOffsetCurve::new(&legacy_circle(Point2::default(), 3.0), -3.0, false).unwrap()
}
#[test]
fn Geom2d_CurveEvalTest__Circle_EvalD0D1_ValidResults() {
    let curve = legacy_circle(Point2::default(), 4.0);
    let point = curve.eval_d0(PI / 4.0).unwrap();
    legacy_near(
        point.distance(Point2::default()),
        4.0,
        precision::confusion(),
    );
    let result = curve.eval_d1(PI / 4.0).unwrap();
    legacy_near(result.point.distance(point), 0.0, precision::confusion());
    legacy_near(
        Vector2::between(Point2::default(), result.point).dot(result.d1),
        0.0,
        1e-10,
    );
}
#[test]
fn Geom2d_CurveEvalTest__OffsetCurve_EvalD0_ValidAtDegenerateCenter() {
    let point = legacy_collapsed_offset().eval_d0(0.0).unwrap();
    legacy_near(
        point.distance(Point2::default()),
        0.0,
        precision::confusion(),
    );
}
#[test]
fn Geom2d_CurveEvalTest__BSplineCurve_EvalD1_ConsistentWithOldAPI() {
    let curve = legacy_interpolated(&[
        Point2::default(),
        Point2::new(1.0, 2.0),
        Point2::new(3.0, 1.0),
        Point2::new(5.0, 3.0),
        Point2::new(7.0, 0.0),
    ]);
    let mid = (curve.first_parameter() + curve.last_parameter()) / 2.0;
    let result = curve.eval_d1(mid).unwrap();
    let (point, derivative) = curve.d1(mid).unwrap();
    legacy_near(result.point.distance(point), 0.0, precision::confusion());
    legacy_near(
        (result.d1 - derivative).magnitude(),
        0.0,
        precision::confusion(),
    );
}
#[test]
fn Geom2d_CurveEvalTest__Ellipse_EvalD2D3_ValidResults() {
    let curve = Geom2dEllipse::new(
        Axis22d::new(Point2::default(), Direction2::X, true).unwrap(),
        5.0,
        3.0,
    )
    .unwrap();
    assert!(curve.eval_d2(PI / 4.0).unwrap().d2.magnitude() > 0.0);
    assert!(curve.eval_d3(PI / 4.0).unwrap().d3.magnitude() > 0.0);
}
#[test]
fn Geom2d_CurveEvalTest__Line_EvalDN_ValidResults() {
    let curve = Geom2dLine::from_point(Point2::new(1.0, 2.0), Direction2::X);
    let first = curve.eval_dn(5.0, 1).unwrap();
    legacy_near(first.x(), 1.0, precision::confusion());
    legacy_near(first.y(), 0.0, precision::confusion());
    legacy_near(
        curve.eval_dn(5.0, 2).unwrap().magnitude(),
        0.0,
        precision::confusion(),
    );
}
#[test]
fn Geom2d_CurveEvalTest__BSplineCurve_EvalDN_InvalidN_Throws() {
    let curve = legacy_interpolated(&[
        Point2::default(),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    ]);
    let mid = (curve.first_parameter() + curve.last_parameter()) / 2.0;
    assert_eq!(
        curve.eval_dn(mid, 0).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
    assert_eq!(
        curve.eval_dn(mid, -1).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
}
#[test]
fn Geom2d_CurveEvalTest__OffsetCurve_EvalD1_DegenerateAtCollapsed() {
    // The basis tangent is valid, so this succeeds even though the offset collapses.
    let result = legacy_collapsed_offset().eval_d1(0.0).unwrap();
    legacy_near(
        result.point.distance(Point2::default()),
        0.0,
        precision::confusion(),
    );
}
fn legacy_original_offset() -> Geom2dOffsetCurve {
    let circle: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dCircle::from_axis(Axis2d::new(Point2::default(), Direction2::X), 5.0, true).unwrap(),
    )
    .cast();
    Geom2dOffsetCurve::new(&circle, 2.0, false).unwrap()
}
#[test]
fn Geom2d_OffsetCurve_Test__CopyConstructorBasicProperties() {
    let original = legacy_original_offset();
    let copy = Geom2dOffsetCurve::copy_of(&original).unwrap();
    assert_eq!(original.offset(), copy.offset());
    assert_eq!(original.is_periodic(), copy.is_periodic());
    assert_eq!(original.is_closed(), copy.is_closed());
}
#[test]
fn Geom2d_OffsetCurve_Test__CopyConstructorBasisCurve() {
    let original = legacy_original_offset();
    let copy = Geom2dOffsetCurve::copy_of(&original).unwrap();
    let first = original.basis_curve();
    let second = copy.basis_curve();
    assert!(first != second);
    assert_eq!(first.first_parameter(), second.first_parameter());
    assert_eq!(first.last_parameter(), second.last_parameter());
}
#[test]
fn Geom2d_OffsetCurve_Test__CopyMethodUsesOptimizedConstructor() {
    let original = legacy_original_offset();
    let copy = original.copy().unwrap().cast::<Geom2dOffsetCurve>();
    assert!(!copy.is_null());
    assert_eq!(original.offset(), copy.offset());
    let mut u = original.first_parameter();
    let last = original.last_parameter();
    let step = (last - u) / 4.0;
    while u <= last {
        assert!(
            original
                .value(u)
                .unwrap()
                .is_equal(copy.value(u).unwrap(), 1e-10)
        );
        u += step;
    }
}
#[test]
fn Geom2d_OffsetCurve_Test__CopyIndependence() {
    let original = legacy_original_offset();
    let copy = Geom2dOffsetCurve::copy_of(&original).unwrap();
    let offset = copy.offset();
    original.set_offset_value(10.0);
    assert_eq!(copy.offset(), offset);
    assert_ne!(copy.offset(), original.offset());
}
#[test]
// Keep the reference length from fclasses/bug31381 verbatim.
#[allow(clippy::excessive_precision)]
fn Geom2d_OffsetCurve_AdditionalTest__FClassesBug_31381_RationalBSplineApproximation() {
    let circle: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dCircle::from_axis(Axis2d::new(Point2::default(), Direction2::X), 1.0, true).unwrap(),
    )
    .cast();
    let converted =
        geom2d_convert::curve_to_bspline_curve(&circle, ParameterisationType::TgtThetaOver2)
            .unwrap();
    assert!(!converted.is_null());
    let offset: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&converted.cast(), -0.5, false).unwrap()).cast();
    let approximation = ApproxCurve::from_curve(&offset, 1e-4, Shape::C2, 25, 5).unwrap();
    assert!(approximation.has_result());
    let curve = approximation.curve();
    assert!(!curve.is_null());
    let adaptor = Geom2dAdaptorCurve::with_curve(&curve.cast()).unwrap();
    let length = AbscissaPoint::length(&adaptor as &dyn Adaptor2dCurve2d).unwrap();
    legacy_near(length, 3.1415850499856011, 1e-6);
}
