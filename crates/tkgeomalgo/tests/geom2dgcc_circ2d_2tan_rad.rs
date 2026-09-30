//! Direct translation of Geom2dGcc_Circ2d2TanRad_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
// The expected lengths are copied digit for digit from the upstream suite.
#![allow(non_snake_case, clippy::excessive_precision)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCartesianPoint, Geom2dCircle, Geom2dCurve,
    Geom2dLine, Geom2dPoint,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::gcc_ent::Position;
use rustycad_tkgeomalgo::geom2d_gcc::{
    Circ2d2TanRad as Geom2dGcc_Circ2d2TanRad, Circ2dTanCen as Geom2dGcc_Circ2dTanCen,
    QualifiedCurve, unqualified,
};
use rustycad_tkgeombase::gcpnts::AbscissaPoint as GCPnts_AbscissaPoint;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Point2, StandardDirection2};

fn circleLength(theCircle: Circle2) -> f64 {
    let aCurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_circ2d(theCircle)).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    GCPnts_AbscissaPoint::length(&anAdaptor).unwrap()
}

fn cartesianPoint(theX: f64, theY: f64) -> Handle<dyn Geom2dPoint> {
    Handle::new(Geom2dCartesianPoint::new(theX, theY)).cast()
}

fn line(theP: Point2, theD: Direction2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(theP, theD)).cast()
}

fn points(theValues: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut anArray = Array1::<Point2>::with_bounds(1, theValues.len() as i32).unwrap();
    for (anIndex, &(aX, aY)) in (1..).zip(theValues) {
        anArray[anIndex] = Point2::new(aX, aY);
    }
    anArray
}

// Test BUC60897: Geom2dGcc_Circ2d2TanRad finds circles tangent to a line and
// a Bezier curve. Verifies that each tangency point lies at the circle radius
// from the circle center (within 1% relative tolerance).
#[test]
fn Geom2dGcc_Circ2d2TanRadTest__BUC60897_TangentToLineAndBezier() {
    // Create a Geom2d_Line from point (100, 0) going in the -X direction
    let aLine = line(
        Point2::new(100.0, 0.0),
        Direction2::from(StandardDirection2::NegX),
    );

    // Create a Geom2d_BezierCurve from three control points
    let aPoints = points(&[(0.0, 0.0), (50.0, 50.0), (0.0, 100.0)]);
    let aCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&aPoints).unwrap()).cast();

    // Build qualified curves (outside tangency)
    let aCLine = Geom2dAdaptorCurve::with_curve(&aLine).unwrap();
    let aCCurve = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let aQualifCurve1 = QualifiedCurve::new(&aCLine, Position::Outside).unwrap();
    let aQualifCurve2 = QualifiedCurve::new(&aCCurve, Position::Outside).unwrap();

    // Find circles with radius 10 tangent to both curves
    let aRadius: f64 = 10.0;
    let aTolerance: f64 = 1e-7;
    let aGccCirc2d =
        Geom2dGcc_Circ2d2TanRad::new(&aQualifCurve1, &aQualifCurve2, aRadius, aTolerance).unwrap();

    assert!(
        aGccCirc2d.is_done(),
        "Geom2dGcc_Circ2d2TanRad failed to compute"
    );
    assert!(aGccCirc2d.nb_solutions() > 0, "No tangent circles found");

    // For each solution, verify tangency points are at the circle radius from its center
    let aMaxDeltaPercent: f64 = 1.0; // 1% tolerance as used in the original Draw test
    for i in 1..=aGccCirc2d.nb_solutions() {
        let aCirc2d: Circle2 = aGccCirc2d.this_solution(i).unwrap();
        let aCenter: Point2 = aCirc2d.location();
        let aR: f64 = aCirc2d.radius();

        let (_aParSol1, _aParArg1, aPntSol1) = aGccCirc2d.tangency1(i).unwrap();
        let (_aParSol2, _aParArg2, aPntSol2) = aGccCirc2d.tangency2(i).unwrap();

        // Distance from tangency point 1 to circle center must equal radius within 1%
        let aD1: f64 = aPntSol1.distance(aCenter);
        let aDelta1: f64 = (aD1 - aR).abs() / aR * 100.0;
        assert!(
            aDelta1 <= aMaxDeltaPercent,
            "Solution {i}: tangency1 distance error {aDelta1}% exceeds 1%"
        );

        // Distance from tangency point 2 to circle center must equal radius within 1%
        let aD2: f64 = aPntSol2.distance(aCenter);
        let aDelta2: f64 = (aD2 - aR).abs() / aR * 100.0;
        assert!(
            aDelta2 <= aMaxDeltaPercent,
            "Solution {i}: tangency2 distance error {aDelta2}% exceeds 1%"
        );
    }
}

