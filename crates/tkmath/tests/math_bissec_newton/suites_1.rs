#[test]
fn MathBissecNewtonTest__CubicRootFinding() {
    // Test finding root of cubic function x^3 - x - 2 = 0
    let mut aFunc = CubicWithDerivative;

    let mut aSolver = BissecNewton::new(1.0e-10);
    aSolver.perform(&mut aFunc, 1.0, 2.0, 100); // Root is approximately x = 1.521

    assert!(aSolver.is_done());
    let aRoot: f64 = aSolver.root().unwrap();
    assert!(aRoot > 1.52);
    assert!(aRoot < 1.53);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-8);
}

#[test]
fn MathBissecNewtonTest__SineFunctionRoot() {
    // Test finding root of sin(x) = 0 near PI
    let mut aFunc = SineWithDerivative;

    let mut aSolver = BissecNewton::new(1.0e-10);
    aSolver.perform(&mut aFunc, 3.0, 3.5, 100); // Find root near PI approximately 3.14159

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), std::f64::consts::PI, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-10);
    assert_near(aSolver.derivative().unwrap(), -1.0, 1.0e-8);
}

#[test]
fn MathBissecNewtonTest__CustomTolerance() {
    // Test with different tolerance values
    let mut aFunc = QuadraticWithDerivative;

    // Loose tolerance
    let mut aSolver1 = BissecNewton::new(1.0e-3);
    aSolver1.perform(&mut aFunc, 0.5, 1.5, 100);

    assert!(aSolver1.is_done());
    assert_near(aSolver1.root().unwrap(), 1.0, 1.0e-2);

    // Tight tolerance
    let mut aSolver2 = BissecNewton::new(1.0e-12);
    aSolver2.perform(&mut aFunc, 0.5, 1.5, 100);

    assert!(aSolver2.is_done());
    assert_near(aSolver2.root().unwrap(), 1.0, 1.0e-10);
}
