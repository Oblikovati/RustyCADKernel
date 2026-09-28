// ============================================================================
// Quartic equation tests
// ============================================================================

#[test]
fn MathPoly_QuarticTest__FourDistinctRoots() {
    // (x-1)(x-2)(x-3)(x-4) = x^4 - 10x^3 + 35x^2 - 50x + 24 = 0
    let aResult: PolyResult = quartic(1.0, -10.0, 35.0, -50.0, 24.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 4);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
    near(aResult.roots[2], 3.0, THE_TOLERANCE);
    near(aResult.roots[3], 4.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuarticTest__TwoRealRoots() {
    // x^4 - 5x^2 + 4 = 0 -> roots: -2, -1, 1, 2
    let aResult: PolyResult = quartic(1.0, 0.0, -5.0, 0.0, 4.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 4);
    near(aResult.roots[0], -2.0, THE_TOLERANCE);
    near(aResult.roots[1], -1.0, THE_TOLERANCE);
    near(aResult.roots[2], 1.0, THE_TOLERANCE);
    near(aResult.roots[3], 2.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuarticTest__NoRealRoots() {
    // x^4 + 1 = 0 -> no real roots
    let aResult: PolyResult = quartic(1.0, 0.0, 0.0, 0.0, 1.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 0);
}

#[test]
fn MathPoly_QuarticTest__Biquadratic() {
    // x^4 - 5x^2 + 4 = 0 is biquadratic (no x^3 or x term)
    let aResult: PolyResult = quartic(1.0, 0.0, -5.0, 0.0, 4.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 4);
    for i in 0..aResult.nb_roots {
        let aValue: f64 = EvalQuartic(1.0, 0.0, -5.0, 0.0, 4.0, aResult.roots[i]);
        near(aValue, 0.0, THE_TOLERANCE);
    }
}

#[test]
fn MathPoly_QuarticTest__ReducesToCubic() {
    // 0*x^4 + x^3 - 6x^2 + 11x - 6 = 0 -> roots: 1, 2, 3
    let aResult: PolyResult = quartic(0.0, 1.0, -6.0, 11.0, -6.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 3);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
    near(aResult.roots[2], 3.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuarticTest__QuadrupleRoot() {
    // (x-2)^4 = x^4 - 8x^3 + 24x^2 - 32x + 16 = 0
    let aResult: PolyResult = quartic(1.0, -8.0, 24.0, -32.0, 16.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], 2.0, THE_TOLERANCE);
    assert_eq!(aResult.multiplicities[0], 4);
}

#[test]
fn MathPoly_QuarticTest__TwoDoubleRoots() {
    // (x-1)^2 * (x-3)^2 = x^4 - 8x^3 + 22x^2 - 24x + 9 = 0
    let aResult: PolyResult = quartic(1.0, -8.0, 22.0, -24.0, 9.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
    assert_eq!(aResult.multiplicities[0], 2);
    near(aResult.roots[1], 3.0, THE_TOLERANCE);
    assert_eq!(aResult.multiplicities[1], 2);
}

#[test]
fn MathPoly_QuarticTest__RootsAreSorted() {
    let aResult: PolyResult = quartic(1.0, -10.0, 35.0, -50.0, 24.0);
    assert!(aResult.is_done());
    for i in 1..aResult.nb_roots {
        assert!(aResult.roots[i - 1] <= aResult.roots[i]);
    }
}

#[test]
fn MathPoly_QuarticTest__VerifyRootsSatisfyEquation() {
    // General quartic with all roots
    let aResult: PolyResult = quartic(1.0, -10.0, 35.0, -50.0, 24.0);
    assert!(aResult.is_done());
    for i in 0..aResult.nb_roots {
        let aValue: f64 = EvalQuartic(1.0, -10.0, 35.0, -50.0, 24.0, aResult.roots[i]);
        near(aValue, 0.0, THE_TOLERANCE);
    }
}

#[test]
fn MathPoly_QuarticTest__GloballyScaledRepeatedEquation() {
    for aScale in [1.0e-280, 1.0e280] {
        let aResult: PolyResult =
            quartic(aScale, -8.0 * aScale, 22.0 * aScale, -24.0 * aScale, 9.0 * aScale);
        assert!(aResult.is_done());
        assert_eq!(aResult.nb_roots, 2);
        near(aResult.roots[0], 1.0, 1.0e-10);
        near(aResult.roots[1], 3.0, 1.0e-10);
        assert_eq!(aResult.multiplicities[0], 2);
        assert_eq!(aResult.multiplicities[1], 2);
    }
}

#[test]
fn MathPoly_QuarticTest__FourUnevenRealRoots() {
    // x^4 + 0.7*x^3 - 5.2*x^2 - 1.3*x + 3.4 = 0
    let aResult: PolyResult = quartic(1.0, 0.7, -5.2, -1.3, 3.4);

    assert_eq!(aResult.status, Status::Ok);
    assert_eq!(aResult.nb_roots, 4);
    const THE_EXPECTED_ROOTS: [f64; 4] = [
        -2.3964634709491510,
        -0.97207148860835169,
        0.76791933673826718,
        1.9006156228192357,
    ];
    for anIndex in 0..aResult.nb_roots {
        assert_eq!(aResult.multiplicities[anIndex], 1);
        near(aResult.roots[anIndex], THE_EXPECTED_ROOTS[anIndex], 1.0e-10);
        if anIndex > 0 {
            assert!(aResult.roots[anIndex - 1] < aResult.roots[anIndex]);
        }
    }
}

#[test]
fn MathPoly_ComplexRootsTest__QuarticWithRealAndComplexRoots() {
    // 1.3*x^4 - 0.4*x^3 + 0.7*x^2 - 2.2*x - 1.1 = 0
    let aCoefficients: [f64; 5] = [-1.1, -2.2, 0.7, -0.4, 1.3];
    let aResult: PolyResult = laguerre(&aCoefficients, 4, LAGUERRE_TOLERANCE, LAGUERRE_MAX_ITERATIONS);

    assert_eq!(aResult.status, Status::Ok);
    assert_eq!(aResult.nb_roots, 2);
    assert!(aResult.roots[0] < aResult.roots[1]);
    near(aResult.roots[0], -0.41478958662056115, 1.0e-10);
    near(aResult.roots[1], 1.2936081173279923, 1.0e-10);
    assert_eq!(aResult.multiplicities[0], 1);
    assert_eq!(aResult.multiplicities[1], 1);

    assert_eq!(aResult.nb_complex_roots, 2);
    near(aResult.complex_roots[0].real(), -0.28556311150756175, 1.0e-10);
    near(aResult.complex_roots[0].imag(), 1.2228682399186652, 1.0e-10);
    near(aResult.complex_roots[1].real(), -0.28556311150756175, 1.0e-10);
    near(aResult.complex_roots[1].imag(), -1.2228682399186652, 1.0e-10);
    assert_eq!(aResult.complex_multiplicities[0], 1);
    assert_eq!(aResult.complex_multiplicities[1], 1);
}

// ============================================================================
// Boolean conversion tests
// ============================================================================

#[test]
fn MathPoly_BoolConversionTest__SuccessfulResultIsTrue() {
    let aResult: PolyResult = quadratic(1.0, -5.0, 6.0);
    assert!(bool::from(&aResult));
}

#[test]
fn MathPoly_BoolConversionTest__NoSolutionResultIsFalse() {
    let aResult: PolyResult = linear(0.0, 5.0);
    assert!(!bool::from(&aResult));
}

// ============================================================================
// Indexing operator tests
// ============================================================================

#[test]
fn MathPoly_IndexingTest__BracketOperator() {
    let aResult: PolyResult = quadratic(1.0, -5.0, 6.0);
    assert!(aResult.is_done());
    assert_eq!(aResult[0], aResult.roots[0]);
    assert_eq!(aResult[1], aResult.roots[1]);
}
