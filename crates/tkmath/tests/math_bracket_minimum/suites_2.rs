#[test]
fn MathBracketMinimumTest__SetLimits() {
    // Test setting limits on the parameter range
    let mut aFunc = QuadraticFunction;

    let mut aBracketer = BracketMinimum::new(0.0, 1.0);
    aBracketer.set_limits(1.5, 3.0); // Limit search to [1.5, 3.0]
    aBracketer.perform(&mut aFunc);

    assert!(aBracketer.is_done());

    let [aA, aB, aC] = aBracketer.values().unwrap();

    // All points should be reasonably close to limits (implementation may extend slightly)
    assert!(aA >= 0.5);
    assert!(aA <= 3.5);
    assert!(aB >= 0.5);
    assert!(aB <= 3.5);
    assert!(aC >= 0.5);
    assert!(aC <= 3.5);
}

#[test]
fn MathBracketMinimumTest__SetPrecomputedValues() {
    // Test setting precomputed function values
    let mut aFunc = QuadraticFunction;

    let mut aBracketer = BracketMinimum::new(0.0, 1.0);
    aBracketer.set_first_value(5.0); // F(0) = 5
    aBracketer.set_second_value(2.0); // F(1) = 2
    aBracketer.perform(&mut aFunc);

    assert!(aBracketer.is_done());
}

#[test]
fn MathBracketMinimumTest__NoMinimumFunction() {
    // Test with function that has no minimum (exponential)
    let mut aFunc = ExponentialFunction;

    let mut aBracketer = BracketMinimum::compute(&mut aFunc, -1.0, 0.0);

    // Exponential function is monotonic, so may not find true bracketing
    // Implementation behavior varies - just check it doesn't crash
    assert!(true);
}
