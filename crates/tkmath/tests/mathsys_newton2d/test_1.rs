#[test]
fn MathSys_Newton2DTest__Solve2D_SmallStepAtRoot_ReturnsOK() {
    let mut aFunc: GenericLinearExactStep = GenericLinearExactStep;

    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.min = [-10.0, -10.0];
    aBounds.max = [10.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-12;
    aOptions.solver.x_tolerance = 100.0;
    aOptions.solver.max_iterations = 5;

    let aResult: NewtonResultN<2> = solve_2d(&mut aFunc, [0.0, 0.0], &aBounds, &aOptions);
    assert!(aResult.is_done());
    assert_eq!(aResult.status, Status::Ok);
    near(aResult.x[0], 1.0, 1.0e-14);
    near(aResult.x[1], 2.0, 1.0e-14);
}
