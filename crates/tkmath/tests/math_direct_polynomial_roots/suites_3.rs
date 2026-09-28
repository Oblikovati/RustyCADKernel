#[test]
fn math_DirectPolynomialRootsTest__PolynomialEvaluation() {
    // Test root verification by polynomial evaluation
    let aRoots = DirectPolynomialRoots::quadratic(1.0, -3.0, 2.0); // x^2 - 3x + 2 = 0, roots: 1, 2

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 2);

    // Verify both roots satisfy the equation x^2 - 3x + 2 = 0 using helper method
    for i in 1..=aRoots.solutions().unwrap() {
        let aRoot: f64 = aRoots.value(i).unwrap();
        verify_root(aRoot, &[1.0, -3.0, 2.0], i, 1e-10);
    }
}

#[test]
fn math_DirectPolynomialRootsTest__NearZeroCoefficients() {
    // Test with very small leading coefficient (effectively lower degree)
    let aRoots = DirectPolynomialRoots::quadratic(1.0e-15, 1.0, -2.0); // Effectively linear: x - 2 = 0

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 2);

    // With very small leading coefficient, we get one small positive root near 2 and one very large
    // negative root
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();

    // One root should be near 2 (the meaningful solution from linear approximation)
    let aFoundNearTwo: bool = ((aRoot1 - 2.0).abs() < 1.0e-6) || ((aRoot2 - 2.0).abs() < 1.0e-6);
    assert!(aFoundNearTwo);
}

#[test]
fn math_DirectPolynomialRootsTest__BiQuadraticPolynomial() {
    // Test biquadratic: x^4 - 10x^2 + 9 = (x^2-1)(x^2-9) = 0
    // Roots: -3, -1, 1, 3
    let aRoots = DirectPolynomialRoots::quartic(1.0, 0.0, -10.0, 0.0, 9.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 4);

    // Natural algorithm order for biquadratic: -3, -1, 3, 1
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();
    let aRoot3: f64 = aRoots.value(3).unwrap();
    let aRoot4: f64 = aRoots.value(4).unwrap();

    assert_near(aRoot1, -3.0, 1.0e-8);
    assert_near(aRoot2, -1.0, 1.0e-8);
    assert_near(aRoot3, 3.0, 1.0e-8);
    assert_near(aRoot4, 1.0, 1.0e-8);

    // Verify roots satisfy the equation x^4 - 10x^2 + 9 = 0
    let aCoeffs: [f64; 5] = [1.0, 0.0, -10.0, 0.0, 9.0];
    verify_root(aRoot1, &aCoeffs, 1, 1.0e-8);
    verify_root(aRoot2, &aCoeffs, 2, 1.0e-8);
    verify_root(aRoot3, &aCoeffs, 3, 1.0e-8);
    verify_root(aRoot4, &aCoeffs, 4, 1.0e-8);
}
