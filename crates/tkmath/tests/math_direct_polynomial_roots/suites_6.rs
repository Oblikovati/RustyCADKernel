#[test]
fn math_DirectPolynomialRootsTest__ExtremeCoefficientsStability() {
    // Test numerical stability with extreme coefficient ranges
    // Similar to the problematic case but with even more extreme values
    let a: f64 = 1.0;
    let b: f64 = 1.0e-15; // Very small
    let c: f64 = -1.0e8; // Large negative
    let d: f64 = 1.0e-10; // Very small positive
    let e: f64 = 1.0e15; // Very large positive

    let aRoots = DirectPolynomialRoots::quartic(a, b, c, d, e);

    assert!(aRoots.is_done());

    // At minimum, verify that residuals are reasonable (not astronomical)
    for i in 1..=aRoots.solutions().unwrap() {
        let root: f64 = aRoots.value(i).unwrap();
        let residual: f64 =
            a * root * root * root * root + b * root * root * root + c * root * root + d * root + e;

        // With extreme coefficients, we expect residuals to be much better than old implementation
        assert!((residual).abs() < 1.0e6);
    }
}

#[test]
fn math_DirectPolynomialRootsTest__CloseToZeroCoefficients() {
    // Test robustness when coefficients are close to zero (degree reduction)
    // x^4 + 0*x^3 + 0*x^2 + 0*x - 16 = 0, essentially x^4 = 16
    // Roots should be +/-2, +/-2i (but we only get real roots: +/-2)
    let epsilon: f64 = 1.0e-14;
    let aRoots = DirectPolynomialRoots::quartic(1.0, epsilon, epsilon, epsilon, -16.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 2);

    // Natural algorithm order for near-zero coefficients: 2, -2
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();

    assert_near(aRoot1, 2.0, 1.0e-10);
    assert_near(aRoot2, -2.0, 1.0e-10);
}
