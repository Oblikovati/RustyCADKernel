#[test]
fn math_DirectPolynomialRootsTest__QuadraticDoubleRoot() {
    // Test quadratic: (x-1)^2 = x^2 - 2x + 1 = 0, double root at x = 1
    let aRoots = DirectPolynomialRoots::quadratic(1.0, -2.0, 1.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 2);

    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();
    assert_near(aRoot1, 1.0, MY_TOLERANCE);
    assert_near(aRoot2, 1.0, MY_TOLERANCE);

    // Verify roots satisfy the equation x^2 - 2x + 1 = 0
    verify_root(aRoot1, &[1.0, -2.0, 1.0], 1, 1e-10);
    verify_root(aRoot2, &[1.0, -2.0, 1.0], 2, 1e-10);
}

#[test]
fn math_DirectPolynomialRootsTest__CubicRoots() {
    // Test cubic: (x-1)(x-2)(x-3) = x^3 - 6x^2 + 11x - 6 = 0
    // Roots should be 1, 2, 3
    let aRoots = DirectPolynomialRoots::cubic(1.0, -6.0, 11.0, -6.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 3);

    // With our root ordering fix, roots should come in descending order: 3, 2, 1
    let aRoot1: f64 = aRoots.value(1).unwrap();
    let aRoot2: f64 = aRoots.value(2).unwrap();
    let aRoot3: f64 = aRoots.value(3).unwrap();

    assert_near(aRoot1, 3.0, MY_TOLERANCE);
    assert_near(aRoot2, 2.0, MY_TOLERANCE);
    assert_near(aRoot3, 1.0, MY_TOLERANCE);

    // Verify roots satisfy the equation x^3 - 6x^2 + 11x - 6 = 0
    verify_root(aRoot1, &[1.0, -6.0, 11.0, -6.0], 1, 1e-10);
    verify_root(aRoot2, &[1.0, -6.0, 11.0, -6.0], 2, 1e-10);
    verify_root(aRoot3, &[1.0, -6.0, 11.0, -6.0], 3, 1e-10);
}

#[test]
fn math_DirectPolynomialRootsTest__CubicOneRealRoot() {
    // Test cubic: x^3 + x + 1 = 0, should have one real root
    let aRoots = DirectPolynomialRoots::cubic(1.0, 0.0, 1.0, 1.0);

    assert!(aRoots.is_done());
    assert_eq!(aRoots.solutions().unwrap(), 1);

    let aRoot: f64 = aRoots.value(1).unwrap();
    // Verify the root by equation verification x^3 + x + 1 = 0
    verify_root(aRoot, &[1.0, 0.0, 1.0, 1.0], 1, 1e-10);
}
