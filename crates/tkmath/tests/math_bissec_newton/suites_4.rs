#[test]
fn MathBissecNewtonTest__EdgeCaseBounds() {
    // Test with bounds very close to the root
    let mut aFunc = QuadraticWithDerivative;
    let mut aSolver = BissecNewton::new(1.0e-10);

    aSolver.perform(&mut aFunc, 0.99, 1.01, 100); // Very narrow bounds around x = 1

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 1.0, 1.0e-8);
}
