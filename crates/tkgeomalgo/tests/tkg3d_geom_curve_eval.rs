//! Direct translation of ModelingData/TKG3d/GTests/Geom_CurveEval_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. The suite belongs to the TKG3d test
//! directory but builds its B-spline curves with GeomAPI_PointsToBSpline
//! (TKGeomAlgo), so it runs in this crate.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomLine, GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkgeomalgo::geom_api::PointsToBSpline;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3, Vector3};
use std::f64::consts::PI;

fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

// `gp_Circ aCirc(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), R); new Geom_Circle(aCirc)`.
fn origin_circle(radius: f64) -> Handle<dyn GeomCurve> {
    let frame = Axis2::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    let aCirc = Circle3::new(frame, radius).unwrap();
    Handle::new(GeomCircle::from_circ(aCirc)).cast()
}

// `GeomAPI_PointsToBSpline(Points)` with the native defaults (3, 8, C2, 1e-3).
fn points_to_bspline(aPnts: &Array1<'_, Point3>) -> PointsToBSpline {
    PointsToBSpline::with_points(aPnts, 3, 8, Shape::C2, 1.0e-3).unwrap()
}

//=================================================================================================
// Circle tests
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__Circle_EvalD0_ReturnsValidPoint() {
    let aCurve = origin_circle(5.0);

    let aU = PI / 4.0;
    let aResult = aCurve.eval_d0(aU).unwrap();

    // Point should lie on circle of radius 5
    let aDist = aResult.distance(&Point3::new(0.0, 0.0, 0.0));
    assert_near(aDist, 5.0, precision::confusion());
}

#[test]
fn Geom_CurveEvalTest__Circle_EvalD1D2D3_ReturnsValidDerivatives() {
    let aCurve = origin_circle(5.0);
    let origin = Point3::new(0.0, 0.0, 0.0);

    let aU = PI / 3.0;

    // EvalD1
    let aD1 = aCurve.eval_d1(aU).unwrap();
    assert_near(aD1.point.distance(&origin), 5.0, precision::confusion());
    // D1 should be perpendicular to radius vector for a circle
    let aRadial = Vector3::between(origin, aD1.point);
    assert_near(aRadial.dot(aD1.d1), 0.0, 1.0e-10);

    // EvalD2
    let aD2 = aCurve.eval_d2(aU).unwrap();
    assert_near(aD2.point.distance(&aD1.point), 0.0, precision::confusion());
    // D2 of circle points toward center (centripetal acceleration)
    assert_near(aD2.d2.magnitude(), 5.0, 1.0e-10);

    // EvalD3
    let aD3 = aCurve.eval_d3(aU).unwrap();
    assert_near(aD3.point.distance(&aD1.point), 0.0, precision::confusion());
    assert!(aD3.d3.magnitude() > 0.0);
}

//=================================================================================================
// Line tests
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__Line_EvalD2_ZeroSecondDerivative() {
    let aCurve = GeomLine::from_point(Point3::new(1.0, 2.0, 3.0), dir(1.0, 0.0, 0.0));

    let aD2 = aCurve.eval_d2(5.0).unwrap();
    assert_near(aD2.d2.magnitude(), 0.0, precision::confusion());

    // D1 should be the direction vector
    assert_near(aD2.d1.x(), 1.0, precision::confusion());
    assert_near(aD2.d1.y(), 0.0, precision::confusion());
    assert_near(aD2.d1.z(), 0.0, precision::confusion());
}

//=================================================================================================
// BSplineCurve consistency tests
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__BSplineCurve_EvalD1_ConsistentWithOldAPI() {
    // Create a simple BSpline through points
    let mut aPnts: Array1<'static, Point3> = Array1::with_bounds(1, 5).unwrap();
    aPnts[1] = Point3::new(0.0, 0.0, 0.0);
    aPnts[2] = Point3::new(1.0, 2.0, 0.0);
    aPnts[3] = Point3::new(3.0, 1.0, 0.0);
    aPnts[4] = Point3::new(5.0, 3.0, 0.0);
    aPnts[5] = Point3::new(7.0, 0.0, 0.0);

    let anInterp = points_to_bspline(&aPnts);
    assert!(anInterp.is_done());
    let aCurve = anInterp.curve().unwrap();

    let aFirst = aCurve.first_parameter();
    let aLast = aCurve.last_parameter();
    let aMid = (aFirst + aLast) / 2.0;

    // Compare EvalD1 with old D1 wrapper
    let aEvalResult = aCurve.eval_d1(aMid).unwrap();

    let (aOldP, aOldV1) = aCurve.d1(aMid).unwrap();

    assert_near(
        aEvalResult.point.distance(&aOldP),
        0.0,
        precision::confusion(),
    );
    assert_near(
        (aEvalResult.d1 - aOldV1).magnitude(),
        0.0,
        precision::confusion(),
    );
}

