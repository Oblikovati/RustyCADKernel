/// Shared body of the pure-imaginary tests: roots are +/- i*`magnitude`.
fn check_pure_imaginary_pair(aCoeffs: &[f64], magnitude: f64) {
    let aResult: PolyResult = solve(aCoeffs, 2);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 0);
    assert_eq!(aResult.nb_complex_roots, 2);
    assert_eq!(aResult.complex_multiplicities[0], 1);
    assert_eq!(aResult.complex_multiplicities[1], 1);
    assert_eq!(aResult.complex_roots[1], aResult.complex_roots[0].conj());
    for anIndex in 0..aResult.nb_complex_roots {
        let aRoot: Complex = aResult.complex_roots[anIndex];
        near(aRoot.real(), 0.0, THE_TOL);
        near(aRoot.imag().abs(), magnitude, THE_TOL);
        assert!(
            EvaluateComplexPoly(aCoeffs, 2, aRoot).abs()
                <= ComplexEvaluationRoundoff(aCoeffs, 2, aRoot)
        );
    }
}

#[test]
fn MathPoly_LaguerreTest__SmallPureImaginaryRootsRemainComplex() {
    const THE_CONSTANT: f64 = 1.0e-14;
    let aCoeffs: [f64; 3] = [THE_CONSTANT, 0.0, 1.0];
    check_pure_imaginary_pair(&aCoeffs, THE_CONSTANT.sqrt());
}

#[test]
fn MathPoly_LaguerreTest__UnitPureImaginaryRootsRemainComplex() {
    let aCoeffs: [f64; 3] = [1.0, 0.0, 1.0];
    check_pure_imaginary_pair(&aCoeffs, 1.0);
}

#[test]
fn MathPoly_LaguerreTest__RepeatedComplexConjugatesReportMultiplicity() {
    let aCoeffs: [f64; 5] = [1.0, 0.0, 2.0, 0.0, 1.0];
    let aResult: PolyResult = solve(&aCoeffs, 4);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 0);
    assert_eq!(aResult.nb_complex_roots, 2);
    assert_eq!(aResult.complex_multiplicities[0], 2);
    assert_eq!(aResult.complex_multiplicities[1], 2);
    assert_eq!(aResult.complex_roots[1], aResult.complex_roots[0].conj());
    for anIndex in 0..aResult.nb_complex_roots {
        let aRoot: Complex = aResult.complex_roots[anIndex];
        near(aRoot.real(), 0.0, THE_TOL);
        near(aRoot.imag().abs(), 1.0, 1.0e-8);
        assert!(
            EvaluateComplexPoly(&aCoeffs, 4, aRoot).abs()
                <= ComplexEvaluationRoundoff(&aCoeffs, 4, aRoot)
        );
    }
}

#[test]
fn MathPoly_LaguerreTest__RepeatedRealAndComplexRootsUseMatchingDeflation() {
    // (x-1)^2 * (x^2+1)
    let aCoefficients: [f64; 5] = [1.0, -2.0, 2.0, -2.0, 1.0];
    let aResult: PolyResult = solve(&aCoefficients, 4);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], 1.0, 1.0e-8);
    assert_eq!(aResult.multiplicities[0], 2);
    assert_eq!(aResult.nb_complex_roots, 2);
    near(aResult.complex_roots[0].real(), 0.0, 1.0e-8);
    near(aResult.complex_roots[0].imag().abs(), 1.0, 1.0e-8);
    assert_eq!(aResult.complex_multiplicities[0], 1);
    assert_eq!(aResult.complex_multiplicities[1], 1);
}

#[test]
fn MathPoly_LaguerreTest__NearbyComplexPairRemainsDistinctFromRepeatedRealRoot() {
    // (x-1)^2 * ((x-1)^2 + 0.01^2)
    const THE_IMAGINARY_PART: f64 = 0.01;
    const THE_SQUARE: f64 = THE_IMAGINARY_PART * THE_IMAGINARY_PART;
    let aCoefficients: [f64; 5] = [
        1.0 + THE_SQUARE,
        -4.0 - 2.0 * THE_SQUARE,
        6.0 + THE_SQUARE,
        -4.0,
        1.0,
    ];
    let aResult: PolyResult = laguerre(&aCoefficients, 4, 1.0e-11, LAGUERRE_MAX_ITERATIONS);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], 1.0, 1.0e-7);
    assert_eq!(aResult.multiplicities[0], 2);
    assert_eq!(aResult.nb_complex_roots, 2);
    near(aResult.complex_roots[0].real(), 1.0, 1.0e-7);
    near(aResult.complex_roots[0].imag().abs(), THE_IMAGINARY_PART, 1.0e-7);
    assert_eq!(aResult.complex_multiplicities[0], 1);
    assert_eq!(aResult.complex_multiplicities[1], 1);
}

#[test]
fn MathPoly_LaguerreTest__GloballyScaledEquation() {
    let aCoeffs: [f64; 6] = [-1.2e-298, 2.74e-298, -2.25e-298, 8.5e-299, -1.5e-299, 1.0e-300];
    let aResult: PolyResult = solve(&aCoeffs, 5);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 5);
    for anIndex in 0..aResult.nb_roots {
        near(aResult.roots[anIndex], (anIndex + 1) as f64, 1.0e-8);
    }
}

#[test]
fn MathPoly_LaguerreTest__RejectsNonFiniteInputAndTolerance() {
    let aNan: f64 = f64::NAN;
    let aCoeffs: [f64; 3] = [1.0, aNan, 1.0];
    assert_eq!(solve(&aCoeffs, 2).status, Status::InvalidInput);
    let aFiniteCoeffs: [f64; 3] = [-1.0, 0.0, 1.0];
    assert_eq!(
        laguerre(&aFiniteCoeffs, 2, aNan, LAGUERRE_MAX_ITERATIONS).status,
        Status::InvalidInput
    );
    assert_eq!(
        laguerre(&aFiniteCoeffs, 2, 0.0, LAGUERRE_MAX_ITERATIONS).status,
        Status::InvalidInput
    );
}

#[test]
fn MathPoly_LaguerreTest__ForcedNonConvergenceIsReported() {
    let aCoeffs: [f64; 6] = [-120.0, 274.0, -225.0, 85.0, -15.0, 1.0];
    let aResult: PolyResult = laguerre(&aCoeffs, 5, 1.0e-12, 0);
    assert!(!aResult.is_done());
    assert_eq!(aResult.status, Status::MaxIterations);
}
