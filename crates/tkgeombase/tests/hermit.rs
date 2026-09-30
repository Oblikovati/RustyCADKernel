//! Direct translation of Hermit_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeombase::hermit;
use rustycad_tkmath::gp::{Point2, Point3};

// The native default tolerances of Hermit::Solution and Hermit::Solutionbis.
const TOL: f64 = 0.000001;

fn weights(theW1: f64, theW2: f64, theW3: f64) -> Array1<'static, f64> {
    let mut aWeights = Array1::with_bounds(1, 3).unwrap();
    aWeights[1] = theW1;
    aWeights[2] = theW2;
    aWeights[3] = theW3;
    aWeights
}

fn knots_and_mults() -> (Array1<'static, f64>, Array1<'static, i32>) {
    let mut aKnots = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    let mut aMults = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 3;
    aMults[2] = 3;
    (aKnots, aMults)
}

// Helper to create a simple rational BSpline curve (degree 2, 3 poles)
// with weights that produce distinct endpoint weight values.
fn MakeRationalBSpline3D(theW1: f64, theW2: f64, theW3: f64) -> Handle<GeomBSplineCurve> {
    let mut aPoles = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 1.0, 0.0);
    aPoles[3] = Point3::new(2.0, 0.0, 0.0);
    let (aKnots, aMults) = knots_and_mults();
    let aWeights = weights(theW1, theW2, theW3);
    Handle::new(
        GeomBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 2, false).unwrap(),
    )
}

// Helper to create a simple rational BSpline 2D curve (degree 2, 3 poles).
fn MakeRationalBSpline2D(theW1: f64, theW2: f64, theW3: f64) -> Handle<Geom2dBSplineCurve> {
    let mut aPoles = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 1.0);
    aPoles[3] = Point2::new(2.0, 0.0);
    let (aKnots, aMults) = knots_and_mults();
    let aWeights = weights(theW1, theW2, theW3);
    Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 2, false).unwrap(),
    )
}

// The values of a(u) at both ends.
fn end_values(aResult: &Handle<Geom2dBSplineCurve>) -> (Point2, Point2) {
    let aP0 = aResult.value(aResult.first_parameter()).unwrap();
    let aP1 = aResult.value(aResult.last_parameter()).unwrap();
    (aP0, aP1)
}

#[test]
fn HermitTest__Solution3D_UniformWeights_ReturnsValidCurve() {
    // Uniform weights => weight function is constant 1.0 everywhere
    let aBS = MakeRationalBSpline3D(1.0, 1.0, 1.0);

    let aResult = hermit::solution_3d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());
    assert!(aResult.nb_poles() >= 4);

    // The result a(u) * D(u) should have value 1 at endpoints
    // For uniform weights D(u)=1 everywhere, so a(0)=1, a(1)=1
    let (aP0, aP1) = end_values(&aResult);

    EXPECT_NEAR(aP0.y(), 1.0, 1.0e-6);
    EXPECT_NEAR(aP1.y(), 1.0, 1.0e-6);
}

#[test]
fn HermitTest__Solution3D_DistinctWeights_ReturnsValidCurve() {
    // Different weights at endpoints: w(0)=2, w(1)=3
    let aBS = MakeRationalBSpline3D(2.0, 1.5, 3.0);

    let aResult = hermit::solution_3d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());

    // a(0) should be 1/w(0) = 0.5, a(1) should be 1/w(1) = 1/3
    let (aP0, aP1) = end_values(&aResult);

    EXPECT_NEAR(aP0.y(), 0.5, 1.0e-4);
    EXPECT_NEAR(aP1.y(), 1.0 / 3.0, 1.0e-4);
}

#[test]
fn HermitTest__Solution3D_HighWeightRatio_Endpoint() {
    // Large weight ratio: w(0) small, w(1) large - tests the Pole0 < Pole3 branch
    let aBS = MakeRationalBSpline3D(0.5, 1.0, 5.0);

    let aResult = hermit::solution_3d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());

    let (aP0, aP1) = end_values(&aResult);

    // a(u)*D(u) = 1 at endpoints => a(0) = 1/w(0) = 2, a(1) = 1/w(1) = 0.2
    EXPECT_NEAR(aP0.y(), 1.0 / 0.5, 1.0e-4);
    EXPECT_NEAR(aP1.y(), 1.0 / 5.0, 1.0e-4);
}

