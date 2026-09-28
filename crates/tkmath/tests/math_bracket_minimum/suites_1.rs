#[test]
fn MathBracketMinimumTest__ConstructorWithOnePrecomputedValue() {
    // Test constructor with one precomputed function value
    let mut aFunc = QuadraticFunction;

    let aA: f64 = 0.0;
    let aB: f64 = 1.0;
    let aFA: f64 = 5.0; // F(0) = 5

    let mut aBracketer = BracketMinimum::with_first_value(&mut aFunc, aA, aB, aFA);

    assert!(aBracketer.is_done());
}

#[test]
fn MathBracketMinimumTest__QuarticFunctionBracketing() {
    // Test with quartic function that has minimum at x = 1
    let mut aFunc = QuarticFunction;

    let mut aBracketer = BracketMinimum::compute(&mut aFunc, 0.0, 0.5);

    assert!(aBracketer.is_done());

    let [aA, aB, aC] = aBracketer.values().unwrap();

    // The minimum should be bracketed around x = 1
    assert!(aB > 0.5);
    assert!(aB < 1.5);
}

#[test]
fn MathBracketMinimumTest__CosineFunction() {
    // Test with cosine function which has minimum at PI
    let mut aFunc = CosineFunction;

    let mut aBracketer = BracketMinimum::compute(&mut aFunc, 2.0, 4.0);

    assert!(aBracketer.is_done());

    let [aA, aB, aC] = aBracketer.values().unwrap();

    // The minimum should be bracketed around PI approximately 3.14159
    assert!(aB > 2.5);
    assert!(aB < 4.5);

    let [aFA, aFB, aFC] = aBracketer.function_values().unwrap();

    assert!(aFB < aFA);
    assert!(aFB < aFC);
}
