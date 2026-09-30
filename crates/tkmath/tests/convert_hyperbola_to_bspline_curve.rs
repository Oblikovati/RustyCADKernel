//! Direct translation of Convert_HyperbolaToBSplineCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkmath::convert::ConicToBSplineCurve;
use rustycad_tkmath::gp::{Axis2d, Direction2, Hyperbola2, Point2};

// gp_Hypr2d(gp_Ax2d(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)), major, minor).
fn hyperbola(major: f64, minor: f64) -> Hyperbola2 {
    let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    Hyperbola2::from_x_axis(axis, major, minor, true).unwrap()
}

// The pole and weight, knot and multiplicity checks shared verbatim by two cases.
fn check_ends_and_tables(aConv: &ConicToBSplineCurve, aHypr: &Hyperbola2, aU1: f64, aU2: f64) {
    // Verify first pole matches hyperbola point at U1
    let aPoles = aConv.poles();
    let aMajR: f64 = aHypr.major_radius();
    let aMinR: f64 = aHypr.minor_radius();
    let aCenter = aHypr.location();
    let aXDir = aHypr.x_axis().direction();
    let aYDir = aHypr.y_axis().direction();

    let aFirstX: f64 =
        aCenter.x() + aMajR * aU1.cosh() * aXDir.x() + aMinR * aU1.sinh() * aYDir.x();
    let aFirstY: f64 =
        aCenter.y() + aMajR * aU1.cosh() * aXDir.y() + aMinR * aU1.sinh() * aYDir.y();
    assert!((aPoles[1].x() - aFirstX).abs() <= 1.0e-10);
    assert!((aPoles[1].y() - aFirstY).abs() <= 1.0e-10);

    // Verify last pole matches hyperbola point at U2
    let aLastX: f64 = aCenter.x() + aMajR * aU2.cosh() * aXDir.x() + aMinR * aU2.sinh() * aYDir.x();
    let aLastY: f64 = aCenter.y() + aMajR * aU2.cosh() * aXDir.y() + aMinR * aU2.sinh() * aYDir.y();
    assert!((aPoles[aConv.nb_poles()].x() - aLastX).abs() <= 1.0e-10);
    assert!((aPoles[aConv.nb_poles()].y() - aLastY).abs() <= 1.0e-10);

    // Verify all weights are positive
    let aWeights = aConv.weights();
    for i in 1..=aConv.nb_poles() {
        assert!(aWeights[i] > 0.0, "Weight at index {i} is not positive");
    }

    // Verify knots are monotonically increasing
    let aKnots = aConv.knots();
    for i in 2..=aConv.nb_knots() {
        assert!(
            aKnots[i] > aKnots[i - 1],
            "Knots not monotonically increasing at index {i}"
        );
    }
}

#[test]
fn Convert_HyperbolaToBSplineCurveTest__BasicConversion() {
    let aHypr = hyperbola(3.0, 2.0);
    let aU1: f64 = -1.0;
    let aU2: f64 = 1.0;
    let aConv = ConicToBSplineCurve::from_hyperbola(&aHypr, aU1, aU2).unwrap();

    assert!(!aConv.is_periodic());
    assert_eq!(aConv.degree(), 2);
    assert!(aConv.nb_poles() > 0);
    assert!(aConv.nb_knots() > 0);

    check_ends_and_tables(&aConv, &aHypr, aU1, aU2);

    // Verify multiplicities are within valid range
    let aMults = aConv.multiplicities();
    for i in 1..=aConv.nb_knots() {
        assert!(aMults[i] > 0);
        assert!(aMults[i] <= aConv.degree() + 1);
    }
}

#[test]
fn Convert_HyperbolaToBSplineCurveTest__LargeRange() {
    let aHypr = hyperbola(1.0, 1.0);
    let aU1: f64 = -2.0;
    let aU2: f64 = 2.0;
    let aConv = ConicToBSplineCurve::from_hyperbola(&aHypr, aU1, aU2).unwrap();

    assert!(!aConv.is_periodic());
    assert!(aConv.nb_poles() > 0);
    assert!(aConv.nb_knots() > 0);

    check_ends_and_tables(&aConv, &aHypr, aU1, aU2);
}

#[test]
fn Convert_HyperbolaToBSplineCurveTest__WeightsArePositive() {
    let aHypr = hyperbola(3.0, 2.0);
    let aConv = ConicToBSplineCurve::from_hyperbola(&aHypr, -1.0, 1.0).unwrap();

    let aWeights = aConv.weights();
    for i in 1..=aConv.nb_poles() {
        assert!(aWeights[i] > 0.0);
    }
}