// Migrated from tests/lowalgos/2dgcc/buc60607_1.  Keep the two solution
// lengths produced by a point/line fixed-radius construction.
#[test]
fn Geom2dGcc_Circ2d2TanRadTest__BUC60607_1_PointLineFixedRadius() {
    let aLine = line(
        Point2::new(0.0, 500.0),
        Direction2::new(2000.0, 500.0).unwrap(),
    );
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aLine).unwrap();
    let aQualifiedLine = unqualified(&anAdaptor).unwrap();

    let aSolver = Geom2dGcc_Circ2d2TanRad::with_point(
        &aQualifiedLine,
        &cartesianPoint(1600.0, 300.0),
        400.0,
        precision::confusion(),
    )
    .unwrap();
    assert!(aSolver.is_done());
    assert_eq!(aSolver.nb_solutions(), 2);

    for anIndex in 1..=aSolver.nb_solutions() {
        let aLength: f64 = circleLength(aSolver.this_solution(anIndex).unwrap());
        assert!((aLength - 2513.2741228718346).abs() <= 2513.2741228718346 * 1.0e-8);
    }
}

// Migrated from tests/lowalgos/2dgcc/buc60607_2.  The DRAW case did not
// print a result, but the construction is a valid no-solution configuration.
#[test]
fn Geom2dGcc_Circ2d2TanRadTest__BUC60607_2_PointLineFixedRadiusNoSolution() {
    let aLine = line(
        Point2::new(0.0, 500.0),
        Direction2::new(2000.0, 500.0).unwrap(),
    );
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aLine).unwrap();
    let aQualifiedLine = unqualified(&anAdaptor).unwrap();

    let aSolver = Geom2dGcc_Circ2d2TanRad::with_point(
        &aQualifiedLine,
        &cartesianPoint(1600.0, 300.0),
        200.0,
        precision::confusion(),
    )
    .unwrap();
    assert!(aSolver.is_done());
    assert_eq!(aSolver.nb_solutions(), 0);
}

// Migrated from tests/lowalgos/2dgcc/buc60607_3.  Preserve the exact DRAW
// line definition (the last two values are its direction, not an endpoint).
#[test]
fn Geom2dGcc_Circ2d2TanRadTest__BUC60607_3_PointLineFixedRadiusNoSolution() {
    let aLine = line(
        Point2::new(3000.0, 1700.0),
        Direction2::new(1000.0, 1550.0).unwrap(),
    );
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aLine).unwrap();
    let aQualifiedLine = unqualified(&anAdaptor).unwrap();

    let aSolver = Geom2dGcc_Circ2d2TanRad::with_point(
        &aQualifiedLine,
        &cartesianPoint(1400.0, 1400.0),
        300.0,
        precision::confusion(),
    )
    .unwrap();
    assert!(aSolver.is_done());
    assert_eq!(aSolver.nb_solutions(), 0);
}

// Migrated from tests/lowalgos/2dgcc/buc60607_4.  This is the same geometry
// as BUC60607_3 with the smaller requested radius.
#[test]
fn Geom2dGcc_Circ2d2TanRadTest__BUC60607_4_PointLineFixedRadiusNoSolution() {
    let aLine = line(
        Point2::new(3000.0, 1700.0),
        Direction2::new(1000.0, 1550.0).unwrap(),
    );
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aLine).unwrap();
    let aQualifiedLine = unqualified(&anAdaptor).unwrap();

    let aSolver = Geom2dGcc_Circ2d2TanRad::with_point(
        &aQualifiedLine,
        &cartesianPoint(1400.0, 1400.0),
        150.0,
        precision::confusion(),
    )
    .unwrap();
    assert!(aSolver.is_done());
    assert_eq!(aSolver.nb_solutions(), 0);
}

