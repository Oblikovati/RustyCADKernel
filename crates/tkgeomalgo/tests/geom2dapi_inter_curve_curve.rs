//! Direct translation of Geom2dAPI_InterCurveCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
// The expected coordinates keep the digits of the upstream test.
#![allow(clippy::excessive_precision)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCurve, Geom2dEllipse, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkmath::gp::{Axis2d, Axis22d, Direction2, Ellipse2, Point2};
use rustycad_tkmath::math::{NewtonFunctionRoot, TrigonometricEquation};

fn createOCC23706Curve1() -> Handle<Geom2dBSplineCurve> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 6).unwrap();
    aPoles[1] = Point2::new(2.0, 0.0);
    aPoles[2] = Point2::new(3.0, -1.0);
    aPoles[3] = Point2::new(5.0, 5.0);
    aPoles[4] = Point2::new(5.0, 5.0);
    aPoles[5] = Point2::new(6.0, 8.0);
    aPoles[6] = Point2::new(4.0, 7.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 5).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 0.2;
    aKnots[3] = 0.3;
    aKnots[4] = 0.4;
    aKnots[5] = 0.5;

    let mut aMultiplicities: Array1<'static, i32> = Array1::with_bounds(1, 5).unwrap();
    aMultiplicities[1] = 3;
    aMultiplicities[2] = 1;
    aMultiplicities[3] = 1;
    aMultiplicities[4] = 1;
    aMultiplicities[5] = 3;

    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 6).unwrap();
    for anIndex in aWeights.lower()..=aWeights.upper() {
        aWeights[anIndex] = 1.0;
    }
    Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMultiplicities, 2, false)
            .unwrap(),
    )
}

fn createOCC23706Curve2(thePerturbed: bool) -> Handle<Geom2dBSplineCurve> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 5).unwrap();
    aPoles[1] = Point2::new(6.0, 3.0);
    aPoles[2] = Point2::new(
        if thePerturbed { 5.001 } else { 5.0 },
        if thePerturbed { 5.01 } else { 5.0 },
    );
    aPoles[3] = aPoles[2];
    aPoles[4] = Point2::new(3.0, 9.0);
    aPoles[5] = Point2::new(2.0, 11.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    aKnots[1] = 1.0;
    aKnots[2] = 2.0;
    aKnots[3] = 3.0;
    aKnots[4] = 5.0;

    let mut aMultiplicities: Array1<'static, i32> = Array1::with_bounds(1, 4).unwrap();
    aMultiplicities[1] = 3;
    aMultiplicities[2] = 1;
    aMultiplicities[3] = 1;
    aMultiplicities[4] = 3;

    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 5).unwrap();
    for anIndex in aWeights.lower()..=aWeights.upper() {
        aWeights[anIndex] = 1.0;
    }
    Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMultiplicities, 2, false)
            .unwrap(),
    )
}

fn ellipse(
    center: Point2,
    direction: Direction2,
    major: f64,
    minor: f64,
) -> Handle<dyn Geom2dCurve> {
    let anEllipse =
        Ellipse2::from_x_axis(Axis2d::new(center, direction), major, minor, true).unwrap();
    Handle::new(Geom2dEllipse::from_elips2d(anEllipse)).cast()
}

#[test]
fn Geom2dAPI_InterCurveCurve_Test__OCC29289_EllipseIntersectionNewtonRoot() {
    // Create two ellipses
    let Ge1 = ellipse(Point2::new(0., 0.), Direction2::X, 2., 1.);
    let Ge2 = ellipse(
        Point2::new(0.5, 0.5),
        Direction2::new(1., 1.).unwrap(),
        2.,
        1.,
    );

    // Find intersection points
    let mut Intersector = InterCurveCurve::new();
    Intersector.init(&Ge1, &Ge2, 1.0e-7).unwrap();
    assert!(
        Intersector.nb_points() > 0,
        "Error: intersector found no points"
    );

    // Setup trigonometric equation: A*cos(x) + B*sin(x) + C*cos(2*x) + D*sin(2*x) + E
    let A = 1.875;
    let B = -0.75;
    let C = -0.5;
    let D = -0.25;
    let E = -0.25;
    let mut MyF = TrigonometricEquation::new(A, B, C, D, E);

    let Tol1 = 1.0e-15;
    let Eps = 1.5e-12;
    let Nit: [i32; 4] = [5, 6, 7, 6];

    // For each intersection point, verify Newton root finding
    let mut TetaPrev = 0.;
    for i in 1..=Intersector.nb_points() {
        let Teta = Intersector.intersector().point(i).unwrap().param_on_first();
        let X = Teta - 0.1 * (Teta - TetaPrev);
        TetaPrev = Teta;

        let Resol =
            NewtonFunctionRoot::with_iterations(&mut MyF, X, Tol1, Eps, Nit[(i - 1) as usize]);
        assert!(Resol.is_done(), "Error: Newton is not done for {Teta}");

        let TetaNewton = Resol.root().unwrap();
        assert!(
            (Teta - TetaNewton).abs() <= 1.0e-7,
            "Error: Newton root is wrong for {Teta}"
        );
    }
}

