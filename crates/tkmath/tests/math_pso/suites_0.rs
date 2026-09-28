// Direct legacy declarations; see parent for OCCT license and provenance.
#[test]
fn MathPSOTest__QuadraticFunctionOptimization() {
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
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 20, 50).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 2).unwrap();

    aSolver
        .perform(&aSteps, &mut aValue, &mut aSolution)
        .unwrap();

    near(aSolution[1], 1.0, 0.5);
    near(aSolution[2], 2.0, 0.5);
    assert!(aValue < 1.0);
}
#[test]
fn MathPSOTest__OneDimensionalOptimization() {
    let mut aFunc = Quadratic1DFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 1).unwrap();
    aLowerBorder[1] = 0.0;

    let mut aUpperBorder = Vector::with_bounds(1, 1).unwrap();
    aUpperBorder[1] = 6.0;

    let mut aSteps = Vector::with_bounds(1, 1).unwrap();
    aSteps[1] = 0.1;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 15, 30).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 1).unwrap();

    aSolver
        .perform(&aSteps, &mut aValue, &mut aSolution)
        .unwrap();

    near(aSolution[1], 3.0, 0.5);
    assert!(aValue < 0.5);
}
#[test]
fn MathPSOTest__ThreeDimensionalOptimization() {
    let mut aFunc = Quadratic3DFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 3).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -2.0;
    aLowerBorder[3] = -2.0;

    let mut aUpperBorder = Vector::with_bounds(1, 3).unwrap();
    aUpperBorder[1] = 2.0;
    aUpperBorder[2] = 2.0;
    aUpperBorder[3] = 2.0;

    let mut aSteps = Vector::with_bounds(1, 3).unwrap();
    aSteps[1] = 0.1;
    aSteps[2] = 0.1;
    aSteps[3] = 0.1;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 25, 40).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 3).unwrap();

    aSolver
        .perform(&aSteps, &mut aValue, &mut aSolution)
        .unwrap();

    near(aSolution[1], 0.0, 0.5);
    near(aSolution[2], 0.0, 0.5);
    near(aSolution[3], 0.0, 0.5);
    assert!(aValue < 1.0);
}
#[test]
fn MathPSOTest__CustomParticleCount() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 4.0;
    aUpperBorder[2] = 5.0;

    let mut aSteps = Vector::with_bounds(1, 2).unwrap();
    aSteps[1] = 0.2;
    aSteps[2] = 0.2;

    let mut aSolver1 =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 5, 20).unwrap();

    let mut aValue1: f64 = 0.0;
    let mut aSolution1 = Vector::with_bounds(1, 2).unwrap();

    aSolver1
        .perform(&aSteps, &mut aValue1, &mut aSolution1)
        .unwrap();

    assert!(aSolution1[1] >= -2.0 && aSolution1[1] <= 4.0);
    assert!(aSolution1[2] >= -1.0 && aSolution1[2] <= 5.0);

    let mut aSolver2 =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 50, 30).unwrap();

    let mut aValue2: f64 = 0.0;
    let mut aSolution2 = Vector::with_bounds(1, 2).unwrap();

    aSolver2
        .perform(&aSteps, &mut aValue2, &mut aSolution2)
        .unwrap();

    assert!(aSolution2[1] >= -2.0 && aSolution2[1] <= 4.0);
    assert!(aSolution2[2] >= -1.0 && aSolution2[2] <= 5.0);
}
