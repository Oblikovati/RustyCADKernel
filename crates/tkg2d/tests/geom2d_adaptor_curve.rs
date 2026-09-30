//! Direct translation of Geom2dAdaptor_Curve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Line2, Point2};
use std::f64::consts::PI;

// Geom2dAdaptor_Curve_Test::SetUp: a diagonal line and a circle of radius 3.
struct Fixture {
    myLine: Handle<dyn Geom2dCurve>,
    myCircle: Handle<dyn Geom2dCurve>,
}

fn setup() -> Fixture {
    // Create a simple 2D line for testing
    let aP1 = Point2::new(0.0, 0.0);
    let aLine = Line2::new(aP1, Direction2::new(1.0, 1.0).unwrap());
    let myLine = Handle::new(Geom2dLine::from_lin2d(aLine)).cast();

    // Create a 2D circle for testing
    let aCenter = Point2::new(5.0, 5.0);
    let aAxis = Axis2d::new(aCenter, Direction2::new(1.0, 0.0).unwrap());
    let aCirc = Circle2::from_x_axis(aAxis, 3.0, true).unwrap();
    let myCircle = Handle::new(Geom2dCircle::from_circ2d(aCirc)).cast();
    Fixture { myLine, myCircle }
}

fn exception(result: Result<(), Failure>) -> &'static str {
    result.unwrap_err().exception_type()
}

#[test]
fn Geom2dAdaptor_Curve_Test__Load_ValidParameters_Success() {
    let f = setup();
    // Test loading with valid parameters
    let anAdaptor = Geom2dAdaptorCurve::default();
    assert!(anAdaptor.load_range(&f.myLine, 0.0, 10.0).is_ok());

    EXPECT_DOUBLE_EQ(anAdaptor.first_parameter().unwrap(), 0.0);
    EXPECT_DOUBLE_EQ(anAdaptor.last_parameter().unwrap(), 10.0);
}

#[test]
fn Geom2dAdaptor_Curve_Test__Load_EqualParameters_Success() {
    let f = setup();
    // Test loading with equal parameters (degenerated curve)
    // This should be allowed as it represents a point
    let anAdaptor = Geom2dAdaptorCurve::default();
    assert!(anAdaptor.load_range(&f.myLine, 5.0, 5.0).is_ok());

    EXPECT_DOUBLE_EQ(anAdaptor.first_parameter().unwrap(), 5.0);
    EXPECT_DOUBLE_EQ(anAdaptor.last_parameter().unwrap(), 5.0);

    // Verify it evaluates to a single point
    let aP1 = anAdaptor.value(5.0).unwrap();
    let aP2 = f.myLine.value(5.0).unwrap();
    assert!(aP1.is_equal(aP2, precision::confusion()));
}

#[test]
fn Geom2dAdaptor_Curve_Test__Load_ParametersWithinConfusion_Success() {
    let f = setup();
    // Test loading with parameters within Precision::Confusion()
    // This should be allowed (degenerated curve handling)
    let aParam1: f64 = 5.0;
    let aParam2: f64 = 5.0 + precision::confusion() * 0.5;

    let anAdaptor = Geom2dAdaptorCurve::default();
    assert!(anAdaptor.load_range(&f.myLine, aParam1, aParam2).is_ok());

    EXPECT_DOUBLE_EQ(anAdaptor.first_parameter().unwrap(), aParam1);
    EXPECT_DOUBLE_EQ(anAdaptor.last_parameter().unwrap(), aParam2);
}

#[test]
fn Geom2dAdaptor_Curve_Test__Load_ParametersAtConfusionBoundary_Success() {
    let f = setup();
    // Test loading with parameters exactly at the confusion tolerance boundary
    let aParam1: f64 = 5.0;
    let aParam2: f64 = 5.0 + precision::confusion();

    let anAdaptor = Geom2dAdaptorCurve::default();
    assert!(anAdaptor.load_range(&f.myLine, aParam1, aParam2).is_ok());

    EXPECT_DOUBLE_EQ(anAdaptor.first_parameter().unwrap(), aParam1);
    EXPECT_DOUBLE_EQ(anAdaptor.last_parameter().unwrap(), aParam2);
}

#[test]
fn Geom2dAdaptor_Curve_Test__Load_FirstGreaterThanLastWithinConfusion_Success() {
    let f = setup();
    // Test loading with theUFirst > theULast but within Precision::Confusion()
    // This represents a degenerated curve and should be allowed
    let aParam1: f64 = 5.0 + precision::confusion() * 0.5;
    let aParam2: f64 = 5.0;

    let anAdaptor = Geom2dAdaptorCurve::default();
    assert!(anAdaptor.load_range(&f.myLine, aParam1, aParam2).is_ok());

    EXPECT_DOUBLE_EQ(anAdaptor.first_parameter().unwrap(), aParam1);
    EXPECT_DOUBLE_EQ(anAdaptor.last_parameter().unwrap(), aParam2);
}

