// Direct legacy declarations; see parent for OCCT license and provenance.
#[test]
fn MathPSOTest__PSOParticlesPoolIntegration() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 4.0;
    aUpperBorder[2] = 5.0;

    let mut aSteps = Vector::with_bounds(1, 2).unwrap();
    aSteps[1] = 0.1;
    aSteps[2] = 0.1;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 20, 30).unwrap();

    let aNbParticles: i32 = 20;
    let mut aParticlesPool = PsoParticlesPool::new(aNbParticles, 2).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 2).unwrap();

    aSolver
        .perform_particles(
            &mut aParticlesPool,
            aNbParticles,
            &mut aValue,
            &mut aSolution,
        )
        .unwrap();

    assert!(aSolution[1] >= -2.0 && aSolution[1] <= 4.0);
    assert!(aSolution[2] >= -1.0 && aSolution[2] <= 5.0);
}
#[test]
fn MathPSOTest__SmallSearchSpace() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = 0.8;
    aLowerBorder[2] = 1.8;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 1.2;
    aUpperBorder[2] = 2.2;

    let mut aSteps = Vector::with_bounds(1, 2).unwrap();
    aSteps[1] = 0.05;
    aSteps[2] = 0.05;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 10, 20).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 2).unwrap();

    aSolver
        .perform(&aSteps, &mut aValue, &mut aSolution)
        .unwrap();

    near(aSolution[1], 1.0, 0.3);
    near(aSolution[2], 2.0, 0.3);
    assert!(aValue < 0.5);
}
#[test]
fn MathPSOTest__AsymmetricBounds() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -5.0;
    aLowerBorder[2] = 1.5;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 2.0;
    aUpperBorder[2] = 10.0;

    let mut aSteps = Vector::with_bounds(1, 2).unwrap();
    aSteps[1] = 0.2;
    aSteps[2] = 0.2;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 25, 40).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 2).unwrap();

    aSolver
        .perform(&aSteps, &mut aValue, &mut aSolution)
        .unwrap();

    assert!(aSolution[1] >= -5.0 && aSolution[1] <= 2.0);
    assert!(aSolution[2] >= 1.5 && aSolution[2] <= 10.0);
    near(aSolution[1], 1.0, 1.5);
    near(aSolution[2], 2.0, 2.0);
}
#[test]
fn MathPSOTest__MinimalConfiguration() {
    let mut aFunc = Quadratic1DFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 1).unwrap();
    aLowerBorder[1] = 0.0;

    let mut aUpperBorder = Vector::with_bounds(1, 1).unwrap();
    aUpperBorder[1] = 6.0;

    let mut aSteps = Vector::with_bounds(1, 1).unwrap();
    aSteps[1] = 0.5;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 3, 5).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 1).unwrap();

    aSolver
        .perform_with_iterations(&aSteps, &mut aValue, &mut aSolution, 3)
        .unwrap();

    assert!(aSolution[1] >= 0.0 && aSolution[1] <= 6.0);
}
