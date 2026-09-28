// Test quintic with known roots: (x-1)(x-2)(x-3)(x-4)(x-5) = x^5 - 15x^4 + 85x^3 - 225x^2 + 274x -
// 120
#[test]
fn MathPoly_LaguerreTest__Quintic_FiveDistinctRoots() {
    let aResult: GeneralPolyResult = quintic(1.0, -15.0, 85.0, -225.0, 274.0, -120.0);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 5);

    for i in 0..aResult.nb_roots {
        let aVal: f64 = aResult.roots[i];
        assert!(is_expected_integer_root(aVal, 5), "Root {aVal} not expected");
    }
}

// Test sextic with known roots: (x-1)(x-2)(x-3)(x-4)(x-5)(x-6)
#[test]
fn MathPoly_LaguerreTest__Sextic_SixDistinctRoots() {
    // (x-1)(x-2)(x-3)(x-4)(x-5)(x-6) = x^6 - 21x^5 + 175x^4 - 735x^3 + 1624x^2 - 1764x + 720
    let aResult: GeneralPolyResult = sextic(1.0, -21.0, 175.0, -735.0, 1624.0, -1764.0, 720.0);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 6);

    for i in 0..aResult.nb_roots {
        let aVal: f64 = aResult.roots[i];
        assert!(is_expected_integer_root(aVal, 6), "Root {aVal} not expected");
    }
}

// Test general Laguerre with all 6 roots
#[test]
fn MathPoly_LaguerreTest__Laguerre_SexticAllRoots() {
    // (x-1)(x-2)(x-3)(x-4)(x-5)(x-6)
    let aCoeffs: [f64; 7] = [720.0, -1764.0, 1624.0, -735.0, 175.0, -21.0, 1.0];

    let aResult: PolyResult = solve(&aCoeffs, 6);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 6);

    for i in 0..aResult.nb_roots {
        let aExpected: f64 = (i + 1) as f64;
        near(aResult.roots[i], aExpected, THE_TOL);
    }
}

// Test quintic with all real roots: (x+2)(x+1)(x)(x-1)(x-2) = x^5 - 5x^3 + 4x
#[test]
fn MathPoly_LaguerreTest__Quintic_SymmetricRoots() {
    let aCoeffs: [f64; 6] = [0.0, 4.0, 0.0, -5.0, 0.0, 1.0];

    let aResult: PolyResult = solve(&aCoeffs, 5);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 5);

    // Should find roots: -2, -1, 0, 1, 2
    near(aResult.roots[0], -2.0, THE_TOL);
    near(aResult.roots[1], -1.0, THE_TOL);
    near(aResult.roots[2], 0.0, THE_TOL);
    near(aResult.roots[3], 1.0, THE_TOL);
    near(aResult.roots[4], 2.0, THE_TOL);
}

// Test sextic: (x+3)(x+2)(x+1)(x-1)(x-2)(x-3) = x^6 - 14x^4 + 49x^2 - 36
#[test]
fn MathPoly_LaguerreTest__Sextic_SymmetricRoots() {
    let aCoeffs: [f64; 7] = [-36.0, 0.0, 49.0, 0.0, -14.0, 0.0, 1.0];

    let aResult: PolyResult = solve(&aCoeffs, 6);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 6);

    near(aResult.roots[0], -3.0, THE_TOL);
    near(aResult.roots[1], -2.0, THE_TOL);
    near(aResult.roots[2], -1.0, THE_TOL);
    near(aResult.roots[3], 1.0, THE_TOL);
    near(aResult.roots[4], 2.0, THE_TOL);
    near(aResult.roots[5], 3.0, THE_TOL);
}

// Test octic with 8 real roots: (x-1)(x-2)(x-3)(x-4)(x-5)(x-6)(x-7)(x-8)
#[test]
fn MathPoly_LaguerreTest__Octic_EightDistinctRoots() {
    // (x-1)(x-2)(x-3)(x-4)(x-5)(x-6)(x-7)(x-8) = x^8 - 36x^7 + 546x^6 - 4536x^5 + 22449x^4 - 67284x^3
    // + 118124x^2 - 109584x + 40320
    let aCoeffs: [f64; 9] = [
        40320.0, -109584.0, 118124.0, -67284.0, 22449.0, -4536.0, 546.0, -36.0, 1.0,
    ];

    let aResult: GeneralPolyResult = octic(&aCoeffs);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 8); // Eight real roots: 1, 2, 3, 4, 5, 6, 7, 8

    for i in 0..aResult.nb_roots {
        let aExpected: f64 = (i + 1) as f64;
        near(aResult.roots[i], aExpected, 1.0e-6);
    }
}

