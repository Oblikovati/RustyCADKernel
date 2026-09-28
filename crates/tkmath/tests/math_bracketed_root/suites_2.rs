#[test]
fn MathBracketedRootTest__CustomTolerance() {
    // Test with different tolerance values
    let mut aFunc = QuadraticFunction;

    // Loose tolerance
    let aSolver1 = BracketedRoot::new(&mut aFunc, 0.5, 1.5, 1.0e-3);
    assert!(aSolver1.is_done());
    assert_near(aSolver1.root().unwrap(), 1.0, 1.0e-2);

    // Tight tolerance
    let aSolver2 = BracketedRoot::new(&mut aFunc, 0.5, 1.5, 1.0e-12);
    assert!(aSolver2.is_done());
    assert_near(aSolver2.root().unwrap(), 1.0, 1.0e-10);
}

#[test]
fn MathBracketedRootTest__CustomIterationLimit() {
    // Test with custom iteration limits
    let mut aFunc = CubicFunction;

    // Very few iterations
    let aSolver1 = BracketedRoot::with_options(&mut aFunc, 1.0, 2.0, 1.0e-12, 5, 1e-12);
    if aSolver1.is_done() {
        assert!(aSolver1.iterations().unwrap() <= 5);
    }

    // Many iterations
    let aSolver2 = BracketedRoot::with_options(&mut aFunc, 1.0, 2.0, 1.0e-15, 200, 1e-12);
    assert!(aSolver2.is_done());
}

#[test]
fn MathBracketedRootTest__InvalidBounds() {
    // Test with bounds that don't bracket a root (same sign function values)
    let mut aFunc = QuadraticFunction; // f(x) = (x-2)^2 - 1

    // Both bounds give positive function values: f(3.5) > 0, f(4) > 0
    let aSolver = BracketedRoot::new(&mut aFunc, 3.5, 4.0, 1.0e-10);

    assert!(!aSolver.is_done());
}
