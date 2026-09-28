#[test]
fn math_TrigonometricFunctionRoots__AllCoefficients() {
    // Test a more complex equation with all coefficients non-zero
    // a*cos^2(x) + 2*b*x.cos()*x.sin() + c*x.cos() + d*x.sin() + e = 0
    let a: f64 = 1.0;
    let b: f64 = 0.5;
    let c: f64 = 0.5;
    let d: f64 = 0.5;
    let e: f64 = -0.25;
    let solver = TrigonometricRoots::new(a, b, c, d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());

    // Check that found solutions are valid
    for i in 1..=solver.solutions().unwrap() {
        let x: f64 = solver.value(i).unwrap();
        let result: f64 =
            a * x.cos() * x.cos() + 2.0 * b * x.cos() * x.sin() + c * x.cos() + d * x.sin() + e;
        assert_near(result, 0.0, TOLERANCE);
    }
}

#[test]
fn math_TrigonometricFunctionRoots__LargeBounds() {
    // Test x.sin() = 0 over multiple periods
    let d: f64 = 1.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::sine(d, e, 0.0, 4.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() >= 2); // Should find multiple roots
}

#[test]
fn math_TrigonometricFunctionRoots__NegativeBounds() {
    // Test x.cos() = 0 with negative bounds
    let c: f64 = 1.0;
    let d: f64 = 0.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::linear(c, d, e, -PI, PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() >= 1);

    for i in 1..=solver.solutions().unwrap() {
        let x: f64 = solver.value(i).unwrap();
        assert!(x >= -PI);
        assert!(x <= PI);
        assert_near((x.cos()).abs(), 0.0, TOLERANCE);
    }
}