#[test]
fn Geom2dAdaptor_Curve_Test__Load_FirstGreaterThanLastBeyondConfusion_ThrowsException() {
    let f = setup();
    // Test loading with UFirst > ULast + Precision::Confusion()
    // This should throw Standard_ConstructionError
    let aParam1: f64 = 10.0;
    let aParam2: f64 = 5.0;

    let anAdaptor = Geom2dAdaptorCurve::default();
    assert_eq!(
        exception(anAdaptor.load_range(&f.myLine, aParam1, aParam2)),
        "Standard_ConstructionError"
    );
}

#[test]
fn Geom2dAdaptor_Curve_Test__Load_FirstSlightlyGreaterThanLast_ThrowsException() {
    let f = setup();
    // Test loading with UFirst slightly greater than ULast beyond tolerance
    let aParam1: f64 = 5.0;
    let aParam2: f64 = 5.0 - precision::confusion() * 2.0;

    let anAdaptor = Geom2dAdaptorCurve::default();
    assert_eq!(
        exception(anAdaptor.load_range(&f.myLine, aParam1, aParam2)),
        "Standard_ConstructionError"
    );
}

#[test]
fn Geom2dAdaptor_Curve_Test__Constructor_DegeneratedCurve_Success() {
    let f = setup();
    // Test constructor with degenerated curve parameters
    assert!(Geom2dAdaptorCurve::with_range(&f.myCircle, 0.0, 0.0).is_ok());

    let anAdaptor = Geom2dAdaptorCurve::with_range(&f.myCircle, 0.0, 0.0).unwrap();
    EXPECT_DOUBLE_EQ(anAdaptor.first_parameter().unwrap(), 0.0);
    EXPECT_DOUBLE_EQ(anAdaptor.last_parameter().unwrap(), 0.0);
}

#[test]
fn Geom2dAdaptor_Curve_Test__Constructor_InvalidParameters_ThrowsException() {
    let f = setup();
    // Test constructor with invalid parameters
    let aResult = Geom2dAdaptorCurve::with_range(&f.myCircle, 10.0, 0.0).map(|_| ());
    assert_eq!(exception(aResult), "Standard_ConstructionError");
}

#[test]
fn Geom2dAdaptor_Curve_Test__Load_NullCurve_ThrowsException() {
    let _fixture = setup();
    // Test loading with null curve
    let aNullCurve: Handle<dyn Geom2dCurve> = Handle::default();
    let anAdaptor = Geom2dAdaptorCurve::default();

    assert_eq!(
        exception(anAdaptor.load_range(&aNullCurve, 0.0, 10.0)),
        "Standard_NullObject"
    );
}

#[test]
fn Geom2dAdaptor_Curve_Test__DegeneratedCurve_CircleAtZeroLength_Success() {
    let f = setup();
    // Test degenerated circle (zero length arc)
    let aParam: f64 = PI;

    let anAdaptor = Geom2dAdaptorCurve::default();
    assert!(anAdaptor.load_range(&f.myCircle, aParam, aParam).is_ok());

    // Verify the curve represents a single point
    let aPoint = anAdaptor.value(aParam).unwrap();
    let aExpectedPoint = f.myCircle.value(aParam).unwrap();

    assert!(aPoint.is_equal(aExpectedPoint, precision::confusion()));
    assert!(
        anAdaptor.is_closed().unwrap()
            || anAdaptor.first_parameter().unwrap() == anAdaptor.last_parameter().unwrap()
    );
}

#[test]
fn Geom2dAdaptor_Curve_Test__DegeneratedCurve_TrimmedCurve_Success() {
    let f = setup();
    // Create a trimmed curve and test degenerated case
    let aTrimmedCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&f.myLine, 0.0, 20.0, true, true).unwrap()).cast();

    let aParam: f64 = 10.0;
    let anAdaptor = Geom2dAdaptorCurve::default();
    assert!(anAdaptor.load_range(&aTrimmedCurve, aParam, aParam).is_ok());

    EXPECT_DOUBLE_EQ(anAdaptor.first_parameter().unwrap(), aParam);
    EXPECT_DOUBLE_EQ(anAdaptor.last_parameter().unwrap(), aParam);
}

#[test]
fn Geom2dAdaptor_Curve_Test__ToleranceBoundary_NegativeCase_ThrowsException() {
    let f = setup();
    // Test parameters just beyond the negative tolerance boundary
    let aParam1: f64 = 5.0;
    let aParam2: f64 = 5.0 - precision::confusion() - 1e-10;

    let anAdaptor = Geom2dAdaptorCurve::default();
    assert_eq!(
        exception(anAdaptor.load_range(&f.myLine, aParam1, aParam2)),
        "Standard_ConstructionError"
    );
}

#[test]
fn Geom2dAdaptor_Curve_Test__LoadWithoutParameters_Success() {
    let f = setup();
    // Test loading curve without specifying parameters (uses curve's own parameters)
    let anAdaptor = Geom2dAdaptorCurve::default();
    assert!(anAdaptor.load(&f.myCircle).is_ok());

    EXPECT_NEAR(
        anAdaptor.first_parameter().unwrap(),
        f.myCircle.first_parameter(),
        precision::confusion(),
    );
    EXPECT_NEAR(
        anAdaptor.last_parameter().unwrap(),
        f.myCircle.last_parameter(),
        precision::confusion(),
    );
    assert!(anAdaptor.is_periodic().unwrap());
}
