// ============================================================================
// Linear equation tests
// ============================================================================

#[test]
fn MathPoly_LinearTest__SimpleLinear() {
    // 2x + 4 = 0 -> x = -2
    let aResult: PolyResult = linear(2.0, 4.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], -2.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_LinearTest__ZeroCoefficient_InfiniteSolutions() {
    // 0*x + 0 = 0 -> infinite solutions
    let aResult: PolyResult = linear(0.0, 0.0);
    assert_eq!(aResult.status, Status::InfiniteSolutions);
}

#[test]
fn MathPoly_LinearTest__ZeroCoefficient_NoSolution() {
    // 0*x + 5 = 0 -> no solution
    let aResult: PolyResult = linear(0.0, 5.0);
    assert_eq!(aResult.status, Status::NoSolution);
}

#[test]
fn MathPoly_LinearTest__UnrepresentableFiniteRootIsNumericalError() {
    // numeric_limits<double>::denorm_min() and max().
    let aResult: PolyResult = linear(f64::from_bits(1), f64::MAX);
    assert_eq!(aResult.status, Status::NumericalError);
    assert_eq!(aResult.nb_roots, 0);
}

// ============================================================================
// Quadratic equation tests
// ============================================================================

#[test]
fn MathPoly_QuadraticTest__TwoDistinctRoots() {
    // x^2 - 5x + 6 = 0 -> roots: 2, 3
    let aResult: PolyResult = quadratic(1.0, -5.0, 6.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0], 2.0, THE_TOLERANCE);
    near(aResult.roots[1], 3.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuadraticTest__DoubleRoot() {
    // x^2 - 4x + 4 = 0 -> root: 2 (double)
    let aResult: PolyResult = quadratic(1.0, -4.0, 4.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    assert_eq!(aResult.multiplicities[0], 2);
    near(aResult.roots[0], 2.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuadraticTest__NoRealRoots() {
    // x^2 + 1 = 0 -> no real roots
    let aResult: PolyResult = quadratic(1.0, 0.0, 1.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 0);
}

#[test]
fn MathPoly_QuadraticTest__NegativeRoots() {
    // x^2 + 5x + 6 = 0 -> roots: -3, -2
    let aResult: PolyResult = quadratic(1.0, 5.0, 6.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0], -3.0, THE_TOLERANCE);
    near(aResult.roots[1], -2.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuadraticTest__MixedSignRoots() {
    // x^2 - 1 = 0 -> roots: -1, 1
    let aResult: PolyResult = quadratic(1.0, 0.0, -1.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0], -1.0, THE_TOLERANCE);
    near(aResult.roots[1], 1.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuadraticTest__ReducesToLinear() {
    // 0*x^2 + 2x + 4 = 0 -> x = -2
    let aResult: PolyResult = quadratic(0.0, 2.0, 4.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], -2.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuadraticTest__LargeCoefficients() {
    // 1e6*x^2 - 2e6*x + 1e6 = 0 -> root: 1 (double)
    let aResult: PolyResult = quadratic(1.0e6, -2.0e6, 1.0e6);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 1);
    near(aResult.roots[0], 1.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuadraticTest__SmallCoefficients() {
    // 1e-6*x^2 - 5e-6*x + 6e-6 = 0 -> roots: 2, 3
    let aResult: PolyResult = quadratic(1.0e-6, -5.0e-6, 6.0e-6);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0], 2.0, THE_TOLERANCE);
    near(aResult.roots[1], 3.0, THE_TOLERANCE);
}

#[test]
fn MathPoly_QuadraticTest__RootsAreSorted() {
    // 2x^2 + 3x - 2 = 0 -> roots: -2, 0.5
    let aResult: PolyResult = quadratic(2.0, 3.0, -2.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    assert!(aResult.roots[0] < aResult.roots[1]);
}

#[test]
fn MathPoly_QuadraticTest__UnequalNonIntegralRoots() {
    // 3.7*x^2 - 2.1*x - 4.3 = 0
    let aResult: PolyResult = quadratic(3.7, -2.1, -4.3);

    assert_eq!(aResult.status, Status::Ok);
    assert_eq!(aResult.nb_roots, 2);
    assert!(aResult.roots[0] < aResult.roots[1]);
    assert_eq!(aResult.multiplicities[0], 1);
    assert_eq!(aResult.multiplicities[1], 1);
    near(aResult.roots[0], -0.83097870127785634, 1.0e-13);
    near(aResult.roots[1], 1.3985462688454240, 1.0e-13);
}

#[test]
fn MathPoly_QuadraticTest__GlobalCoefficientScalingDoesNotChangeRoots() {
    for aScale in [1.0e-300, 1.0e300] {
        let aResult: PolyResult = quadratic(aScale, -5.0 * aScale, 6.0 * aScale);
        assert!(aResult.is_done());
        assert_eq!(aResult.nb_roots, 2);
        near(aResult.roots[0], 2.0, THE_TOLERANCE);
        near(aResult.roots[1], 3.0, THE_TOLERANCE);
    }
}

#[test]
fn MathPoly_QuadraticTest__ExtremeRootMagnitudes() {
    let aResult: PolyResult = quadratic(1.0, -1.0e150, 1.0);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots, 2);
    near(aResult.roots[0] / 1.0e-150, 1.0, 1.0e-12);
    near(aResult.roots[1] / 1.0e150, 1.0, 1.0e-12);
}

#[test]
fn MathPoly_QuadraticTest__AdjacentDiscriminantsRemainDistinct() {
    // std::nextafter(1.0, 0.0) and std::nextafter(1.0, 2.0).
    let aBelowOne: f64 = f64::from_bits(1.0_f64.to_bits() - 1);
    let aRealResult: PolyResult = quadratic(1.0, -2.0, aBelowOne);
    assert_eq!(aRealResult.status, Status::Ok);
    assert_eq!(aRealResult.nb_roots, 2);
    assert!(aRealResult.roots[0] < 1.0);
    assert!(aRealResult.roots[1] > 1.0);
    assert_eq!(aRealResult.multiplicities[0], 1);
    assert_eq!(aRealResult.multiplicities[1], 1);

    let aAboveOne: f64 = f64::from_bits(1.0_f64.to_bits() + 1);
    let aComplexResult: PolyResult = quadratic(1.0, -2.0, aAboveOne);
    assert_eq!(aComplexResult.status, Status::Ok);
    assert_eq!(aComplexResult.nb_roots, 0);
}

#[test]
fn MathPoly_QuadraticTest__RejectsNonFiniteCoefficients() {
    let aNan: f64 = f64::NAN;
    let anInfinity: f64 = f64::INFINITY;
    assert_eq!(quadratic(aNan, 1.0, 1.0).status, Status::InvalidInput);
    assert_eq!(quadratic(1.0, anInfinity, 1.0).status, Status::InvalidInput);
}
