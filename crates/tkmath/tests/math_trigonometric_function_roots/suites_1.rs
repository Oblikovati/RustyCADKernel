#[test]
fn math_TrigonometricFunctionRoots__LinearCosineAndSine() {
    // Test c*x.cos() + d*x.sin() + e = 0
    // Example: x.cos() + x.sin() = 0 => tan(x) = -1 => x = 3*PI/4, 7*PI/4
    let c: f64 = 1.0;
    let d: f64 = 1.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::linear(c, d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() > 0);

    if solver.solutions().unwrap() >= 1 {
        let x1: f64 = solver.value(1).unwrap();
        let result: f64 = x1.cos() + x1.sin();
        assert_near(result, 0.0, TOLERANCE);
    }
}

#[test]
fn math_TrigonometricFunctionRoots__PureCosineEquation() {
    // Test x.cos() = 0 => x = PI/2, 3*PI/2
    let c: f64 = 1.0;
    let d: f64 = 0.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::linear(c, d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() >= 1);

    if solver.solutions().unwrap() >= 1 {
        let x1: f64 = solver.value(1).unwrap();
        assert_near((x1.cos()).abs(), 0.0, TOLERANCE);
    }
}

#[test]
fn math_TrigonometricFunctionRoots__PureSineEquation() {
    // Test x.sin() = 0 => x = 0, PI, 2*PI
    let d: f64 = 1.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::sine(d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() >= 2);

    if solver.solutions().unwrap() >= 1 {
        let x1: f64 = solver.value(1).unwrap();
        assert_near((x1.sin()).abs(), 0.0, TOLERANCE);
    }
}
