//! Direct translation of GeomGridEval_BezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalBezierCurve, GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3,
};
use rustycad_tkmath::gp::{Point3, Vector3};

const THE_TOLERANCE: f64 = 1e-10;

// The cubic Bezier curve that most tests build.
fn CubicBezier() -> Handle<GeomBezierCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(3.0, 2.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);
    Handle::new(GeomBezierCurve::new(&aPoles).unwrap())
}

// Rational Bezier curve (quarter circle)
fn QuarterCircleBezier() -> Handle<GeomBezierCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();

    aPoles[1] = Point3::new(1.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 1.0, 0.0);
    aPoles[3] = Point3::new(0.0, 1.0, 0.0);

    aWeights[1] = 1.0;
    aWeights[2] = 1.0 / 2.0_f64.sqrt();
    aWeights[3] = 1.0;

    Handle::new(GeomBezierCurve::with_weights(&aPoles, &aWeights).unwrap())
}

#[test]
fn GeomGridEval_BezierCurveTest__BasicEvaluation() {
    // Create a cubic Bezier curve
    let aBezier = CubicBezier();

    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());
    assert!(!anEval.geometry().is_null());

    // Test at 0, 0.5, 1
    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    aParams[1] = 0.0;
    aParams[2] = 0.5;
    aParams[3] = 1.0;

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 3);

    // Verify points
    for i in 1..=3 {
        let aExpected: Point3 = aBezier.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(&aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__DerivativeD1() {
    let aBezier = CubicBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array1<GeomGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=5 {
        let (aPnt, aD1): (Point3, Vector3) = aBezier.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__DerivativeD2() {
    let aBezier = CubicBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array1<GeomGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=5 {
        let (aPnt, aD1, aD2) = aBezier.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__DerivativeD3() {
    let aBezier = CubicBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=5 {
        let (aPnt, aD1, aD2, aD3) = aBezier.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__RationalEvaluation() {
    // Rational Bezier curve (quarter circle)
    let aBezier = QuarterCircleBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=11 {
        let aExpected: Point3 = aBezier.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(&aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__DerivativeDN_Order1() {
    let aBezier = CubicBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array1<Vector3> = anEval.evaluate_grid_dn(&aParams, 1).unwrap();

    for i in 1..=5 {
        let aExpected: Vector3 = aBezier.dn(aParams[i], 1).unwrap();
        EXPECT_NEAR((aGrid[i] - aExpected).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__DerivativeDN_Order2() {
    let aBezier = CubicBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array1<Vector3> = anEval.evaluate_grid_dn(&aParams, 2).unwrap();

    for i in 1..=5 {
        let aExpected: Vector3 = aBezier.dn(aParams[i], 2).unwrap();
        EXPECT_NEAR((aGrid[i] - aExpected).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__DerivativeDN_Order3() {
    let aBezier = CubicBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array1<Vector3> = anEval.evaluate_grid_dn(&aParams, 3).unwrap();

    for i in 1..=5 {
        let aExpected: Vector3 = aBezier.dn(aParams[i], 3).unwrap();
        EXPECT_NEAR((aGrid[i] - aExpected).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__DerivativeDN_BeyondDegree() {
    // Cubic Bezier (degree 3), test DN for N > 3 should return zero vectors
    let aBezier = CubicBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier);

    let aParams = CreateUniformParams(0.0, 1.0, 5);

    // 4th derivative of cubic Bezier should be zero
    let aGrid: Array1<Vector3> = anEval.evaluate_grid_dn(&aParams, 4).unwrap();

    for i in 1..=5 {
        EXPECT_NEAR(aGrid[i].magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierCurveTest__DerivativeDN_RationalCurve() {
    // Rational Bezier curve (quarter circle)
    let aBezier = QuarterCircleBezier();
    let anEval = GeomGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    // Test DN for orders 1, 2
    for aOrder in 1..=2 {
        let aGrid: Array1<Vector3> = anEval.evaluate_grid_dn(&aParams, aOrder).unwrap();

        for i in 1..=11 {
            let aExpected: Vector3 = aBezier.dn(aParams[i], aOrder).unwrap();
            EXPECT_NEAR((aGrid[i] - aExpected).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}