#[test]
fn Geom2dAPI_InterCurveCurve_Test__PointRejectsZeroIndex() {
    let aCurve1 = ellipse(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        2.0,
        1.0,
    );
    let aCurve2 = ellipse(
        Point2::new(0.5, 0.5),
        Direction2::new(1.0, 1.0).unwrap(),
        2.0,
        1.0,
    );

    let anIntersector = InterCurveCurve::between(&aCurve1, &aCurve2, 1.0e-7).unwrap();
    assert!(anIntersector.nb_points() > 0);

    // EXPECT_THROW(Standard_OutOfRange)
    assert_eq!(
        anIntersector.point(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

// Regression test for bug #11: Init with null handle must raise Standard_NullObject.
#[test]
fn Geom2dAPI_InterCurveCurve_Test__Init_NullHandle_RaisesException() {
    let aNullCurve: Handle<dyn Geom2dCurve> = Handle::default();
    let aValidCurve = ellipse(
        Point2::new(0., 0.),
        Direction2::new(1., 0.).unwrap(),
        2.,
        1.,
    );

    // Both null.
    assert_eq!(
        InterCurveCurve::between(&aNullCurve, &aNullCurve, 1.0e-7)
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );

    // First null.
    assert_eq!(
        InterCurveCurve::between(&aNullCurve, &aValidCurve, 1.0e-7)
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );

    // Second null.
    assert_eq!(
        InterCurveCurve::between(&aValidCurve, &aNullCurve, 1.0e-7)
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );

    // Single-curve Init with null.
    let mut anInter = InterCurveCurve::new();
    assert_eq!(
        anInter
            .init_self(&aNullCurve, 1.0e-7)
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
}

// Migrated from tests/lowalgos/2dinter/bug23706_4.  Preserve the rounded
// four-decimal coordinate checks used by the DRAW regression.
#[test]
fn Geom2dAPI_InterCurveCurve_Test__OCC23706_4_BSplineIntersectionCoordinates() {
    let aCurve1: Handle<dyn Geom2dCurve> = createOCC23706Curve1().cast();
    let aCurve2: Handle<dyn Geom2dCurve> = createOCC23706Curve2(true).cast();
    let anIntersector = InterCurveCurve::between(&aCurve1, &aCurve2, 0.001).unwrap();

    assert_eq!(anIntersector.nb_points(), 2);
    let aPoint1 = anIntersector.point(1).unwrap();
    let aPoint2 = anIntersector.point(2).unwrap();
    EXPECT_NEAR(aPoint1.x(), 5.0024, 1.0e-4);
    EXPECT_NEAR(aPoint1.y(), 5.0072, 1.0e-4);
    EXPECT_NEAR(aPoint2.x(), 4.0024, 1.0e-4);
    EXPECT_NEAR(aPoint2.y(), 7.0012, 1.0e-4);
}

// Migrated from tests/lowalgos/2dinter/bug23706_5.  The original script used
// integer truncation for the second point, so retain that intent with ranges.
#[test]
fn Geom2dAPI_InterCurveCurve_Test__OCC23706_5_BSplineIntersectionCoordinates() {
    let aCurve1: Handle<dyn Geom2dCurve> = createOCC23706Curve1().cast();
    let aCurve2: Handle<dyn Geom2dCurve> = createOCC23706Curve2(false).cast();
    let anIntersector = InterCurveCurve::between(&aCurve1, &aCurve2, 0.001).unwrap();

    assert_eq!(anIntersector.nb_points(), 2);
    let aPoint1 = anIntersector.point(1).unwrap();
    let aPoint2 = anIntersector.point(2).unwrap();
    EXPECT_NEAR(aPoint1.x(), 5.0, 1.0e-7);
    EXPECT_NEAR(aPoint1.y(), 5.0, 1.0e-7);
    assert!(aPoint2.x() >= 4.0);
    assert!(aPoint2.x() < 5.0);
    assert!(aPoint2.y() >= 7.0);
    assert!(aPoint2.y() < 8.0);
}

// Migrated from tests/lowalgos/2dinter/bug29162.  The DRAW command uses its
// default 1.e-3 intersection tolerance.
#[test]
fn Geom2dAPI_InterCurveCurve_Test__OCC29162_TrimmedEllipseLineIntersection() {
    let anEllipse: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dEllipse::new(
            Axis22d::new(
                Point2::new(-610.348096534595, -710.720096056787),
                Direction2::new(0.999999902285153, 0.000442074298181498).unwrap(),
                true,
            )
            .unwrap(),
            15.0066332711999,
            0.291884102212871,
        )
        .unwrap(),
    )
    .cast();
    let aTrimmedEllipse: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dTrimmedCurve::new(&anEllipse, 3.09462291909258, 9.37780822627216, true, true)
            .unwrap(),
    )
    .cast();

    let aLine: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(-625.34430362036, -680.713463264921),
        Direction2::new(-0.000252749178714602, -0.999999968058926).unwrap(),
    ))
    .cast();
    let aTrimmedLine: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&aLine, 0.0, 30.0132665523925, true, true).unwrap())
            .cast();

    let anIntersector = InterCurveCurve::between(&aTrimmedEllipse, &aTrimmedLine, 0.001).unwrap();
    assert_eq!(anIntersector.nb_points(), 1);
    let aPoint = anIntersector.point(1).unwrap();
    EXPECT_NEAR(aPoint.x(), -625.35188801291508, 1.0e-7);
    EXPECT_NEAR(aPoint.y(), -710.72104765746838, 1.0e-7);
}
