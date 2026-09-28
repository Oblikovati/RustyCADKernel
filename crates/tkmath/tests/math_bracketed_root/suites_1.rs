#[test]
fn MathBracketedRootTest__CubicRootFinding() {
    // Test finding root of cubic function x^3 - x - 2 = 0
    let mut aFunc = CubicFunction;

    let aSolver = BracketedRoot::new(&mut aFunc, 1.0, 2.0, 1.0e-10);

    assert!(aSolver.is_done());
    let aRoot: f64 = aSolver.root().unwrap();
    assert!(aRoot > 1.52);
    assert!(aRoot < 1.53);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-8);
}

#[test]
fn MathBracketedRootTest__SineFunctionRoot() {
    // Test finding root of sin(x) = 0 near PI
    let mut aFunc = SineFunction;

    let aSolver = BracketedRoot::new(&mut aFunc, 3.0, 3.5, 1.0e-10);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), std::f64::consts::PI, 1.0e-8);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-9);
}

#[test]
fn MathBracketedRootTest__LinearFunctionRoot() {
    // Test finding root of linear function 2x - 4 = 0
    let mut aFunc = LinearFunction;

    let aSolver = BracketedRoot::new(&mut aFunc, 1.0, 3.0, 1.0e-10);

    assert!(aSolver.is_done());
    assert_near(aSolver.root().unwrap(), 2.0, 1.0e-12);
    assert_near(aSolver.value().unwrap(), 0.0, 1.0e-12);
}
