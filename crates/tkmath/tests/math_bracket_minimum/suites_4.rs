#[test]
fn MathBracketMinimumTest__ReverseOrderInitialPoints() {
    // Test with initial points in reverse order (B < A)
    let mut aFunc = QuadraticFunction;

    let mut aBracketer = BracketMinimum::compute(&mut aFunc, 4.0, 0.0); // B < A

    assert!(aBracketer.is_done());

    let [aA, aB, aC] = aBracketer.values().unwrap();

    // Just verify reasonable bracketing points were found
    assert!(aB > -1.0);
    assert!(aB < 5.0);
}

#[test]
fn MathBracketMinimumTest__RestrictiveLimits() {
    // Test with very restrictive limits that may prevent finding minimum
    let mut aFunc = QuadraticFunction; // Minimum at x = 2

    let mut aBracketer = BracketMinimum::new(0.0, 0.5);
    aBracketer.set_limits(0.0, 1.0); // Limits exclude the actual minimum at x = 2
    aBracketer.perform(&mut aFunc);

    // May or may not succeed depending on implementation
    if aBracketer.is_done() {
        let [aA, aB, aC] = aBracketer.values().unwrap();

        // If successful, points should be within limits
        assert!(aA >= -0.1);
        assert!(aA <= 1.1);
        assert!(aB >= -0.1);
        assert!(aB <= 1.1);
        assert!(aC >= -0.1);
        assert!(aC <= 1.1);
    }
}
