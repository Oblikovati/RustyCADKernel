// Direct legacy declarations; see parent for OCCT license and provenance.
#[test]
fn MathPSOTest__CustomIterationCount() {
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
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 20, 10).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 2).unwrap();

    aSolver
        .perform_with_iterations(&aSteps, &mut aValue, &mut aSolution, 5)
        .unwrap();

    assert!(aSolution[1] >= -2.0 && aSolution[1] <= 4.0);
    assert!(aSolution[2] >= -1.0 && aSolution[2] <= 5.0);

    aSolver
        .perform_with_iterations(&aSteps, &mut aValue, &mut aSolution, 100)
        .unwrap();

    assert!(aSolution[1] >= -2.0 && aSolution[1] <= 4.0);
    assert!(aSolution[2] >= -1.0 && aSolution[2] <= 5.0);
}
#[test]
fn MathPSOTest__RosenbrockOptimization() {
    let mut aFunc = RosenbrockFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 2.0;
    aUpperBorder[2] = 3.0;

    let mut aSteps = Vector::with_bounds(1, 2).unwrap();
    aSteps[1] = 0.1;
    aSteps[2] = 0.1;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 40, 100).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 2).unwrap();

    aSolver
        .perform(&aSteps, &mut aValue, &mut aSolution)
        .unwrap();

    assert!(aSolution[1] >= -2.0 && aSolution[1] <= 2.0);
    assert!(aSolution[2] >= -1.0 && aSolution[2] <= 3.0);
    assert!(aValue < 100.0);
}
#[test]
fn MathPSOTest__MultiModalOptimization() {
    let mut aFunc = MultiModalFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = 0.0;
    aLowerBorder[2] = 0.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 2.0 * PI;
    aUpperBorder[2] = 2.0 * PI;

    let mut aSteps = Vector::with_bounds(1, 2).unwrap();
    aSteps[1] = 0.2;
    aSteps[2] = 0.2;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 30, 50).unwrap();

    let mut aValue: f64 = 0.0;
    let mut aSolution = Vector::with_bounds(1, 2).unwrap();

    aSolver
        .perform(&aSteps, &mut aValue, &mut aSolution)
        .unwrap();

    assert!(aSolution[1] >= 0.0 && aSolution[1] <= 2.0 * PI);
    assert!(aSolution[2] >= 0.0 && aSolution[2] <= 2.0 * PI);
    assert!(aValue < 0.0);
}
#[test]
fn MathPSOTest__DifferentStepSizes() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 4.0;
    aUpperBorder[2] = 5.0;

    let mut aLargeSteps = Vector::with_bounds(1, 2).unwrap();
    aLargeSteps[1] = 0.5;
    aLargeSteps[2] = 0.5;

    let mut aSolver1 = Pso::with_options(
        &mut aFunc,
        &aLowerBorder,
        &aUpperBorder,
        &aLargeSteps,
        15,
        20,
    )
    .unwrap();

    let mut aValue1: f64 = 0.0;
    let mut aSolution1 = Vector::with_bounds(1, 2).unwrap();

    aSolver1
        .perform(&aLargeSteps, &mut aValue1, &mut aSolution1)
        .unwrap();

    assert!(aSolution1[1] >= -2.0 && aSolution1[1] <= 4.0);
    assert!(aSolution1[2] >= -1.0 && aSolution1[2] <= 5.0);

    let mut aSmallSteps = Vector::with_bounds(1, 2).unwrap();
    aSmallSteps[1] = 0.05;
    aSmallSteps[2] = 0.05;

    let mut aSolver2 = Pso::with_options(
        &mut aFunc,
        &aLowerBorder,
        &aUpperBorder,
        &aSmallSteps,
        15,
        20,
    )
    .unwrap();

    let mut aValue2: f64 = 0.0;
    let mut aSolution2 = Vector::with_bounds(1, 2).unwrap();

    aSolver2
        .perform(&aSmallSteps, &mut aValue2, &mut aSolution2)
        .unwrap();

    assert!(aSolution2[1] >= -2.0 && aSolution2[1] <= 4.0);
    assert!(aSolution2[2] >= -1.0 && aSolution2[2] <= 5.0);
}
