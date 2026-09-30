//! Direct translation of Geom2d_Line_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dGeometry, Geom2dLine};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2d, Direction2, Line2, Point2, Transform2, Vector2};
use std::f64::consts::PI;

fn dir(x: f64, y: f64) -> Direction2 {
    Direction2::new(x, y).unwrap()
}

// Geom2d_LineTest::SetUp: horizontal line through origin.
fn setup() -> Handle<Geom2dLine> {
    Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), dir(1.0, 0.0)))
}

#[test]
fn Geom2d_LineTest__ConstructFromPointAndDir() {
    let myLine = setup();
    EXPECT_NEAR(myLine.direction().x(), 1.0, precision::confusion());
    EXPECT_NEAR(myLine.direction().y(), 0.0, precision::confusion());
    EXPECT_NEAR(myLine.location().x(), 0.0, precision::confusion());
    EXPECT_NEAR(myLine.location().y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__ConstructFromLin2d() {
    let aLin = Line2::new(Point2::new(1.0, 2.0), dir(0.0, 1.0));
    let aLine = Handle::new(Geom2dLine::from_lin2d(aLin));

    EXPECT_NEAR(aLine.direction().x(), 0.0, precision::confusion());
    EXPECT_NEAR(aLine.direction().y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__ConstructFromAxis() {
    let anAxis = Axis2d::new(Point2::new(3.0, 4.0), dir(1.0, 0.0));
    let aLine = Handle::new(Geom2dLine::new(anAxis));

    EXPECT_NEAR(aLine.location().x(), 3.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__ParameterBounds() {
    let myLine = setup();
    // OCCT uses large finite bounds (not numeric_limits::max)
    assert!(myLine.first_parameter() < -1e+90);
    assert!(myLine.last_parameter() > 1e+90);
}

#[test]
fn Geom2d_LineTest__IsNotClosedNotPeriodic() {
    let myLine = setup();
    assert!(!myLine.is_closed().unwrap());
    assert!(!myLine.is_periodic());
}

#[test]
fn Geom2d_LineTest__Continuity() {
    let myLine = setup();
    assert_eq!(myLine.continuity(), Shape::CN);
    assert!(myLine.is_cn(100).unwrap());
}

#[test]
fn Geom2d_LineTest__EvalD0() {
    let myLine = setup();
    let aPnt: Point2 = myLine.eval_d0(5.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__EvalD1_DirectionIsConstant() {
    let myLine = setup();
    let aD1: Vector2 = myLine.eval_d1(5.0).unwrap().d1;
    EXPECT_NEAR(aD1.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aD1.y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__EvalD2_IsZero() {
    let myLine = setup();
    let aD2: Vector2 = myLine.eval_d2(5.0).unwrap().d2;
    EXPECT_NEAR(aD2.magnitude(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__EvalD3_IsZero() {
    let myLine = setup();
    let aD3: Vector2 = myLine.eval_d3(5.0).unwrap().d3;
    EXPECT_NEAR(aD3.magnitude(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__EvalDN_HigherOrder() {
    let myLine = setup();
    let aD4: Vector2 = myLine.eval_dn(0.0, 4).unwrap();
    EXPECT_NEAR(aD4.magnitude(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__Distance() {
    let myLine = setup();
    EXPECT_NEAR(
        myLine.distance(Point2::new(5.0, 3.0)),
        3.0,
        precision::confusion(),
    );
    EXPECT_NEAR(
        myLine.distance(Point2::new(0.0, 0.0)),
        0.0,
        precision::confusion(),
    );
}

#[test]
fn Geom2d_LineTest__SetDirection() {
    let myLine = setup();
    myLine.set_direction(dir(0.0, 1.0));
    EXPECT_NEAR(myLine.direction().x(), 0.0, precision::confusion());
    EXPECT_NEAR(myLine.direction().y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__SetLocation() {
    let myLine = setup();
    myLine.set_location(Point2::new(3.0, 4.0));
    EXPECT_NEAR(myLine.location().x(), 3.0, precision::confusion());
    EXPECT_NEAR(myLine.location().y(), 4.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__Lin2d() {
    let myLine = setup();
    let aLin: Line2 = myLine.lin2d();
    EXPECT_NEAR(aLin.direction().x(), 1.0, precision::confusion());
    EXPECT_NEAR(aLin.location().x(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__Copy() {
    let myLine = setup();
    let aCopyGeom: Handle<dyn Geom2dGeometry> = myLine.copy().unwrap();
    let aCopy: Handle<Geom2dLine> = aCopyGeom.cast::<Geom2dLine>();

    assert!(!aCopy.is_null());
    EXPECT_NEAR(aCopy.direction().x(), 1.0, precision::confusion());

    aCopy.set_direction(dir(0.0, 1.0));
    EXPECT_NEAR(myLine.direction().x(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__Transform_Translation() {
    let myLine = setup();
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(5.0, 10.0));
    myLine.transform(aTrsf).unwrap();

    EXPECT_NEAR(myLine.location().x(), 5.0, precision::confusion());
    EXPECT_NEAR(myLine.location().y(), 10.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__Transform_Rotation() {
    let myLine = setup();
    let mut aTrsf = Transform2::default();
    aTrsf.set_rotation(Point2::new(0.0, 0.0), PI / 2.0);
    myLine.transform(aTrsf).unwrap();

    // Horizontal line rotated 90 degrees becomes vertical
    EXPECT_NEAR(myLine.direction().x(), 0.0, precision::confusion());
    EXPECT_NEAR(myLine.direction().y().abs(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_LineTest__ReversedParameter() {
    let myLine = setup();
    EXPECT_DOUBLE_EQ(myLine.reversed_parameter(5.0).unwrap(), -5.0);
}

#[test]
fn Geom2d_LineTest__DiagonalLine_Value() {
    let aDiag = Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), dir(1.0, 1.0)));

    let aPnt: Point2 = aDiag.eval_d0(2.0_f64.sqrt()).unwrap();
    EXPECT_NEAR(aPnt.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 1.0, precision::confusion());
}
