// ============================================================================
// Cubic equation tests
// ============================================================================

#[test]
fn MathPoly_CubicTest__ThreeDistinctRoots() {
    // x^3 - 6x^2 + 11x - 6 = 0 -> roots: 1, 2, 3
    let aResult: PolyResult = cubic(1.0, -6.0, 11.0, -6.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 3);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
    near(aResult.roots[2], 3.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_CubicTest__MixedSignFractionalRoots() {
    let aResult: PolyResult = cubic(1.0, -1.5, -5.5, 3.0);

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 3);
    const THE_EXPECTED_ROOTS: [f64; 3] = [-2.0, 0.5, 3.0];
    for anIndex in 0..aResult.nb_roots {
        near(aResult.roots[anIndex], THE_EXPECTED_ROOTS[anIndex], 1.0e-13);
    }
}

#[test]
fn MathPoly_CubicTest__OneRealRoot() {
    // x^3 + x + 1 = 0 -> one real root approximately -0.6824
    let aResult: PolyResult = cubic(1.0, 0.0, 1.0, 1.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    // Verify the root satisfies the equation
    let aValue: f64 = EvalCubic(1.0, 0.0, 1.0, 1.0, aResult.roots[0]);
    near(aValue, 0.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_CubicTest__TripleRoot() {
    // x^3 - 3x^2 + 3x - 1 = 0 -> root: 1 (triple)
    // This is (x-1)^3
    let aResult: PolyResult = cubic(1.0, -3.0, 3.0, -1.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
    assert_eq!(aResult.multiplicities[0], 3);
}

#[test]
fn MathPoly_CubicTest__OneSimpleOneDouble() {
    // x^3 - 5x^2 + 8x - 4 = 0 -> roots: 1, 2 (double)
    // This is (x-1)(x-2)^2
    let aResult: PolyResult = cubic(1.0, -5.0, 8.0, -4.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
    assert_eq!(aResult.multiplicities[0], 1);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
    assert_eq!(aResult.multiplicities[1], 2);
}

#[test]
fn MathPoly_CubicTest__ReducesToQuadratic() {
    // 0*x^3 + x^2 - 5x + 6 = 0 -> roots: 2, 3
    let aResult: PolyResult = cubic(0.0, 1.0, -5.0, 6.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0], 2.0, THE_TOLERANCE);
    near(aResult.roots[1], 3.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_CubicTest__NegativeRoots() {
    // x^3 + 6x^2 + 11x + 6 = 0 -> roots: -3, -2, -1
    let aResult: PolyResult = cubic(1.0, 6.0, 11.0, 6.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 3);
    near(aResult.roots[0], -3.0, THE_TOLERANCE);
    near(aResult.roots[1], -2.0, THE_TOLERANCE);
    near(aResult.roots[2], -1.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_CubicTest__DepressedCubic() {
    // x^3 - 7x + 6 = 0 -> roots: -3, 1, 2
    let aResult: PolyResult = cubic(1.0, 0.0, -7.0, 6.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 3);
    near(aResult.roots[0], -3.0, THE_TOLERANCE);
    near(aResult.roots[1], 1.0, THE_TOLERANCE);
    near(aResult.roots[2], 2.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_CubicTest__RootsAreSorted() {
    let aResult: PolyResult = cubic(1.0, -6.0, 11.0, -6.0);
    assert!(aResult.is_done());
    for i in 1..aResult.nb_roots {
        assert!(aResult.roots[i - 1] <= aResult.roots[i]);
    }
}

#[test]
fn MathPoly_CubicTest__ClusteredRootsRemainDistinct() {
    // (x-1)(x-1.001)(x-2)
    let aResult: PolyResult = cubic(1.0, -4.001, 5.003, -2.002);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 3);
    near(aResult.roots[0], 1.0, 1.0e-11);
    near(aResult.roots[1], 1.001, 1.0e-11);
    near(aResult.roots[2], 2.0, 1.0e-11);
}

#[test]
fn MathPoly_CubicTest__CloselySpacedRootsRemainDistinct() {
    // x * (x - 1.0e-6) * (x - 1)
    let aResult: PolyResult = cubic(1.0, -1.000001, 1.0e-6, 0.0);

    assert_eq!(aResult.status, Status::Ok);
    assert_eq!(aResult.nb_roots, 3);
    const THE_EXPECTED_ROOTS: [f64; 3] = [0.0, 1.0e-6, 1.0];
    for anIndex in 0..aResult.nb_roots {
        assert_eq!(aResult.multiplicities[anIndex], 1);
        near(aResult.roots[anIndex], THE_EXPECTED_ROOTS[anIndex], 1.0e-13);
    }
}

#[test]
fn MathPoly_CubicTest__ThreeUnevenRealRoots() {
    // 1.7*x^3 - 2.3*x^2 - 4.8*x + 1.1 = 0
    let aResult: PolyResult = cubic(1.7, -2.3, -4.8, 1.1);

    assert_eq!(aResult.status, Status::Ok);
    assert_eq!(aResult.nb_roots, 3);
    const THE_EXPECTED_ROOTS: [f64; 3] = [-1.2704399324056863, 0.21113914549269189, 2.4122419633835825];
    for anIndex in 0..aResult.nb_roots {
        assert_eq!(aResult.multiplicities[anIndex], 1);
        near(aResult.roots[anIndex], THE_EXPECTED_ROOTS[anIndex], 1.0e-12);
        if anIndex > 0 {
            assert!(aResult.roots[anIndex - 1] < aResult.roots[anIndex]);
        }
    }
}

#[test]
fn MathPoly_CubicTest__FractionalTangentialRoot() {
    // x^3 - 2.125*x^2 + 0.171875*x + 1.181640625 = 0
    let aResult: PolyResult = cubic(1.0, -2.125, 0.171875, 1.181640625);

    assert_eq!(aResult.status, Status::Ok);
    assert_eq!(aResult.nb_roots, 2);
    assert!(aResult.roots[0] < aResult.roots[1]);
    near(aResult.roots[0], -0.625, 1.0e-13);
    assert_eq!(aResult.multiplicities[0], 1);
    near(aResult.roots[1], 1.375, 1.0e-12);
    assert_eq!(aResult.multiplicities[1], 2);
}

#[test]
fn MathPoly_CubicTest__LargeScaleIrregularEquation() {
    // 2.3e80*x^3 - 1.7e80*x^2 - 3.9e80*x + 6.5e79 = 0
    let aResult: PolyResult = cubic(2.3e80, -1.7e80, -3.9e80, 6.5e79);

    assert_eq!(aResult.status, Status::Ok);
    assert_eq!(aResult.nb_roots, 3);
    const THE_EXPECTED_ROOTS: [f64; 3] = [-1.0776640490606961, 0.15810155136079423, 1.6586929324825108];
    for anIndex in 0..aResult.nb_roots {
        assert_eq!(aResult.multiplicities[anIndex], 1);
        near(aResult.roots[anIndex], THE_EXPECTED_ROOTS[anIndex], 1.0e-11);
        if anIndex > 0 {
            assert!(aResult.roots[anIndex - 1] < aResult.roots[anIndex]);
        }
    }
}
