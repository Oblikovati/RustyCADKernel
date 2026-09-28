#[test]
fn MathBrentMinimumTest__CustomIterationLimit() {
    // Test with custom iteration limits
    let mut aFunc = RosenbrockSliceFunction; // More challenging function

    // Few iterations
    let mut aSolver1 = BrentMinimum::with_options(1.0e-10, 5, 1e-12);
    aSolver1.perform(&mut aFunc, 0.0, 0.5, 2.0);

    if aSolver1.is_done() {
        assert!(aSolver1.iterations().unwrap() <= 5);
    }

    // Many iterations
    let mut aSolver2 = BrentMinimum::with_options(1.0e-12, 200, 1e-12);
    aSolver2.perform(&mut aFunc, 0.0, 0.5, 2.0);

    assert!(aSolver2.is_done());
}

#[test]
fn MathBrentMinimumTest__InvalidBracketingTriplet() {
    // Test with invalid bracketing (Bx not between Ax and Cx)
    let mut aFunc = QuadraticFunction;

    let mut aSolver = BrentMinimum::new(1.0e-10);
    aSolver.perform(&mut aFunc, 0.0, 4.0, 1.5); // Bx > Cx, invalid bracketing

    // Implementation may handle this gracefully or fail
    if aSolver.is_done() {
        // If it succeeds, the result should still be reasonable
        assert!(aSolver.location().unwrap() > 0.0);
        assert!(aSolver.location().unwrap() < 5.0);
    }
}

#[test]
fn MathBrentMinimumTest__FlatFunction() {
    // Test with a very flat function around minimum
    let mut aFunc = QuarticFunction; // Has very flat minimum at x=1

    let mut aSolver = BrentMinimum::new(1.0e-12);
    aSolver.perform(&mut aFunc, 0.99, 1.0, 1.01); // Very narrow bracketing

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), 1.0, 1.0e-8);
}
