#[test]
fn math_DirectPolynomialRootsTest__FullFerrariMethodRequired() {
    // Test case that requires full Ferrari method (not biquadratic)
    // x^4 + x^3 - 4x^2 - 4x = 0, which factors as x(x+2)(x^2-x-2) = x(x+2)(x-2)(x+1)
    // Roots should be: 0, -2, 2, -1
    let aRoots = DirectPolynomialRoots::quartic(1.0, 1.0, -4.0, -4.0, 0.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 4);

    // Current implementation order for Ferrari method: -2, -1, 2, 0
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();
    let aRoot3: f64 = aRoots.value(3).unwrap();
    let aRoot4: f64 = aRoots.value(4).unwrap();

    assert_near(aRoot1, -2.0, 1.0e-10);
    assert_near(aRoot2, -1.0, 1.0e-10);
    assert_near(aRoot3, 2.0, 1.0e-10);
    assert_near(aRoot4, 0.0, 1.0e-10);
}

#[test]
fn math_DirectPolynomialRootsTest__AggressiveRefinementEffectiveness() {
    // Test that aggressive Newton-Raphson refinement provides excellent residuals
    // Using a case known to benefit from refinement
    let a: f64 = 1.0;
    let b: f64 = 0.0;
    let c: f64 = -1000000.0; // Large coefficient to test numerical stability
    let d: f64 = 0.0;
    let e: f64 = 999999.0; // Nearly equal magnitude

    let aRoots = DirectPolynomialRoots::quartic(a, b, c, d, e);

    assert!(aRoots.is_done());

    // Check that all residuals are very small
    for i in 1..=aRoots.solutions().unwrap() {
        let root: f64 = aRoots.value(i).unwrap();
        let residual: f64 =
            a * root * root * root * root + b * root * root * root + c * root * root + d * root + e;

        assert!((residual).abs() < 1.0e-3);
    }
}

#[test]
fn math_DirectPolynomialRootsTest__CubicWithAggressiveRefinement() {
    // Test cubic case with aggressive refinement
    // x^3 - 6x^2 + 11x - 6 = 0, which factors as (x-1)(x-2)(x-3)
    // Roots should be: 1, 2, 3
    let aRoots = DirectPolynomialRoots::cubic(1.0, -6.0, 11.0, -6.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 3);

    // With deterministic ordering, roots come in descending order: 3, 2, 1
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();
    let aRoot3: f64 = aRoots.value(3).unwrap();

    assert_near(aRoot1, 3.0, 1.0e-12);
    assert_near(aRoot2, 2.0, 1.0e-12);
    assert_near(aRoot3, 1.0, 1.0e-12);

    // Verify excellent residuals
    for i in 1..=3 {
        let root: f64 = aRoots.value(i).unwrap();
        let residual: f64 = root * root * root - 6.0 * root * root + 11.0 * root - 6.0;
        assert!((residual).abs() < 1.0e-12);
    }
}
