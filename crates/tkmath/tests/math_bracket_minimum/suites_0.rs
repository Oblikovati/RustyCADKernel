#[test]
fn MathBracketMinimumTest__QuadraticMinimumBracketing() {
    // Test bracketing minimum of quadratic function
    let mut aFunc = QuadraticFunction;

    let mut aBracketer = BracketMinimum::compute(&mut aFunc, 0.0, 1.0);

    assert!(aBracketer.is_done());

    let [aA, aB, aC] = aBracketer.values().unwrap();

    // Check that B is between A and C
    assert!((aA < aB && aB < aC) || (aC < aB && aB < aA));

    // Check that the minimum is around x = 2
    assert!(aB > 1.0);
    assert!(aB < 3.0);

    let [aFA, aFB, aFC] = aBracketer.function_values().unwrap();

    // Check that F(B) is less than both F(A) and F(C)
    assert!(aFB < aFA);
    assert!(aFB < aFC);
}

#[test]
fn MathBracketMinimumTest__ConstructorWithPrecomputedValues() {
    // Test constructor with precomputed function values
    let mut aFunc = QuadraticFunction;

    let aA: f64 = 0.0;
    let aB: f64 = 1.0;
    let aFA: f64 = (aA - 2.0) * (aA - 2.0) + 1.0; // F(0) = 5
    let aFB: f64 = (aB - 2.0) * (aB - 2.0) + 1.0; // F(1) = 2

    let mut aBracketer = BracketMinimum::with_values(&mut aFunc, aA, aB, aFA, aFB);

    assert!(aBracketer.is_done());

    let [aRetA, aRetB, aRetC] = aBracketer.values().unwrap();

    assert!((aRetA < aRetB && aRetB < aRetC) || (aRetC < aRetB && aRetB < aRetA));
}
