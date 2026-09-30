//! Direct translation of Geom2dHatch_Intersector_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_hatch::Intersector;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};

// Verify LocalGeometry produces valid outputs on a circle (well-defined tangent).
#[test]
fn Geom2dHatch_Intersector__LocalGeometry_Circle_ValidOutputs() {
    let anAxis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let aCircle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(anAxis, 1.0, true).unwrap()).cast();
    let aCurveAdaptor = Geom2dAdaptorCurve::with_curve(&aCircle).unwrap();

    let anIntersector = Intersector::new(1.0e-7, 1.0e-7);

    let (aTang, _aNorm, aCurv) = anIntersector.local_geometry(&aCurveAdaptor, 0.0).unwrap();

    // Circle tangent at param=0 is (0,1), normal is (-1,0), curvature is 1.0.
    EXPECT_NEAR(aTang.x(), 0.0, 1.0e-10);
    EXPECT_NEAR(aTang.y(), 1.0, 1.0e-10);
    EXPECT_NEAR(aCurv, 1.0, 1.0e-10);
}

// Verify LocalGeometry produces valid outputs on a straight line (zero curvature).
#[test]
fn Geom2dHatch_Intersector__LocalGeometry_Line_ZeroCurvature() {
    let anAxis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let aLine: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(anAxis)).cast();
    let aCurveAdaptor = Geom2dAdaptorCurve::with_curve(&aLine).unwrap();

    let anIntersector = Intersector::new(1.0e-7, 1.0e-7);

    let (aTang, aNorm, aCurv) = anIntersector.local_geometry(&aCurveAdaptor, 0.5).unwrap();

    EXPECT_NEAR(aTang.x(), 1.0, 1.0e-10);
    EXPECT_NEAR(aTang.y(), 0.0, 1.0e-10);
    EXPECT_NEAR(aCurv, 0.0, 1.0e-10);
    // Normal should be perpendicular to tangent.
    EXPECT_NEAR(aNorm.x(), 0.0, 1.0e-10);
    EXPECT_NEAR(aNorm.y().abs(), 1.0, 1.0e-10);
}

// Regression test for bug #24: LocalGeometry with undefined tangent
// must still produce initialized (deterministic) output values.
#[test]
fn Geom2dHatch_Intersector__LocalGeometry_DegenerateCurve_InitializedOutputs() {
    // Construct a degenerate BSpline where all control points coincide,
    // causing IsTangentDefined() to return false.
    let mut aPoles = Array1::<Point2>::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(1.0, 1.0);
    aPoles[2] = Point2::new(1.0, 1.0);
    aPoles[3] = Point2::new(1.0, 1.0);
    aPoles[4] = Point2::new(1.0, 1.0);

    let mut aKnots = Array1::<f64>::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMults = Array1::<i32>::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;

    let aDegenerateCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap()).cast();
    let aCurveAdaptor = Geom2dAdaptorCurve::with_curve(&aDegenerateCurve).unwrap();

    let anIntersector = Intersector::new(1.0e-7, 1.0e-7);

    // Must not crash and must produce initialized outputs.
    let aResult = anIntersector.local_geometry(&aCurveAdaptor, 0.5);
    assert!(aResult.is_ok());
    let (aTang, aNorm, aCurv) = aResult.unwrap();

    // Curvature should be initialized to 0 for degenerate case.
    EXPECT_NEAR(aCurv, 0.0, 1.0e-10);
    // Tang and Norm should be valid unit directions (magnitude == 1).
    let aTangMag = (aTang.x() * aTang.x() + aTang.y() * aTang.y()).sqrt();
    EXPECT_NEAR(aTangMag, 1.0, 1.0e-10);
    let aNormMag = (aNorm.x() * aNorm.x() + aNorm.y() * aNorm.y()).sqrt();
    EXPECT_NEAR(aNormMag, 1.0, 1.0e-10);
}
