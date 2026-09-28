// Direct OCCT legacy declarations; license in parent file.
#[test]
fn MathGlobOptMinTest__AllSolutionsSearch() {
    let mut aFunc = MultiModalFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -3.0;
    aLowerBorder[2] = -3.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 3.0;
    aUpperBorder[2] = 3.0;

    let mut aSolver = GlobalOptimum::new(
        GlobalObjective::Value(&mut aFunc),
        &aLowerBorder,
        &aUpperBorder,
    )
    .unwrap();
    aSolver.perform(false).unwrap();

    assert!(aSolver.is_done());
    assert!(aSolver.number_of_extrema() >= 1);
}
#[test]
fn MathGlobOptMinTest__CustomTolerances() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 3.0;
    aUpperBorder[2] = 4.0;

    let mut aSolver = GlobalOptimum::with_options(
        GlobalObjective::Value(&mut aFunc),
        &aLowerBorder,
        &aUpperBorder,
        9.0,
        1.0e-3,
        1.0e-8,
    )
    .unwrap();

    let mut aDiscTol: f64 = 0.0;
    let mut aSameTol: f64 = 0.0;
    [aDiscTol, aSameTol] = aSolver.tolerances();
    near(aDiscTol, 1.0e-3, 1.0e-12);
    near(aSameTol, 1.0e-8, 1.0e-12);

    aSolver.set_tolerances(1.0e-2, 1.0e-6);
    [aDiscTol, aSameTol] = aSolver.tolerances();
    near(aDiscTol, 1.0e-2, 1.0e-12);
    near(aSameTol, 1.0e-6, 1.0e-12);

    aSolver.perform(true).unwrap();
    assert!(aSolver.is_done());
}
#[test]
fn MathGlobOptMinTest__LocalParamsReduction() {
    let mut aFunc = QuadraticFunction;

    let mut aGlobalLower = Vector::with_bounds(1, 2).unwrap();
    aGlobalLower[1] = -5.0;
    aGlobalLower[2] = -5.0;

    let mut aGlobalUpper = Vector::with_bounds(1, 2).unwrap();
    aGlobalUpper[1] = 5.0;
    aGlobalUpper[2] = 5.0;

    let mut aSolver = GlobalOptimum::new(
        GlobalObjective::Value(&mut aFunc),
        &aGlobalLower,
        &aGlobalUpper,
    )
    .unwrap();

    let mut aLocalLower = Vector::with_bounds(1, 2).unwrap();
    aLocalLower[1] = 0.0;
    aLocalLower[2] = 1.0;

    let mut aLocalUpper = Vector::with_bounds(1, 2).unwrap();
    aLocalUpper[1] = 2.0;
    aLocalUpper[2] = 3.0;

    aSolver
        .set_local_params(&aLocalLower, &aLocalUpper)
        .unwrap();
    aSolver.perform(true).unwrap();

    assert!(aSolver.is_done());

    let mut aSol = Vector::with_bounds(1, 2).unwrap();
    aSolver.points(1, &mut aSol).unwrap();
    assert!(aSol[1] >= 0.0 && aSol[1] <= 2.0);
    assert!(aSol[2] >= 1.0 && aSol[2] <= 3.0);
}
#[test]
fn MathGlobOptMinTest__ContinuitySettings() {
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

    let aDefaultCont: i32 = aSolver.continuity();
    assert!(aDefaultCont >= 0);

    aSolver.set_continuity(1);
    assert_eq!(aSolver.continuity(), 1);

    aSolver.set_continuity(2);
    assert_eq!(aSolver.continuity(), 2);

    aSolver.perform(true).unwrap();
    assert!(aSolver.is_done());
}
