#[test]
fn MathBrentMinimumTest__QuadraticMinimumFinding() {
    // Test finding minimum of quadratic function
    let mut aFunc = QuadraticFunction;

    let mut aSolver = BrentMinimum::new(1.0e-10);
    aSolver.perform(&mut aFunc, 0.0, 1.5, 4.0); // Bracketing triplet around minimum at x=2

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), 2.0, 1.0e-8);
    assert_near(aSolver.minimum().unwrap(), 1.0, 1.0e-10);
    assert!(aSolver.iterations().unwrap() > 0);
}

#[test]
fn MathBrentMinimumTest__QuarticMinimumFinding() {
    // Test with quartic function that has flat minimum
    let mut aFunc = QuarticFunction;

    let mut aSolver = BrentMinimum::new(1.0e-10);
    aSolver.perform(&mut aFunc, 0.0, 0.8, 2.0); // Bracketing triplet around minimum at x=1

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), 1.0, 1.0e-4);
    assert_near(aSolver.minimum().unwrap(), 2.0, 1.0e-6);
}
