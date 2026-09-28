// Direct OCCT legacy declarations; license in parent file.
#[test]
fn MathGlobOptMinTest__SetGlobalParamsMethod() {
    let mut aFunc1 = QuadraticFunction;
    let mut aFunc2 = LinearFunction;

    let mut aLowerBorder1 = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder1[1] = -2.0;
    aLowerBorder1[2] = -1.0;

    let mut aUpperBorder1 = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder1[1] = 3.0;
    aUpperBorder1[2] = 4.0;

    let mut aSolver = GlobalOptimum::new(
        GlobalObjective::Value(&mut aFunc1),
        &aLowerBorder1,
        &aUpperBorder1,
    )
    .unwrap();
    aSolver.perform(true).unwrap();

    assert!(aSolver.is_done());

    let mut aLowerBorder2 = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder2[1] = 0.0;
    aLowerBorder2[2] = 0.0;

    let mut aUpperBorder2 = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder2[1] = 1.0;
    aUpperBorder2[2] = 1.0;

    aSolver
        .set_global_params(
            GlobalObjective::Value(&mut aFunc2),
            &aLowerBorder2,
            &aUpperBorder2,
            9.0,
            1e-2,
            1e-7,
        )
        .unwrap();
    aSolver.perform(true).unwrap();

    assert!(aSolver.is_done());
}
#[test]
fn MathGlobOptMinTest__MultipleExtremaAccess() {
    let mut aFunc = MultiModalFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -2.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 2.0;
    aUpperBorder[2] = 2.0;

    let mut aSolver = GlobalOptimum::new(
        GlobalObjective::Value(&mut aFunc),
        &aLowerBorder,
        &aUpperBorder,
    )
    .unwrap();
    aSolver.perform(false).unwrap();

    assert!(aSolver.is_done());
    let aNbSol: i32 = aSolver.number_of_extrema();
    assert!(aNbSol > 0);

    for i in 1..=aNbSol {
        let mut aSol = Vector::with_bounds(1, 2).unwrap();
        assert!(aSolver.points(i, &mut aSol).is_ok());
        assert!(aSol[1] >= -2.0 && aSol[1] <= 2.0);
        assert!(aSol[2] >= -2.0 && aSol[2] <= 2.0);
    }
}
#[test]
fn MathGlobOptMinTest__SmallSearchSpace() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = 0.99;
    aLowerBorder[2] = 1.99;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 1.01;
    aUpperBorder[2] = 2.01;

    let mut aSolver = GlobalOptimum::with_options(
        GlobalObjective::Value(&mut aFunc),
        &aLowerBorder,
        &aUpperBorder,
        9.0,
        1.0e-3,
        1.0e-8,
    )
    .unwrap();
    aSolver.perform(true).unwrap();

    assert!(aSolver.is_done());

    let mut aSol = Vector::with_bounds(1, 2).unwrap();
    aSolver.points(1, &mut aSol).unwrap();
    near(aSol[1], 1.0, 0.02);
    near(aSol[2], 2.0, 0.02);
}