//=================================================================================================
// EvalDN consistency tests
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__EvalDN_N1_ConsistentWithEvalD1() {
    let aCurve = origin_circle(3.0);

    let aU = PI / 6.0;

    let aD1 = aCurve.eval_d1(aU).unwrap();
    let aDN1 = aCurve.eval_dn(aU, 1).unwrap();
    assert_near((aD1.d1 - aDN1).magnitude(), 0.0, precision::confusion());
}

//=================================================================================================
// OffsetCurve failure-path tests
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__OffsetCurve_EvalD0_ReturnsNulloptAtSingular() {
    // Create a circle with radius equal to offset -> singular at all points
    let aBasis = origin_circle(5.0);

    // Create a zero-radius circle by offsetting inward by -radius
    // This creates a degenerate curve at the center
    let anOffset = GeomOffsetCurve::new(&aBasis, -5.0, dir(0.0, 0.0, 1.0), false).unwrap();

    // For a circle offset by -radius, the resulting curve is a single point (the center).
    // EvalD0 should still succeed (it's just a point), but D1 may fail at singular points.
    // Actually, the offset curve evaluation uses the basis curve's tangent for computing
    // the offset direction, and a degenerate offset curve may or may not fail at D0.
    // Let's just verify the API works and returns a value.
    let aResult = anOffset.eval_d0(0.0).unwrap();
    // For a circle offset by -radius, this should still return a valid point (the center)
    assert_near(
        aResult.distance(&Point3::new(0.0, 0.0, 0.0)),
        0.0,
        precision::confusion(),
    );
}

#[test]
fn Geom_CurveEvalTest__OffsetCurve_D0Wrapper_ThrowsAtSingular() {
    // Create a line with zero-length tangent (not possible for a line, but we can test the
    // backward-compatibility wrapper by using a normal offset curve that works fine)
    let aBasis = origin_circle(5.0);

    let anOffset = GeomOffsetCurve::new(&aBasis, 2.0, dir(0.0, 0.0, 1.0), false).unwrap();

    // The backward-compatibility wrapper should NOT throw for valid evaluation
    assert!(anOffset.d0(0.0).is_ok());
}

//=================================================================================================
// Parabola tests
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__Parabola_EvalD3_ZeroThirdDerivative() {
    // Parabola: y = x^2 / (4*f), third derivative is zero
    let frame = Axis2::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    let aCurve = GeomParabola::new(frame, 2.0).unwrap();

    let aD3 = aCurve.eval_d3(1.0).unwrap();
    assert_near(aD3.d3.magnitude(), 0.0, precision::confusion());
}

//=================================================================================================
// BSplineCurve EvalDN invalid N test
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__BSplineCurve_EvalDN_InvalidN_Throws() {
    let mut aPnts: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    aPnts[1] = Point3::new(0.0, 0.0, 0.0);
    aPnts[2] = Point3::new(1.0, 1.0, 0.0);
    aPnts[3] = Point3::new(2.0, 0.0, 0.0);

    let anInterp = points_to_bspline(&aPnts);
    assert!(anInterp.is_done());
    let aCurve = anInterp.curve().unwrap();

    let aMid = (aCurve.first_parameter() + aCurve.last_parameter()) / 2.0;

    let thrown = |n: i32| {
        aCurve
            .eval_dn(aMid, n)
            .unwrap_err()
            .exception_type()
            .to_string()
    };
    assert_eq!(thrown(0), "Geom_UndefinedDerivative");
    assert_eq!(thrown(-1), "Geom_UndefinedDerivative");
}

//=================================================================================================
// TrimmedCurve delegation test
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__TrimmedCurve_EvalD1_DelegatesToBasis() {
    let aBasis = origin_circle(5.0);

    let aTrimmed = GeomTrimmedCurve::new(&aBasis, 0.0, PI, true, true).unwrap();

    let aU = PI / 4.0;

    let aBasisD1 = aBasis.eval_d1(aU).unwrap();
    let aTrimmedD1 = aTrimmed.eval_d1(aU).unwrap();

    assert_near(
        aBasisD1.point.distance(&aTrimmedD1.point),
        0.0,
        precision::confusion(),
    );
    assert_near(
        (aBasisD1.d1 - aTrimmedD1.d1).magnitude(),
        0.0,
        precision::confusion(),
    );
}

//=================================================================================================
// OffsetCurve EvalD1 singular test
//=================================================================================================

#[test]
fn Geom_CurveEvalTest__OffsetCurve_EvalD1_DegenerateAtCollapsed() {
    let aBasis = origin_circle(5.0);

    // Offset by -radius creates a collapsed curve at the center.
    // The basis circle has valid tangents everywhere, so EvalD1 succeeds
    // but produces a degenerate result (all points map to the center).
    let anOffset = GeomOffsetCurve::new(&aBasis, -5.0, dir(0.0, 0.0, 1.0), false).unwrap();

    let aResult = anOffset.eval_d1(0.0).unwrap();

    // The point should be near the center
    assert_near(
        aResult.point.distance(&Point3::new(0.0, 0.0, 0.0)),
        0.0,
        precision::confusion(),
    );
}
