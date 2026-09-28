#[test]
fn MathBissecNewtonTest__QuadraticRootFinding() {
    // Test finding roots of quadratic function (x-2)^2 - 1 = 0
    // Roots are at x = 1 and x = 3
    let mut aFunc = QuadraticWithDerivative;

    let mut aSolver = BissecNewton::new(1.0e-10);
    aSolver.perform(&mut aFunc, 0.0, 1.5, 100); // Find root near x = 1

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 1.0, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), -2.0, 1.0e-8);
}

#[test]
fn MathBissecNewtonTest__QuadraticSecondRoot() {
    // Test finding the second root of the same quadratic function
    let mut aFunc = QuadraticWithDerivative;

    let mut aSolver = BissecNewton::new(1.0e-10);
    aSolver.perform(&mut aFunc, 2.5, 4.0, 100); // Find root near x = 3

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 3.0, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), 2.0, 1.0e-8);
}
