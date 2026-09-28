// Tests for polynomial evaluation
#[test]
fn PLibTest__EvalPolynomial() {
    // Test simple polynomial evaluation: f(x) = 1 + 2x + 3x^2
    let aDegree: i32 = 2;
    let aDimension: i32 = 1;
    let aDerivOrder: i32 = 0;

    let mut aCoeffs = reals(1, 3);
    aCoeffs[1] = 1.0; // constant term
    aCoeffs[2] = 2.0; // linear term
    aCoeffs[3] = 3.0; // quadratic term

    let mut aResults = reals(1, 1);

    // Test at x = 0: f(0) = 1
    plib::eval_polynomial(
        0.0,
        aDerivOrder,
        aDegree,
        aDimension,
        aCoeffs.as_slice(),
        aResults.as_mut_slice(),
    )
    .unwrap();
    assert!((aResults[1] - 1.0).abs() <= precision::confusion());

    // Test at x = 1: f(1) = 1 + 2 + 3 = 6
    plib::eval_polynomial(
        1.0,
        aDerivOrder,
        aDegree,
        aDimension,
        aCoeffs.as_slice(),
        aResults.as_mut_slice(),
    )
    .unwrap();
    assert!((aResults[1] - 6.0).abs() <= precision::confusion());

    // Test at x = 2: f(2) = 1 + 4 + 12 = 17
    plib::eval_polynomial(
        2.0,
        aDerivOrder,
        aDegree,
        aDimension,
        aCoeffs.as_slice(),
        aResults.as_mut_slice(),
    )
    .unwrap();
    assert!((aResults[1] - 17.0).abs() <= precision::confusion());
}

// Tests for polynomial evaluation with derivatives
#[test]
fn PLibTest__EvalPolynomialWithDerivatives() {
    // Test polynomial f(x) = 1 + 2x + 3x^2
    // f'(x) = 2 + 6x
    // f''(x) = 6
    let aDegree: i32 = 2;
    let aDimension: i32 = 1;
    let aDerivOrder: i32 = 2;

    let mut aCoeffs = reals(1, 3);
    aCoeffs[1] = 1.0;
    aCoeffs[2] = 2.0;
    aCoeffs[3] = 3.0;

    let mut aResults = reals(1, 3); // value + 1st + 2nd derivative

    // Test at x = 1
    plib::eval_polynomial(
        1.0,
        aDerivOrder,
        aDegree,
        aDimension,
        aCoeffs.as_slice(),
        aResults.as_mut_slice(),
    )
    .unwrap();

    assert!((aResults[1] - 6.0).abs() <= precision::confusion()); // f(1) = 6
    assert!((aResults[2] - 8.0).abs() <= precision::confusion()); // f'(1) = 2 + 6 = 8
    assert!((aResults[3] - 6.0).abs() <= precision::confusion()); // f''(1) = 6
}

// Tests for constraint order conversion functions
#[test]
fn PLibTest__ConstraintOrderConversion() {
    // Test NivConstr function
    assert_eq!(plib::niv_constr(Shape::C0).unwrap(), 0);
    assert_eq!(plib::niv_constr(Shape::C1).unwrap(), 1);
    assert_eq!(plib::niv_constr(Shape::C2).unwrap(), 2);

    // Test ConstraintOrder function
    assert_eq!(plib::constraint_order(0).unwrap(), Shape::C0);
    assert_eq!(plib::constraint_order(1).unwrap(), Shape::C1);
    assert_eq!(plib::constraint_order(2).unwrap(), Shape::C2);

    // Test round-trip consistency
    for i in 0..=2 {
        let aShape: Shape = plib::constraint_order(i).unwrap();
        let aLevel: i32 = plib::niv_constr(aShape).unwrap();
        assert_eq!(aLevel, i, "Round-trip conversion failed for level {i}");
    }
}

