//! Direct translation of Convert_CircleToBSplineCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkmath::convert::{ConicToBSplineCurve, ParameterisationType};
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Point2};
use std::f64::consts::PI;

// gp_Circ2d(gp_Ax2d(gp_Pnt2d(x, y), gp_Dir2d(dx, dy)), radius).
fn circle(x: f64, y: f64, dx: f64, dy: f64, radius: f64) -> Circle2 {
    let axis = Axis2d::new(Point2::new(x, y), Direction2::new(dx, dy).unwrap());
    Circle2::from_x_axis(axis, radius, true).unwrap()
}

#[test]
fn Convert_CircleToBSplineCurveTest__FullCircle_TgtThetaOver2() {
    let aCirc = circle(0.0, 0.0, 1.0, 0.0, 5.0);
    let aConv =
        ConicToBSplineCurve::from_circle(&aCirc, ParameterisationType::TgtThetaOver2).unwrap();

    assert_eq!(aConv.degree(), 2);
    assert!(aConv.is_periodic());
    assert!(aConv.nb_poles() > 0);
    assert!(aConv.nb_knots() > 0);

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
fn Convert_CircleToBSplineCurveTest__FullCircle_RationalC1() {
    let aCirc = circle(1.0, 2.0, 1.0, 0.0, 3.0);
    let aConv = ConicToBSplineCurve::from_circle(&aCirc, ParameterisationType::RationalC1).unwrap();

    assert!(aConv.is_periodic());
    assert!(aConv.nb_poles() > 0);

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
fn Convert_CircleToBSplineCurveTest__Arc_TgtThetaOver2() {
    let aCirc = circle(0.0, 0.0, 1.0, 0.0, 1.0);
    let aU1: f64 = PI / 6.0;
    let aU2: f64 = 5.0 * PI / 3.0;
    let aConv =
        ConicToBSplineCurve::from_circle_arc(&aCirc, aU1, aU2, ParameterisationType::TgtThetaOver2)
            .unwrap();

    assert!(!aConv.is_periodic());
    assert_eq!(aConv.degree(), 2);
    assert!(aConv.nb_poles() > 0);

    // Verify first pole matches conic point at U1
    let aPoles = aConv.poles();
    let aR: f64 = aCirc.radius();
    let aCenter = aCirc.location();
    let aXDir = aCirc.x_axis().direction();
    let aYDir = aCirc.y_axis().direction();

    let aFirstX: f64 = aCenter.x() + aR * (aU1.cos() * aXDir.x() + aU1.sin() * aYDir.x());
    let aFirstY: f64 = aCenter.y() + aR * (aU1.cos() * aXDir.y() + aU1.sin() * aYDir.y());
    assert!((aPoles[1].x() - aFirstX).abs() <= 1.0e-10);
    assert!((aPoles[1].y() - aFirstY).abs() <= 1.0e-10);

    // Verify last pole matches conic point at U2
    let aLastX: f64 = aCenter.x() + aR * (aU2.cos() * aXDir.x() + aU2.sin() * aYDir.x());
    let aLastY: f64 = aCenter.y() + aR * (aU2.cos() * aXDir.y() + aU2.sin() * aYDir.y());
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
fn Convert_CircleToBSplineCurveTest__Arc_QuasiAngular() {
    let aCirc = circle(0.0, 0.0, 1.0, 0.0, 2.0);
    let aU1: f64 = 0.0;
    let aU2: f64 = PI;
    let aConv =
        ConicToBSplineCurve::from_circle_arc(&aCirc, aU1, aU2, ParameterisationType::QuasiAngular)
            .unwrap();

    assert!(!aConv.is_periodic());
    assert_eq!(aConv.degree(), 6);
    assert!(aConv.nb_poles() > 0);

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
fn Convert_CircleToBSplineCurveTest__Arc_Polynomial() {
    let aCirc = circle(0.0, 0.0, 1.0, 0.0, 1.0);
    let aU1: f64 = 0.0;
    let aU2: f64 = PI * 0.5;
    let aConv =
        ConicToBSplineCurve::from_circle_arc(&aCirc, aU1, aU2, ParameterisationType::Polynomial)
            .unwrap();

    assert!(!aConv.is_periodic());
    assert_eq!(aConv.degree(), 7);
    assert_eq!(aConv.nb_poles(), 8);
}

#[test]
fn Convert_CircleToBSplineCurveTest__WeightsArePositive() {
    let aCirc = circle(0.0, 0.0, 1.0, 0.0, 1.0);
    let aConv =
        ConicToBSplineCurve::from_circle(&aCirc, ParameterisationType::TgtThetaOver2).unwrap();

    let aWeights = aConv.weights();
    for i in 1..=aConv.nb_poles() {
        assert!(aWeights[i] > 0.0);
    }
}
