#[test]
fn MathBracketedRootTest__ZeroAtBoundary() {
    // Test when the root is exactly at one of the boundaries
    let mut aFunc = LinearFunction; // f(x) = 2x - 4, root at x = 2

    let aSolver = BracketedRoot::new(&mut aFunc, 2.0, 3.0, 1.0e-10);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-12);
}

#[test]
fn MathBracketedRootTest__VeryNarrowBounds() {
    // Test with very narrow bracketing interval
    let mut aFunc = LinearFunction;

    let aSolver = BracketedRoot::new(&mut aFunc, 1.999, 2.001, 1.0e-10);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-8);
}

#[test]
fn MathBracketedRootTest__ReverseBounds() {
    // Test with bounds in reverse order (Bound2 < Bound1)
    let mut aFunc = QuadraticFunction;

    let aSolver = BracketedRoot::new(&mut aFunc, 1.5, 0.0, 1.0e-10); // Reversed bounds

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 1.0, 1.0e-8);
}
