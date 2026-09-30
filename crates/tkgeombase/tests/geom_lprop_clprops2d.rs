//! Direct translation of GeomLProp_CLProps2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dLine};
use rustycad_tkgeombase::geom_lprop::CLProps2d;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Ellipse2, Line2, Point2, Vector2};
use std::f64::consts::PI;

/// The GeomLProp_CLProps2dTest fixture (SetUp).
struct Fixture {
    myCircle: Handle<dyn Geom2dCurve>,
    myLine: Handle<dyn Geom2dCurve>,
    myEllipse: Handle<dyn Geom2dCurve>,
}

fn SetUp() -> Fixture {
    // Circle centered at origin, radius 5
    let anAxis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let aCirc = Circle2::from_x_axis(anAxis, 5.0, true).unwrap();
    // Line through origin along X
    let aLin = Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    // Ellipse centered at origin, major radius 10, minor radius 5
    let anElips = Ellipse2::from_x_axis(anAxis, 10.0, 5.0, true).unwrap();
    Fixture {
        myCircle: Handle::new(Geom2dCircle::from_circ2d(aCirc)).cast(),
        myLine: Handle::new(Geom2dLine::from_lin2d(aLin)).cast(),
        myEllipse: Handle::new(Geom2dEllipse::from_elips2d(anElips)).cast(),
    }
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_Value() {
    let f = SetUp();
    let aProps = CLProps2d::with_parameter(f.myCircle, 0.0, 2, precision::confusion()).unwrap();

    let aPnt: Point2 = aProps.value();
    EXPECT_NEAR(aPnt.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 0.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_SetParameter() {
    let f = SetUp();
    let mut aProps = CLProps2d::new(f.myCircle, 2, precision::confusion()).unwrap();

    aProps.set_parameter(PI / 2.0).unwrap();
    let aPnt: Point2 = aProps.value();
    EXPECT_NEAR(aPnt.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 5.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_TangentDefined() {
    let f = SetUp();
    let mut aProps = CLProps2d::with_parameter(f.myCircle, 0.0, 2, precision::confusion()).unwrap();
    assert!(aProps.is_tangent_defined().unwrap());
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_TangentAtZero() {
    let f = SetUp();
    // At U=0 on a circle, the tangent should point in +Y direction
    let mut aProps = CLProps2d::with_parameter(f.myCircle, 0.0, 2, precision::confusion()).unwrap();

    let aTangent: Direction2 = aProps.tangent().unwrap();
    EXPECT_NEAR(aTangent.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aTangent.y(), 1.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_Curvature() {
    let f = SetUp();
    // Curvature of a circle of radius R is 1/R
    let mut aProps = CLProps2d::with_parameter(f.myCircle, 0.0, 2, precision::confusion()).unwrap();

    let aCurvature: f64 = aProps.curvature().unwrap();
    EXPECT_NEAR(aCurvature, 1.0 / 5.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_CurvatureConstant() {
    let f = SetUp();
    // Curvature on a circle should be the same at every point
    let mut aProps = CLProps2d::new(f.myCircle, 2, precision::confusion()).unwrap();

    let mut u: f64 = 0.0;
    while u < 2.0 * PI {
        aProps.set_parameter(u).unwrap();
        EXPECT_NEAR(
            aProps.curvature().unwrap(),
            1.0 / 5.0,
            precision::confusion(),
        );
        u += PI / 4.0;
    }
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_Normal() {
    let f = SetUp();
    // Normal at U=0 on a circle centered at origin should point inward (-X)
    let mut aProps = CLProps2d::with_parameter(f.myCircle, 0.0, 2, precision::confusion()).unwrap();

    let aNormal: Direction2 = aProps.normal().unwrap();
    // Normal should be perpendicular to tangent
    let aTangent: Direction2 = aProps.tangent().unwrap();
    EXPECT_NEAR(
        (aTangent.x() * aNormal.x() + aTangent.y() * aNormal.y()).abs(),
        0.0,
        precision::confusion(),
    );
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_CentreOfCurvature() {
    let f = SetUp();
    // Centre of curvature of a circle is its center
    let mut aProps = CLProps2d::with_parameter(f.myCircle, 0.0, 2, precision::confusion()).unwrap();

    let aCenter: Point2 = aProps.centre_of_curvature().unwrap();
    EXPECT_NEAR(aCenter.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aCenter.y(), 0.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__Circle_CentreOfCurvature_AtPiHalf() {
    let f = SetUp();
    let mut aProps =
        CLProps2d::with_parameter(f.myCircle, PI / 2.0, 2, precision::confusion()).unwrap();

    let aCenter: Point2 = aProps.centre_of_curvature().unwrap();
    EXPECT_NEAR(aCenter.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aCenter.y(), 0.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__Line_TangentDefined() {
    let f = SetUp();
    let mut aProps = CLProps2d::with_parameter(f.myLine, 0.0, 2, precision::confusion()).unwrap();
    assert!(aProps.is_tangent_defined().unwrap());
}

#[test]
fn GeomLProp_CLProps2dTest__Line_Tangent() {
    let f = SetUp();
    let mut aProps = CLProps2d::with_parameter(f.myLine, 5.0, 2, precision::confusion()).unwrap();

    let aTangent: Direction2 = aProps.tangent().unwrap();
    EXPECT_NEAR(aTangent.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aTangent.y(), 0.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__Line_CurvatureIsZero() {
    let f = SetUp();
    let mut aProps = CLProps2d::with_parameter(f.myLine, 5.0, 2, precision::confusion()).unwrap();
    let aCurvature: f64 = aProps.curvature().unwrap();
    EXPECT_NEAR(aCurvature, 0.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__Line_CentreOfCurvature_Throws() {
    let f = SetUp();
    // Centre of curvature is not defined for a line (zero curvature)
    let mut aProps = CLProps2d::with_parameter(f.myLine, 5.0, 2, precision::confusion()).unwrap();
    EXPECT_THROW(aProps.centre_of_curvature(), "LProp_NotDefined");
}

#[test]
fn GeomLProp_CLProps2dTest__Ellipse_CurvatureAtMajorVertex() {
    let f = SetUp();
    // Radius of curvature at the major vertex = b^2/a, so curvature = a/b^2
    let mut aProps =
        CLProps2d::with_parameter(f.myEllipse, 0.0, 2, precision::confusion()).unwrap();

    let aCurvature: f64 = aProps.curvature().unwrap();
    let aExpected: f64 = 10.0 / (5.0 * 5.0); // a/b^2 = 10/25 = 0.4
    EXPECT_NEAR(aCurvature, aExpected, 1e-6);
}

#[test]
fn GeomLProp_CLProps2dTest__Ellipse_CurvatureAtMinorVertex() {
    let f = SetUp();
    // At U=PI/2, ellipse is at minor vertex. Curvature = b/a^2
    let mut aProps =
        CLProps2d::with_parameter(f.myEllipse, PI / 2.0, 2, precision::confusion()).unwrap();

    let aCurvature: f64 = aProps.curvature().unwrap();
    let aExpected: f64 = 5.0 / (10.0 * 10.0); // b/a^2 = 5/100 = 0.05
    EXPECT_NEAR(aCurvature, aExpected, 1e-6);
}

#[test]
fn GeomLProp_CLProps2dTest__SetCurve() {
    let f = SetUp();
    let mut aProps =
        CLProps2d::with_parameter(f.myEllipse, 0.0, 2, precision::confusion()).unwrap();
    aProps.set_curve(f.myCircle);
    aProps.set_parameter(0.0).unwrap();

    let aPnt: Point2 = aProps.value();
    EXPECT_NEAR(aPnt.x(), 5.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__D1_Circle() {
    let f = SetUp();
    let mut aProps = CLProps2d::with_parameter(f.myCircle, 0.0, 2, precision::confusion()).unwrap();
    let aD1: Vector2 = aProps.d1().unwrap();

    // First derivative at U=0 on circle(R=5) is (0, 5)
    EXPECT_NEAR(aD1.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aD1.y(), 5.0, precision::confusion());
}

#[test]
fn GeomLProp_CLProps2dTest__D2_Circle() {
    let f = SetUp();
    let mut aProps = CLProps2d::with_parameter(f.myCircle, 0.0, 2, precision::confusion()).unwrap();
    let aD2: Vector2 = aProps.d2().unwrap();

    // Second derivative at U=0 on circle(R=5) is (-5, 0)
    EXPECT_NEAR(aD2.x(), -5.0, precision::confusion());
    EXPECT_NEAR(aD2.y(), 0.0, precision::confusion());
}