// Test for Hermite interpolation
#[test]
fn PLibTest__HermiteInterpolate() {
    let aDimension: i32 = 1;
    let aFirstParam: f64 = 0.0;
    let aLastParam: f64 = 1.0;
    let aFirstOrder: i32 = 1; // value + 1st derivative
    let aLastOrder: i32 = 1; // value + 1st derivative

    // Define constraints: f(0) = 0, f'(0) = 1, f(1) = 1, f'(1) = 0
    let mut aFirstConstr: Array2<f64> = Array2::with_bounds(1, aDimension, 0, aFirstOrder).unwrap();
    aFirstConstr[(1, 0)] = 0.0; // f(0) = 0
    aFirstConstr[(1, 1)] = 1.0; // f'(0) = 1

    let mut aLastConstr: Array2<f64> = Array2::with_bounds(1, aDimension, 0, aLastOrder).unwrap();
    aLastConstr[(1, 0)] = 1.0; // f(1) = 1
    aLastConstr[(1, 1)] = 0.0; // f'(1) = 0

    let aCoeffCount: i32 = aFirstOrder + aLastOrder + 2;

    let mut aCoeffs = reals(0, aCoeffCount - 1); // 0-based indexing as expected by HermiteInterpolate

    // Perform Hermite interpolation
    let aResult: bool = plib::hermite_interpolate(
        aDimension,
        aFirstParam,
        aLastParam,
        aFirstOrder,
        aLastOrder,
        &aFirstConstr,
        &aLastConstr,
        &mut aCoeffs,
    )
    .unwrap();

    assert!(aResult, "Hermite interpolation failed");

    if aResult {
        // Verify that the resulting polynomial satisfies the constraints
        let mut aResults = reals(1, 2); // value + 1st derivative

        // Check at first parameter
        plib::eval_polynomial(
            aFirstParam,
            1,
            aCoeffCount - 1,
            aDimension,
            aCoeffs.as_slice(),
            aResults.as_mut_slice(),
        )
        .unwrap();
        assert!(
            (aResults[1] - 0.0).abs() <= precision::confusion(),
            "f(0) constraint not satisfied"
        );
        assert!(
            (aResults[2] - 1.0).abs() <= precision::confusion(),
            "f'(0) constraint not satisfied"
        );

        // Check at last parameter
        plib::eval_polynomial(
            aLastParam,
            1,
            aCoeffCount - 1,
            aDimension,
            aCoeffs.as_slice(),
            aResults.as_mut_slice(),
        )
        .unwrap();
        assert!(
            (aResults[1] - 1.0).abs() <= precision::confusion(),
            "f(1) constraint not satisfied"
        );
        assert!(
            (aResults[2] - 0.0).abs() <= precision::confusion(),
            "f'(1) constraint not satisfied"
        );
    }
}

// Edge case tests
#[test]
fn PLibTest__EdgeCases() {
    // Test with very small coefficients
    let mut aSmallCoeffs = reals(1, 3);
    aSmallCoeffs[1] = 1.0e-12;
    aSmallCoeffs[2] = 1.0e-12;
    aSmallCoeffs[3] = 1.0e-12;

    let mut aResults = reals(1, 1);

    // This should not crash even with very small coefficients
    assert!(
        plib::eval_polynomial(
            1.0,
            0,
            2,
            1,
            aSmallCoeffs.as_slice(),
            aResults.as_mut_slice()
        )
        .is_ok()
    );

    // Test with large coefficients
    let mut aLargeCoeffs = reals(1, 3);
    aLargeCoeffs[1] = 1.0e10;
    aLargeCoeffs[2] = 1.0e10;
    aLargeCoeffs[3] = 1.0e10;

    assert!(
        plib::eval_polynomial(
            1.0,
            0,
            2,
            1,
            aLargeCoeffs.as_slice(),
            aResults.as_mut_slice()
        )
        .is_ok()
    );
}

// Test for Jacobi parameter calculation
#[test]
fn PLibTest__JacobiParameters() {
    let mut aNbGaussPoints: i32 = 0;
    let mut aWorkDegree: i32 = 0;

    // Test various constraint orders and codes
    plib::jacobi_parameters(Shape::C0, 10, 1, &mut aNbGaussPoints, &mut aWorkDegree).unwrap();
    assert!(
        aNbGaussPoints > 0,
        "Number of Gauss points should be positive"
    );
    assert!(aWorkDegree > 0, "Work degree should be positive");

    plib::jacobi_parameters(Shape::C1, 15, 2, &mut aNbGaussPoints, &mut aWorkDegree).unwrap();
    assert!(aNbGaussPoints > 0);
    assert!(aWorkDegree > 0);

    plib::jacobi_parameters(Shape::C2, 20, 3, &mut aNbGaussPoints, &mut aWorkDegree).unwrap();
    assert!(aNbGaussPoints > 0);
    assert!(aWorkDegree > 0);
}
