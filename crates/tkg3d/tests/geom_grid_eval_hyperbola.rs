//! Direct translation of GeomGridEval_Hyperbola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomHyperbola};
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3, GeomGridEvalHyperbola,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};

const THE_TOLERANCE: f64 = 1e-10;

// `new Geom_Hyperbola(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 3.0, 2.0)`.
fn Hyperbola() -> Handle<GeomHyperbola> {
    let anAxis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Handle::new(GeomHyperbola::new(anAxis, 3.0, 2.0).unwrap())
}

#[test]
fn GeomGridEval_HyperbolaTest__BasicEvaluation() {
    // Hyperbola in XY plane, Major=3, Minor=2, center at origin
    let aHypr = Hyperbola();

    let anEval = GeomGridEvalHyperbola::new(aHypr.clone());
    assert!(!anEval.geometry().is_null());

    // Test from -2 to 2
    let aParams = CreateUniformParams(-2.0, 2.0, 9);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 9);

    // Verify points
    for i in 1..=9 {
        let aExpected: Point3 = aHypr.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(&aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_HyperbolaTest__DerivativeD1() {
    let aHypr = Hyperbola();
    let anEval = GeomGridEvalHyperbola::new(aHypr.clone());

    let aParams = CreateUniformParams(-2.0, 2.0, 9);

    let aGrid: Array1<GeomGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1) = aHypr.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_HyperbolaTest__DerivativeD2() {
    let aHypr = Hyperbola();
    let anEval = GeomGridEvalHyperbola::new(aHypr.clone());

    let aParams = CreateUniformParams(-2.0, 2.0, 9);

    let aGrid: Array1<GeomGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2) = aHypr.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_HyperbolaTest__DerivativeD3() {
    let aHypr = Hyperbola();
    let anEval = GeomGridEvalHyperbola::new(aHypr.clone());

    let aParams = CreateUniformParams(-2.0, 2.0, 9);

    let aGrid: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2, aD3) = aHypr.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(&aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}
