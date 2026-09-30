//! Direct translation of Geom2dGridEval_Parabola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dParabola};
use rustycad_tkg2d::geom2d_grid_eval::{
    Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3, Geom2dGridEvalParabola,
};
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2, Vector2};

const THE_TOLERANCE: f64 = 1e-10;

// `new Geom2d_Parabola(gp_Ax22d(gp_Pnt2d(x, y), gp_Dir2d(1, 0)), focal)`.
fn Parabola(x: f64, y: f64, focal: f64) -> Handle<Geom2dParabola> {
    let aAxis = Axis22d::new(Point2::new(x, y), Direction2::new(1.0, 0.0).unwrap(), true).unwrap();
    Handle::new(Geom2dParabola::new(aAxis, focal).unwrap())
}

#[test]
fn Geom2dGridEval_ParabolaTest__BasicD0() {
    let aParab = Parabola(0.0, 0.0, 1.0);

    let anEval = Geom2dGridEvalParabola::new(aParab.clone());
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(-3.0, 3.0, 13);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 13);

    for i in 1..=13 {
        let aExpected: Point2 = aParab.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_ParabolaTest__AtOrigin() {
    let aParab = Parabola(0.0, 0.0, 1.0);

    let anEval = Geom2dGridEvalParabola::new(aParab);

    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 1).unwrap();
    aParams[1] = 0.0;

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    // At u=0: P = Center + 0*XDir + 0*YDir = (0, 0)
    EXPECT_NEAR(aGrid[1].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[1].y(), 0.0, THE_TOLERANCE);
}

#[test]
fn Geom2dGridEval_ParabolaTest__D1() {
    let aParab = Parabola(1.0, 2.0, 2.0);

    let anEval = Geom2dGridEvalParabola::new(aParab.clone());

    let aParams = CreateUniformParams(-3.0, 3.0, 13);

    let aGrid: Array1<Geom2dGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=13 {
        let (aPnt, aD1) = aParab.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_ParabolaTest__D2() {
    let aParab = Parabola(0.0, 0.0, 1.0);

    let anEval = Geom2dGridEvalParabola::new(aParab.clone());

    let aParams = CreateUniformParams(-3.0, 3.0, 13);

    let aGrid: Array1<Geom2dGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=13 {
        let (aPnt, aD1, aD2) = aParab.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_ParabolaTest__D3() {
    let aParab = Parabola(0.0, 0.0, 1.0);

    let anEval = Geom2dGridEvalParabola::new(aParab.clone());

    let aParams = CreateUniformParams(-3.0, 3.0, 13);

    let aGrid: Array1<Geom2dGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=13 {
        let (aPnt, aD1, aD2, aD3) = aParab.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_ParabolaTest__DN_HighOrder() {
    let aParab = Parabola(0.0, 0.0, 1.0);

    let anEval = Geom2dGridEvalParabola::new(aParab);

    let aParams = CreateUniformParams(-3.0, 3.0, 7);

    // D3 and higher should be zero for parabola
    let aGrid: Array1<Vector2> = anEval.evaluate_grid_dn(&aParams, 3).unwrap();

    for i in 1..=7 {
        EXPECT_NEAR(aGrid[i].magnitude(), 0.0, THE_TOLERANCE);
    }
}
