//! Direct translation of GeomGridEval_Ellipse_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomEllipse};
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3, GeomGridEvalEllipse,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-10;

// `new Geom_Ellipse(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 3.0, 2.0)`.
fn Ellipse() -> Handle<GeomEllipse> {
    let anAxis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Handle::new(GeomEllipse::new(anAxis, 3.0, 2.0).unwrap())
}

#[test]
fn GeomGridEval_EllipseTest__BasicEvaluation() {
    // Ellipse in XY plane, Major=3, Minor=2, center at origin
    let anEllipse = Ellipse();

    let anEval = GeomGridEvalEllipse::new(anEllipse);
    assert!(!anEval.geometry().is_null());

    // Test at 0, PI/2, PI, 3PI/2, 2PI
    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 5).unwrap();
    aParams[1] = 0.0;
    aParams[2] = PI / 2.0;
    aParams[3] = PI;
    aParams[4] = 3.0 * PI / 2.0;
    aParams[5] = 2.0 * PI;

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // u=0: (3, 0, 0)
    EXPECT_NEAR(aGrid[1].x(), 3.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[1].y(), 0.0, THE_TOLERANCE);

    // u=PI/2: (0, 2, 0)
    EXPECT_NEAR(aGrid[2].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[2].y(), 2.0, THE_TOLERANCE);

    // u=PI: (-3, 0, 0)
    EXPECT_NEAR(aGrid[3].x(), -3.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[3].y(), 0.0, THE_TOLERANCE);

    // u=3PI/2: (0, -2, 0)
    EXPECT_NEAR(aGrid[4].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[4].y(), -2.0, THE_TOLERANCE);

    // u=2PI: back to (3, 0, 0)
    EXPECT_NEAR(aGrid[5].x(), 3.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[5].y(), 0.0, THE_TOLERANCE);
}

#[test]
fn GeomGridEval_EllipseTest__DerivativeD1() {
    let anEllipse = Ellipse();
    let anEval = GeomGridEvalEllipse::new(anEllipse.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<GeomGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1) = anEllipse.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_EllipseTest__DerivativeD2() {
    let anEllipse = Ellipse();
    let anEval = GeomGridEvalEllipse::new(anEllipse.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<GeomGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2) = anEllipse.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_EllipseTest__DerivativeD3() {
    let anEllipse = Ellipse();
    let anEval = GeomGridEvalEllipse::new(anEllipse.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2, aD3) = anEllipse.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}
