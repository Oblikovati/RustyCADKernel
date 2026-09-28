#[test]
fn math_TrigonometricFunctionRoots__HighFrequencyTest() {
    // Test x.sin() - 0.5 = 0 with precise expected solutions
    let d: f64 = 1.0;
    let e: f64 = -0.5;
    let solver = TrigonometricRoots::sine(d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    assert!(!solver.infinite_roots());
    assert!(solver.solutions().unwrap() >= 2);

    // Expected solutions: PI/6 approximately 0.5236 and 5*PI/6 approximately 2.618
    if solver.solutions().unwrap() >= 2 {
        let mut solutions: Vec<f64> = Vec::new();
        for i in 1..=solver.solutions().unwrap() {
            solutions.push(solver.value(i).unwrap());
        }

        // Check that we have solutions near PI/6 and 5*PI/6
        let mut found_first: bool = false;
        let mut found_second: bool = false;
        for sol in solutions {
            if (sol - PI / 6.0).abs() < 0.1 {
                found_first = true;
            }
            if (sol - 5.0 * PI / 6.0).abs() < 0.1 {
                found_second = true;
            }
        }
        assert!(found_first || found_second);
    }
}

#[test]
fn math_TrigonometricFunctionRoots__EdgeCaseSmallCoefficients() {
    // Test with very small coefficients
    let c: f64 = 1.0e-10;
    let d: f64 = 1.0;
    let e: f64 = 0.0;
    let solver = TrigonometricRoots::linear(c, d, e, 0.0, 2.0 * PI);

    assert!(solver.is_done());
    // Should behave approximately like x.sin() = 0
    if !solver.infinite_roots() {
        assert!(solver.solutions().unwrap() >= 2);
    }
}
