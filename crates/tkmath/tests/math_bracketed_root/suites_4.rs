#[test]
fn MathBracketedRootTest__NotDoneState() {
    // Test state handling for incomplete calculations
    // Create solver with invalid bounds to force failure
    let mut aFunc = QuadraticFunction;
    let aSolver = BracketedRoot::new(&mut aFunc, 3.5, 4.0, 1.0e-10); // No root in interval

    assert!(!aSolver.is_done());
}

#[test]
fn MathBracketedRootTest__HighPrecisionRequirement() {
    // Test with extremely tight tolerance
    let mut aFunc = SineFunction;
    let aSolver = BracketedRoot::with_options(&mut aFunc, 3.0, 3.5, 1.0e-15, 200, 1e-12);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), std::f64::consts::PI, 1.0e-10);
    assert!(aSolver.iterations().unwrap() > 5);
}

#[test]
fn MathBracketedRootTest__CustomZEPS() {
    // Test with custom ZEPS parameter (machine epsilon)
    let mut aFunc = QuadraticFunction;

    let aSolver = BracketedRoot::with_options(&mut aFunc, 0.5, 1.5, 1.0e-10, 100, 1.0e-15);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 1.0, 1.0e-8);
}
