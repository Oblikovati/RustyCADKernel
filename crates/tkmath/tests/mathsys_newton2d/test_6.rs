#[test]
fn MathSys_Newton2DTest__Regression_ArmijoUsesJtFDirectionalDerivative() {
    let mut aFunc: ArmijoRegressionFunc = ArmijoRegressionFunc;

    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.min = [-10.0, -10.0];
    aBounds.max = [10.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.enable_line_search = true;
    aOptions.solver.max_iterations = 10;

    let aResult: NewtonResultN<2> = solve_2d_symmetric(&mut aFunc, [0.0, 0.0], &aBounds, &aOptions);

    assert!(!(aResult.is_done()));
    assert_eq!(aResult.status, Status::NonDescentDirection);
}
