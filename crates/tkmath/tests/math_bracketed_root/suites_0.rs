#[test]
fn MathBracketedRootTest__QuadraticRootFinding() {
    // Test finding root of quadratic function between x = 0 and x = 1.5
    let mut aFunc = QuadraticFunction;

    let aSolver = BracketedRoot::new(&mut aFunc, 0.0, 1.5, 1.0e-10);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 1.0, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-9);
    assert!(aSolver.iterations().unwrap() > 0);
}

#[test]
fn MathBracketedRootTest__QuadraticSecondRoot() {
    // Test finding the second root of quadratic function between x = 2.5 and x = 4
    let mut aFunc = QuadraticFunction;

    let aSolver = BracketedRoot::new(&mut aFunc, 2.5, 4.0, 1.0e-10);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 3.0, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-9);
}
