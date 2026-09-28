#[test]
fn MathNewtonFunctionRootTest__MultipleCubicRoots() {
    // Test finding different roots of cubic function with different starting points
    let mut aFunc = CubicWithDerivative;

    // Find root with different starting points - Newton's method converges to different roots
    let mut aSolver1 = NewtonFunctionRoot::new(&mut aFunc, 0.5, 1.0e-10, 1.0e-10);
    assert!(aSolver1.is_done());
    let aRoot1: f64 = aSolver1.root().unwrap();
    assert!(
        (aRoot1 - 1.0).abs() < 1.0e-6
            || (aRoot1 - 2.0).abs() < 1.0e-6
            || (aRoot1 - 3.0).abs() < 1.0e-6
    );

    // Find root with different starting point
    let mut aSolver2 = NewtonFunctionRoot::new(&mut aFunc, 2.8, 1.0e-10, 1.0e-10);
    assert!(aSolver2.is_done());
    let aRoot2: f64 = aSolver2.root().unwrap();
    assert!(
        (aRoot2 - 1.0).abs() < 1.0e-6
            || (aRoot2 - 2.0).abs() < 1.0e-6
            || (aRoot2 - 3.0).abs() < 1.0e-6
    );

    // Find root with third starting point
    let mut aSolver3 = NewtonFunctionRoot::new(&mut aFunc, 1.8, 1.0e-10, 1.0e-10);
    assert!(aSolver3.is_done());
    let aRoot3: f64 = aSolver3.root().unwrap();
    assert!(
        (aRoot3 - 1.0).abs() < 1.0e-6
            || (aRoot3 - 2.0).abs() < 1.0e-6
            || (aRoot3 - 3.0).abs() < 1.0e-6
    );
}

#[test]
fn MathNewtonFunctionRootTest__ConvergenceFromFarGuess() {
    // Test convergence from initial guess far from root
    let mut aFunc = QuadraticWithDerivative;

    let mut aSolver = NewtonFunctionRoot::new(&mut aFunc, 100.0, 1.0e-10, 1.0e-10); // Very far initial guess

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-8);
    assert!(aSolver.iterations().unwrap() > 5);
}
