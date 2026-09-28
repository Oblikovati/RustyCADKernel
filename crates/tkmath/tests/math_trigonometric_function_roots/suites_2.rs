#[test]
fn math_TrigonometricFunctionRoots__NoSolution() {
    // Test x.sin() + 2 = 0 => x.sin() = -2 (impossible)
    let d: f64 = 1.0;
    let e: f64 = 2.0;
    let solver = TrigonometricRoots::sine(d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert_eq!(solver.solutions().unwrap(), 0);
}

#[test]
fn math_TrigonometricFunctionRoots__InfiniteSolutions() {
    // Test 0*x.sin() + 0 = 0 (always true)
    let d: f64 = 0.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::sine(d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(solver.infinite_roots());
}

#[test]
fn math_TrigonometricFunctionRoots__CustomBounds() {
    // Test x.sin() = 0 in range [PI/2, 3*PI/2]
    let d: f64 = 1.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::sine(d, e, PI / 2.0, 3.0 * PI / 2.0);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() >= 1);

    for i in 1..=solver.solutions().unwrap() {
        let x: f64 = solver.value(i).unwrap();
        assert!(x >= PI / 2.0);
        assert!(x <= 3.0 * PI / 2.0);
        assert_near((x.sin()).abs(), 0.0, TOLERANCE);
    }
}
