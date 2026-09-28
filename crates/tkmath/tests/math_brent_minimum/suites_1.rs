#[test]
fn MathBrentMinimumTest__CosineMinimumFinding() {
    // Test with cosine function, minimum at PI
    let mut aFunc = CosineFunction;

    let mut aSolver = BrentMinimum::new(1.0e-10);
    aSolver.perform(&mut aFunc, 2.5, 3.1, 4.0); // Bracketing triplet around minimum at PI

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), std::f64::consts::PI, 1.0e-8);
    assert_near(aSolver.minimum().unwrap(), -1.0, 1.0e-10);
}

#[test]
fn MathBrentMinimumTest__ConstructorWithKnownValue() {
    // Test constructor when F(Bx) is known
    let mut aFunc = QuadraticFunction;
    let Bx: f64 = 1.5;
    let Fbx: f64 = (Bx - 2.0) * (Bx - 2.0) + 1.0; // F(1.5) = 1.25

    let mut aSolver = BrentMinimum::with_value(1.0e-10, Fbx, 100, 1e-12);
    aSolver.perform(&mut aFunc, 0.0, Bx, 4.0);

    assert!(aSolver.is_done());
    assert_near(aSolver.location().unwrap(), 2.0, 1.0e-8);
}

#[test]
fn MathBrentMinimumTest__CustomTolerance() {
    // Test with different tolerance values
    let mut aFunc = QuadraticFunction;

    // Loose tolerance
    let mut aSolver1 = BrentMinimum::new(1.0e-3);
    aSolver1.perform(&mut aFunc, 0.0, 1.5, 4.0);

    assert!(aSolver1.is_done());
    assert_near(aSolver1.location().unwrap(), 2.0, 1.0e-2);

    // Tight tolerance
    let mut aSolver2 = BrentMinimum::new(1.0e-12);
    aSolver2.perform(&mut aFunc, 0.0, 1.5, 4.0);

    assert!(aSolver2.is_done());
    assert_near(aSolver2.location().unwrap(), 2.0, 1.0e-8);
}
