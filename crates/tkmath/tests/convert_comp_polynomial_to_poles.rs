//! Direct translation of Convert_CompPolynomialToPoles_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::convert::{CompPolynomialToPoles, PolynomialSpans};

// NCollection_Array1<double>(1, n) filled with the listed values.
fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut array: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

#[test]
fn Convert_CompPolynomialToPolesTest__SingleLinearPolynomial() {
    // Convert f(x) = 2*x + 1 on [-1,1] to BSpline
    // Polynomial domain [-1,1], True interval [-1,1]
    let aDim: i32 = 1;
    let aMaxDeg: i32 = 1;
    let aDeg: i32 = 1;

    let aCoeffs = reals(&[1.0, 2.0]); // constant term, linear term
    let aPolyIntervals = reals(&[-1.0, 1.0]);
    let aTrueIntervals = reals(&[-1.0, 1.0]);

    let aConv = CompPolynomialToPoles::from_single_span(
        aDim,
        aMaxDeg,
        aDeg,
        &aCoeffs,
        &aPolyIntervals,
        &aTrueIntervals,
    )
    .unwrap();

    assert!(aConv.is_done());
    assert_eq!(aConv.degree(), 1);
    assert_eq!(aConv.nb_knots(), 2);
    assert!(aConv.nb_poles() > 0);
}

#[test]
fn Convert_CompPolynomialToPolesTest__SingleQuadraticPolynomial() {
    // Convert f(x) = x^2 on [0,1] to BSpline
    let aDim: i32 = 1;
    let aMaxDeg: i32 = 2;
    let aDeg: i32 = 2;

    let aCoeffs = reals(&[0.0, 0.0, 1.0]); // x^0, x^1, x^2
    let aPolyIntervals = reals(&[0.0, 1.0]);
    let aTrueIntervals = reals(&[0.0, 1.0]);

    let aConv = CompPolynomialToPoles::from_single_span(
        aDim,
        aMaxDeg,
        aDeg,
        &aCoeffs,
        &aPolyIntervals,
        &aTrueIntervals,
    )
    .unwrap();

    assert!(aConv.is_done());
    assert_eq!(aConv.degree(), 2);
}

#[test]
fn Convert_CompPolynomialToPolesTest__TwoSpansUniformContinuity() {
    // Two linear polynomials with C0 continuity
    let aNumCurves: i32 = 2;
    let aContinuity: i32 = 0;
    let aDim: i32 = 1;
    let aMaxDeg: i32 = 1;

    // linear = 2 coefficients
    let mut aNumCoeff: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aNumCoeff[1] = 2;
    aNumCoeff[2] = 2;

    // Coefficients: [curve1: c0, c1] [curve2: c0, c1]
    let aCoeffs = reals(&[0.0, 1.0, 1.0, -1.0]);

    let mut aPolyIntervals: Array2<'static, f64> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aPolyIntervals[(1, 1)] = 0.0;
    aPolyIntervals[(1, 2)] = 1.0;
    aPolyIntervals[(2, 1)] = 0.0;
    aPolyIntervals[(2, 2)] = 1.0;

    let aTrueIntervals = reals(&[0.0, 0.5, 1.0]);

    let aSpans = PolynomialSpans {
        num_coeff_per_curve: &aNumCoeff,
        coefficients: &aCoeffs,
        polynomial_intervals: &aPolyIntervals,
        true_intervals: &aTrueIntervals,
    };
    let aConv = CompPolynomialToPoles::from_uniform_continuity(
        aNumCurves,
        aContinuity,
        aDim,
        aMaxDeg,
        Some(aSpans),
    )
    .unwrap();

    assert!(aConv.is_done());
    assert_eq!(aConv.degree(), 1);
    assert_eq!(aConv.nb_knots(), 3);
}

#[test]
fn Convert_CompPolynomialToPolesTest__ThreeDimensional() {
    // 3D linear curve
    let aDim: i32 = 3;
    let aMaxDeg: i32 = 1;
    let aDeg: i32 = 1;

    // Coefficients for 3D: [x0, y0, z0, x1, y1, z1]
    let aCoeffs = reals(&[0.0, 0.0, 0.0, 1.0, 2.0, 3.0]); // constant, linear

    let aPolyIntervals = reals(&[0.0, 1.0]);
    let aTrueIntervals = reals(&[0.0, 1.0]);

    let aConv = CompPolynomialToPoles::from_single_span(
        aDim,
        aMaxDeg,
        aDeg,
        &aCoeffs,
        &aPolyIntervals,
        &aTrueIntervals,
    )
    .unwrap();

    assert!(aConv.is_done());
    assert_eq!(aConv.degree(), 1);

    let aPoles = aConv.poles().unwrap();
    // Poles should be [1..NbPoles][1..3]
    assert_eq!(aPoles.nb_columns() as i32, 3);
}
