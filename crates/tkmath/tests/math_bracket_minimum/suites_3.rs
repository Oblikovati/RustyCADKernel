#[test]
fn MathBracketMinimumTest__MultipleLocalMinima() {
    // Test with function having multiple local minima
    let mut aFunc = MultipleMinFunction;

    let mut aBracketer = BracketMinimum::compute(&mut aFunc, -2.0, 0.0);

    assert!(aBracketer.is_done());

    let [aFA, aFB, aFC] = aBracketer.function_values().unwrap();

    assert!(aFB < aFA);
    assert!(aFB < aFC);
}

#[test]
fn MathBracketMinimumTest__UnperformedState() {
    // Test state handling for incomplete calculations
    let mut aBracketer = BracketMinimum::new(0.0, 1.0);

    assert!(!aBracketer.is_done());
}

#[test]
fn MathBracketMinimumTest__VeryNarrowInitialBounds() {
    // Test with very close initial points
    let mut aFunc = QuadraticFunction;

    let mut aBracketer = BracketMinimum::compute(&mut aFunc, 1.99, 2.01); // Very close to minimum

    assert!(aBracketer.is_done());

    let [aA, aB, aC] = aBracketer.values().unwrap();

    // When very close to minimum, bracketing may be approximate
    // Just verify we get reasonable results
    assert!(aA > 1.5);
    assert!(aC < 2.5);
}
