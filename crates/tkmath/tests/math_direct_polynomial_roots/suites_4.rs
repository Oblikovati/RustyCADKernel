#[test]
fn math_DirectPolynomialRootsTest__RepeatedRoots() {
    // Test polynomial with repeated roots: (x-1)^3 = x^3 - 3x^2 + 3x - 1 = 0
    let aRoots = DirectPolynomialRoots::cubic(1.0, -3.0, 3.0, -1.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 3);

    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();
    let aRoot3: f64 = aRoots.value(3).unwrap();
    assert_near(aRoot1, 1.0, 1.0e-8);
    assert_near(aRoot2, 1.0, 1.0e-8);
    assert_near(aRoot3, 1.0, 1.0e-8);

    // Verify roots satisfy the equation x^3 - 3x^2 + 3x - 1 = 0
    verify_root(aRoot1, &[1.0, -3.0, 3.0, -1.0], 1, 1.0e-8);
    verify_root(aRoot2, &[1.0, -3.0, 3.0, -1.0], 2, 1.0e-8);
    verify_root(aRoot3, &[1.0, -3.0, 3.0, -1.0], 3, 1.0e-8);
}

// ====================================================================================================
// Tests for the modernized C++17 implementation with aggressive Newton-Raphson refinement
// ====================================================================================================

#[test]
fn math_DirectPolynomialRootsTest__ProblematicQuarticCaseDetailed() {
    // The specific problematic case that motivated the implementation rework
    // Coefficients that caused issues with previous implementation
    let a: f64 = 1.0000000000000004;
    let b: f64 = -2.2737367544323211e-13;
    let c: f64 = -17361733.368892364;
    let d: f64 = 28.998342463569099;
    let e: f64 = 75357446168894.516;

    // Expected roots from external solver verification (in descending order)
    // These values are used for verification in the test below

    let aRoots = DirectPolynomialRoots::quartic(a, b, c, d, e);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 4);
    assert!(!aRoots.infinite_roots());

    // Collect actual roots for analysis
    let mut actualRoots: [f64; 4] = [0.0; 4];
    for i in 1..=4 {
        actualRoots[(i - 1) as usize] = aRoots.value(i).unwrap();
    }

    // Current implementation order for problematic quartic case
    assert_near(actualRoots[0], -2946.425490, 1.0e-3);
    assert_near(actualRoots[1], -2946.236617, 1.0e-3);
    assert_near(actualRoots[2], 2946.394276, 1.0e-3);
    assert_near(actualRoots[3], 2946.267830, 1.0e-3);

    // Verify residuals are extremely small (main goal of the rework)
    for i in 1..=4 {
        let root: f64 = aRoots.value(i).unwrap();
        let residual: f64 =
            a * root * root * root * root + b * root * root * root + c * root * root + d * root + e;

        assert!((residual).abs() < 1.0);
    }
}

#[test]
fn math_DirectPolynomialRootsTest__ModernImplementationBiquadraticDetection() {
    // Test case that should be detected as biquadratic (q ~= 0 after Ferrari transformation)
    // x^4 - 5x^2 + 4 = 0, which factors as (x^2-1)(x^2-4) = 0
    // Roots should be: +/-1, +/-2
    let aRoots = DirectPolynomialRoots::quartic(1.0, 0.0, -5.0, 0.0, 4.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 4);

    // Natural algorithm order for biquadratic: -2, -1, 2, 1
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();
    let aRoot3: f64 = aRoots.value(3).unwrap();
    let aRoot4: f64 = aRoots.value(4).unwrap();

    assert_near(aRoot1, -2.0, 1.0e-12);
    assert_near(aRoot2, -1.0, 1.0e-12);
    assert_near(aRoot3, 2.0, 1.0e-12);
    assert_near(aRoot4, 1.0, 1.0e-12);
}
