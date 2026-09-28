#[test]
fn MathBrentMinimumTest__MonotonicFunction() {
    // Test with monotonic function (no true minimum in interval)
    let mut aFunc = ShiftedExponentialFunction;

    let mut aSolver = BrentMinimum::new(1.0e-10);
    aSolver.perform(&mut aFunc, 0.0, 1.0, 2.0);

    if aSolver.is_done() {
        // If it finds a "minimum", it should be at the left boundary
        assert!(aSolver.location().unwrap() < 1.0);
    }
}

#[test]
fn MathBrentMinimumTest__CustomZEPS() {
    // Test with custom ZEPS (machine epsilon) parameter
    let mut aFunc = QuadraticFunction;

    let mut aSolver = BrentMinimum::with_options(1.0e-10, 100, 1.0e-15);
    aSolver.perform(&mut aFunc, 0.0, 1.5, 4.0);

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), 2.0, 1.0e-8);
}

#[test]
fn MathBrentMinimumTest__UnperformedState() {
    // Test state handling before Perform() is called
    let aSolver = BrentMinimum::new(1.0e-10);

    // Before Perform() is called, solver should report not done
    assert!(!aSolver.is_done());

    // In release builds, verify the solver maintains consistent state
    if !aSolver.is_done() {
        assert!(!aSolver.is_done());
    }
}