// Migrated from tests/lowalgos/2dgcc/buc60941.  The DRAW regression is about
// the two-curve constructor not raising while processing a BSpline curve.
#[test]
fn Geom2dGcc_Circ2d2TanRadTest__BUC60941_TwoCurveFixedRadiusDoesNotThrow() {
    let aCircle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_circ2d(
        Circle2::from_x_axis(
            Axis2d::new(
                Point2::new(1.79356127430613, -11.9148900515605),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            12.0491272292007,
            true,
        )
        .unwrap(),
    ))
    .cast();

    let aPoles = points(&[
        (0.0, 0.0),
        (-2.14533906935467, 4.92056269221697),
        (-0.537398638327716, 10.0937503357781),
        (10.3564232507092, 8.73929826869582),
        (16.6366922939454, 3.11858911810943),
        (16.39, -0.262),
    ]);

    let mut aKnots = Array1::<f64>::with_bounds(1, 4).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 8.6971225126475;
    aKnots[3] = 16.903955884905;
    aKnots[4] = 28.0087167558108;

    let mut aMultiplicities = Array1::<i32>::with_bounds(1, 4).unwrap();
    aMultiplicities[1] = 4;
    aMultiplicities[2] = 1;
    aMultiplicities[3] = 1;
    aMultiplicities[4] = 4;

    let mut aWeights = Array1::<f64>::with_bounds(1, 6).unwrap();
    for anIndex in aWeights.lower()..=aWeights.upper() {
        aWeights[anIndex] = 1.0;
    }
    let aBSpline: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMultiplicities, 3, false)
            .unwrap(),
    )
    .cast();

    let anAdaptor1 = Geom2dAdaptorCurve::with_curve(&aCircle).unwrap();
    let anAdaptor2 = Geom2dAdaptorCurve::with_curve(&aBSpline).unwrap();
    let aQualifiedCircle = unqualified(&anAdaptor1).unwrap();
    let aQualifiedBSpline = unqualified(&anAdaptor2).unwrap();

    let mut anIsDone = false;
    let mut aNbSolutions: i32 = -1;
    // EXPECT_NO_THROW: the constructor returns Ok.
    let aSolver = Geom2dGcc_Circ2d2TanRad::new(
        &aQualifiedCircle,
        &aQualifiedBSpline,
        2.0,
        precision::confusion(),
    );
    assert!(aSolver.is_ok(), "{:?}", aSolver.as_ref().err());
    if let Ok(aSolver) = aSolver {
        anIsDone = aSolver.is_done();
        if anIsDone {
            aNbSolutions = aSolver.nb_solutions();
        }
    }
    assert!(anIsDone);
    assert!(aNbSolutions >= 0);
}

// Migrated from tests/lowalgos/2dgcc/bug29694.  This keeps the center and
// radius checks from the DRAW script while using the algorithm API directly.
#[test]
fn Geom2dGcc_Circ2d2TanRadTest__Bug29694_CurvePointCenter() {
    let aPoles = points(&[(-10.0, -5.0), (10.0, -5.0)]);

    let mut aKnots = Array1::<f64>::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMultiplicities = Array1::<i32>::with_bounds(1, 2).unwrap();
    aMultiplicities[1] = 2;
    aMultiplicities[2] = 2;
    let aSegment: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMultiplicities, 1, false).unwrap())
            .cast();

    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aSegment).unwrap();
    let aQualifiedSegment = unqualified(&anAdaptor).unwrap();
    let aSolver = Geom2dGcc_Circ2dTanCen::new(
        &aQualifiedSegment,
        &cartesianPoint(1.0, 1.0),
        precision::confusion(),
    )
    .unwrap();

    assert!(aSolver.is_done());
    assert_eq!(aSolver.nb_solutions(), 1);
    let aSolution: Circle2 = aSolver.this_solution(1).unwrap();
    assert!((aSolution.location().x() - 1.0).abs() <= 1.0e-7);
    assert!((aSolution.location().y() - 1.0).abs() <= 1.0e-7);
    assert!((aSolution.radius() - 6.0).abs() <= 1.0e-7);
}
