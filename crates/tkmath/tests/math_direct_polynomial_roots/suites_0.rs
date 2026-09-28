#[test]
fn math_DirectPolynomialRootsTest__QuadraticRoots() {
    // Test quadratic: x^2 - 5x + 6 = 0, roots should be 2 and 3
    let aRoots = DirectPolynomialRoots::quadratic(1.0, -5.0, 6.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 2);

    // Natural algorithm order from quadratic solver
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();

    assert_near(aRoot1, 3.0, MY_TOLERANCE);
    assert_near(aRoot2, 2.0, MY_TOLERANCE);

    // Verify roots satisfy the equation x^2 - 5x + 6 = 0
    verify_root(aRoot1, &[1.0, -5.0, 6.0], 1, 1e-10);
    verify_root(aRoot2, &[1.0, -5.0, 6.0], 2, 1e-10);
}

#[test]
fn math_DirectPolynomialRootsTest__QuadraticNoRealRoots() {
    // Test quadratic: x^2 + x + 1 = 0, no real roots (discriminant < 0)
    let aRoots = DirectPolynomialRoots::quadratic(1.0, 1.0, 1.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 0);
}
