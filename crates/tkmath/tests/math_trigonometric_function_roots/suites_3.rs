#[test]
fn math_TrigonometricFunctionRoots__NarrowBounds() {
    // Test x.cos() = 0 in range [0, PI/4]
    let c: f64 = 1.0;
    let d: f64 = 0.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::linear(c, d, e, 0.0, PI / 4.0);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    // No solutions expected in this narrow range
    assert_eq!(solver.solutions().unwrap(), 0);
}

#[test]
fn math_TrigonometricFunctionRoots__QuadraticTerms() {
    // Test cos^2(x) - 0.5 = 0 => cos^2(x) = 0.5 => x.cos() = +/-sqrt(0.5)
    let a: f64 = 1.0;
    let b: f64 = 0.0;
    let c: f64 = 0.0;
    let d: f64 = 0.0;
    let e: f64 = -0.5;
    let solver = TrigonometricRoots::new(a, b, c, d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() >= 2);

    if solver.solutions().unwrap() >= 1 {
        let x1: f64 = solver.value(1).unwrap();
        let cos_squared: f64 = x1.cos() * x1.cos();
        assert_near(cos_squared, 0.5, TOLERANCE);
    }
}

#[test]
fn math_TrigonometricFunctionRoots__MixedTerms() {
    // Test x.cos() + x.sin() - sqrt(2) = 0
    // This gives x.cos() + x.sin() = sqrt(2)
    // Which is sqrt(2)*sin(x + PI/4) = sqrt(2)
    // So sin(x + PI/4) = 1 => x + PI/4 = PI/2 => x = PI/4
    let c: f64 = 1.0;
    let d: f64 = 1.0;
    let e: f64 = -2.0_f64.sqrt();
    let solver = TrigonometricRoots::linear(c, d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() >= 1);

    if solver.solutions().unwrap() >= 1 {
        let x1: f64 = solver.value(1).unwrap();
        let result: f64 = x1.cos() + x1.sin();
        assert_near(result, 2.0_f64.sqrt(), TOLERANCE);
    }
}
