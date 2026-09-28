#[test]
fn math_DirectPolynomialRootsTest__QuarticRoots() {
    // Test quartic: (x-1)(x-2)(x+1)(x+2) = (x^2-1)(x^2-4) = x^4 - 5x^2 + 4 = 0
    // Roots should be -2, -1, 1, 2
    let aRoots = DirectPolynomialRoots::quartic(1.0, 0.0, -5.0, 0.0, 4.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 4);

    // Natural algorithm order from Ferrari's method: -2, -1, 2, 1
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();
    let aRoot3: f64 = aRoots.value(3).unwrap();
    let aRoot4: f64 = aRoots.value(4).unwrap();

    assert_near(aRoot1, -2.0, MY_TOLERANCE);
    assert_near(aRoot2, -1.0, MY_TOLERANCE);
    assert_near(aRoot3, 2.0, MY_TOLERANCE);
    assert_near(aRoot4, 1.0, MY_TOLERANCE);

    // Verify roots satisfy the equation x^4 - 5x^2 + 4 = 0
    let aCoeffs: [f64; 5] = [1.0, 0.0, -5.0, 0.0, 4.0];
    verify_root(aRoot1, &aCoeffs, 1, 1e-10);
    verify_root(aRoot2, &aCoeffs, 2, 1e-10);
    verify_root(aRoot3, &aCoeffs, 3, 1e-10);
    verify_root(aRoot4, &aCoeffs, 4, 1e-10);
}

#[test]
fn math_DirectPolynomialRootsTest__LinearCase() {
    // Test linear: 2x - 6 = 0, root should be x = 3
    let aRoots = DirectPolynomialRoots::linear(2.0, -6.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 1);

    let aRoot: f64 = aRoots.value(1).unwrap();
    assert_near(aRoot, 3.0, MY_TOLERANCE);

    // Verify root satisfies the equation 2x - 6 = 0
    verify_root(aRoot, &[2.0, -6.0], 1, 1e-10);
}

#[test]
fn math_DirectPolynomialRootsTest__DegenerateLinearCase() {
    // Test degenerate linear: 0x + 5 = 0 (no solution)
    let aRoots = DirectPolynomialRoots::linear(0.0, 5.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 0);
}
