#[test]
fn math_TrigonometricFunctionRoots__FullEquationBasic() {
    // Test a*cos^2(x) + 2*b*x.cos()*x.sin() + c*x.cos() + d*x.sin() + e = 0
    // Example: cos^2(x) - sin^2(x) = 0 => 2x.cos() = 0
    // a=1, b=0, c=0, d=0, e=-sin^2(x) equivalent to: cos^2(x) - sin^2(x) = 2x.cos() = 0
    // But let's use: cos^2(x) + c*x.cos() = 0 => x.cos()(x.cos() + c) = 0
    let a: f64 = 1.0;
    let b: f64 = 0.0;
    let c: f64 = 1.0;
    let d: f64 = 0.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::new(a, b, c, d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() > 0);
}

#[test]
fn math_TrigonometricFunctionRoots__LinearSineOnly() {
    // Test d*x.sin() + e = 0 => x.sin() = -e/d
    // Example: x.sin() - 0.5 = 0 => x.sin() = 0.5 => x = PI/6, 5*PI/6
    let d: f64 = 1.0;
    let e: f64 = -0.5;
    let solver = TrigonometricRoots::sine(d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() > 0);

    if solver.solutions().unwrap() >= 1 {
        let x1: f64 = solver.value(1).unwrap();
        let sin_val: f64 = x1.sin();
        assert_near(sin_val, 0.5, TOLERANCE);
    }
}