// Test with simple distinct roots (not multiple)
#[test]
fn MathPoly_LaguerreTest__Quintic_SimpleRoots() {
    // (x-1)(x-2)(x-3)(x-4)(x-5) = x^5 - 15x^4 + 85x^3 - 225x^2 + 274x - 120
    let aCoeffs: [f64; 6] = [-120.0, 274.0, -225.0, 85.0, -15.0, 1.0];

    let aResult: PolyResult = solve(&aCoeffs, 5);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 5);

    // Check roots are 1, 2, 3, 4, 5
    for i in 0..aResult.nb_roots {
        let aExpected: f64 = (i + 1) as f64;
        near(aResult.roots[i], aExpected, THE_TOL);
    }
}

// Test quartic via Laguerre (degree reduction)
#[test]
fn MathPoly_LaguerreTest__Laguerre_Quartic() {
    // (x-1)(x-2)(x-3)(x-4) = x^4 - 10x^3 + 35x^2 - 50x + 24
    let aCoeffs: [f64; 5] = [24.0, -50.0, 35.0, -10.0, 1.0];

    let aResult: PolyResult = solve(&aCoeffs, 4);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 4);

    near(aResult.roots[0], 1.0, THE_TOL);
    near(aResult.roots[1], 2.0, THE_TOL);
    near(aResult.roots[2], 3.0, THE_TOL);
    near(aResult.roots[3], 4.0, THE_TOL);
}

// Test accuracy: verify roots satisfy the polynomial
#[test]
fn MathPoly_LaguerreTest__Laguerre_VerifyRoots() {
    // Well-behaved polynomial with known real roots: (x-1)(x-2)(x-3)(x-4)(x-5)(x-6)
    let aCoeffs: [f64; 7] = [720.0, -1764.0, 1624.0, -735.0, 175.0, -21.0, 1.0];

    let aResult: PolyResult = solve(&aCoeffs, 6);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 6);

    // Verify each real root
    for i in 0..aResult.nb_roots {
        let aVal: f64 = EvaluatePoly(&aCoeffs, 6, aResult.roots[i]);
        near(aVal, 0.0, 1.0e-6);
    }
}

// Test high-degree polynomial (degree 10)
#[test]
fn MathPoly_LaguerreTest__Laguerre_Degree10() {
    // (x-1)(x-2)...(x-10) - only test that it computes without crashing
    // Coefficients from expansion of product
    let aCoeffs: [f64; 11] = [
        3628800.0,   // 10!
        -10628640.0, // sum of products
        12753576.0,
        -8409500.0,
        3416930.0,
        -902055.0,
        157773.0,
        -18150.0,
        1320.0,
        -55.0,
        1.0,
    ];

    let aResult: PolyResult = solve(&aCoeffs, 10);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 10);

    // Verify roots are 1, 2, ..., 10
    for i in 0..aResult.nb_roots {
        let aExpected: f64 = (i + 1) as f64;
        near(aResult.roots[i], aExpected, 1.0e-6);
    }
}

// Performance comparison (informational)
#[test]
fn MathPoly_LaguerreTest__Performance_CompareToQuartic() {
    // Compare Laguerre on degree 4 vs dedicated Quartic solver
    let aCoeffs: [f64; 5] = [24.0, -50.0, 35.0, -10.0, 1.0]; // (x-1)(x-2)(x-3)(x-4)

    // Laguerre
    let aLagResult: PolyResult = solve(&aCoeffs, 4);
    assert!(aLagResult.is_done());

    // Quartic
    let aQuartResult: PolyResult = quartic(1.0, -10.0, 35.0, -50.0, 24.0);
    assert!(aQuartResult.is_done());

    assert_eq!(aLagResult.nb_roots, aQuartResult.nb_roots);

    for i in 0..aQuartResult.nb_roots {
        near(aLagResult.roots[i], aQuartResult.roots[i], THE_TOL);
    }
}

#[test]
fn MathPoly_LaguerreTest__RepeatedRootsReportMultiplicity() {
    // (x-1)^3 * (x+2)^2
    let aCoeffs: [f64; 6] = [-4.0, 8.0, -1.0, -5.0, 1.0, 1.0];
    let aResult: PolyResult = laguerre(&aCoeffs, 5, 1.0e-11, LAGUERRE_MAX_ITERATIONS);
    assert!(
        aResult.is_done(),
        "status={}, real roots={}, complex roots={}",
        aResult.status as i32,
        aResult.nb_roots,
        aResult.nb_complex_roots
    );
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0], -2.0, 1.0e-6);
    near(aResult.roots[1], 1.0, 1.0e-6);
    assert_eq!(aResult.multiplicities[0], 2);
    assert_eq!(aResult.multiplicities[1], 3);
}
