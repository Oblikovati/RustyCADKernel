// Direct OCCT legacy declarations; license in parent file.
#[test]
fn MathGlobOptMinTest__FunctionalMinimalValue() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 3.0;
    aUpperBorder[2] = 4.0;

    let mut aSolver = GlobalOptimum::new(
        GlobalObjective::Value(&mut aFunc),
        &aLowerBorder,
        &aUpperBorder,
    )
    .unwrap();

    let aDefaultMin: f64 = aSolver.functional_minimal_value();
    assert_eq!(aDefaultMin, -2e100);

    aSolver.set_functional_minimal_value(-1.0);
    near(aSolver.functional_minimal_value(), -1.0, 1.0e-12);

    aSolver.perform(true).unwrap();
    assert!(aSolver.is_done());
}
#[test]
fn MathGlobOptMinTest__LipschitzConstantState() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 3.0;
    aUpperBorder[2] = 4.0;

    let mut aSolver = GlobalOptimum::new(
        GlobalObjective::Value(&mut aFunc),
        &aLowerBorder,
        &aUpperBorder,
    )
    .unwrap();

    let aDefaultState: bool = aSolver.lipschitz_locked();
    assert!(!(aDefaultState));

    aSolver.set_lipschitz_locked(true);
    assert!(aSolver.lipschitz_locked());

    aSolver.set_lipschitz_locked(false);
    assert!(!(aSolver.lipschitz_locked()));

    aSolver.perform(true).unwrap();
    assert!(aSolver.is_done());
}
#[test]
fn MathGlobOptMinTest__RosenbrockOptimization() {
    let mut aFunc = RosenbrockFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 2.0;
    aUpperBorder[2] = 3.0;

    let mut aSolver = GlobalOptimum::with_options(
        GlobalObjective::Value(&mut aFunc),
        &aLowerBorder,
        &aUpperBorder,
        50.0,
        1.0e-2,
        1.0e-6,
    )
    .unwrap();
    aSolver.perform(true).unwrap();

    assert!(aSolver.is_done());
    assert!(aSolver.number_of_extrema() > 0);

    let mut aSol = Vector::with_bounds(1, 2).unwrap();
    aSolver.points(1, &mut aSol).unwrap();

    assert!(aSol[1] >= -2.0 && aSol[1] <= 2.0);
    assert!(aSol[2] >= -1.0 && aSol[2] <= 3.0);
}
#[test]
fn MathGlobOptMinTest__LinearFunctionOptimization() {
    let mut aFunc = LinearFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = 0.0;
    aLowerBorder[2] = 0.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 2.0;
    aUpperBorder[2] = 2.0;

    let mut aSolver = GlobalOptimum::new(
        GlobalObjective::Value(&mut aFunc),
        &aLowerBorder,
        &aUpperBorder,
    )
    .unwrap();
    aSolver.perform(true).unwrap();

    assert!(aSolver.is_done());
    assert!(aSolver.number_of_extrema() > 0);

    let mut aSol = Vector::with_bounds(1, 2).unwrap();
    aSolver.points(1, &mut aSol).unwrap();

    near(aSol[1], 0.0, 1.0e-1);
    near(aSol[2], 0.0, 1.0e-1);
}