#[test]
fn HermitTest__Solution3D_ReversedWeightRatio_Endpoint() {
    // Reversed ratio: w(0) large, w(1) small - tests the Pole0 > Pole3 branch
    let aBS = MakeRationalBSpline3D(5.0, 1.0, 0.5);

    let aResult = hermit::solution_3d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());

    let (aP0, aP1) = end_values(&aResult);

    EXPECT_NEAR(aP0.y(), 1.0 / 5.0, 1.0e-4);
    EXPECT_NEAR(aP1.y(), 1.0 / 0.5, 1.0e-4);
}

#[test]
fn HermitTest__Solution3D_PositivePoles() {
    // The result curve should have all positive Y coordinates (positive denominator)
    let aBS = MakeRationalBSpline3D(2.0, 3.0, 1.5);

    let aResult = hermit::solution_3d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());
    for i in 1..=aResult.nb_poles() {
        assert!(
            aResult.pole(i).unwrap().y() > 0.0,
            "Pole {i} Y should be positive"
        );
    }
}

#[test]
fn HermitTest__Solution2D_UniformWeights_ReturnsValidCurve() {
    let aBS = MakeRationalBSpline2D(1.0, 1.0, 1.0);

    let aResult = hermit::solution_2d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());

    let (aP0, aP1) = end_values(&aResult);

    EXPECT_NEAR(aP0.y(), 1.0, 1.0e-6);
    EXPECT_NEAR(aP1.y(), 1.0, 1.0e-6);
}

#[test]
fn HermitTest__Solution2D_DistinctWeights_ReturnsValidCurve() {
    let aBS = MakeRationalBSpline2D(2.0, 1.5, 3.0);

    let aResult = hermit::solution_2d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());

    let (aP0, aP1) = end_values(&aResult);

    EXPECT_NEAR(aP0.y(), 0.5, 1.0e-4);
    EXPECT_NEAR(aP1.y(), 1.0 / 3.0, 1.0e-4);
}

#[test]
fn HermitTest__Solution2D_HighWeightRatio_Endpoint() {
    // Large ratio tests the Pole0 < Pole3 branch in 2D PolyTest
    let aBS = MakeRationalBSpline2D(0.5, 1.0, 5.0);

    let aResult = hermit::solution_2d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());

    let (aP0, aP1) = end_values(&aResult);

    EXPECT_NEAR(aP0.y(), 1.0 / 0.5, 1.0e-4);
    EXPECT_NEAR(aP1.y(), 1.0 / 5.0, 1.0e-4);
}

#[test]
fn HermitTest__Solutionbis_UniformWeights_KnotsUnchanged() {
    let aBS = MakeRationalBSpline3D(1.0, 1.0, 1.0);

    let mut aKnotmin: f64 = 0.0;
    let mut aKnotmax: f64 = 1.0;

    hermit::solutionbis(&aBS, &mut aKnotmin, &mut aKnotmax, TOL, TOL).unwrap();

    // For uniform weights, the Hermite coefficients are trivial (1,0,0,1)
    // and no tolerance knots are needed, so the output knots should be
    // the second and second-to-last of the resulting BS2
    assert!(aKnotmin >= 0.0);
    assert!(aKnotmax <= 1.0);
}

#[test]
fn HermitTest__Solutionbis_DistinctWeights_ReturnsValidKnots() {
    let aBS = MakeRationalBSpline3D(2.0, 1.5, 3.0);

    let mut aKnotmin: f64 = 0.0;
    let mut aKnotmax: f64 = 1.0;

    hermit::solutionbis(&aBS, &mut aKnotmin, &mut aKnotmax, TOL, TOL).unwrap();

    assert!(aKnotmin >= 0.0);
    assert!(aKnotmax <= 1.0);
    assert!(aKnotmin <= aKnotmax);
}

#[test]
fn HermitTest__Solution3D_Symmetric_WeightsProduceSymmetricResult() {
    // Symmetric weights: w(0) == w(1), so a(0) == a(1)
    let aBS = MakeRationalBSpline3D(2.0, 1.0, 2.0);

    let aResult = hermit::solution_3d(&aBS, TOL, TOL).unwrap();

    assert!(!aResult.is_null());

    let (aP0, aP1) = end_values(&aResult);

    // Both endpoints should yield the same value: 1/w = 0.5
    EXPECT_NEAR(aP0.y(), 0.5, 1.0e-4);
    EXPECT_NEAR(aP1.y(), 0.5, 1.0e-4);
}
