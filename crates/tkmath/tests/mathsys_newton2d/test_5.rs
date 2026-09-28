#[test]
fn MathSys_Newton2DTest__Regression_TinyStepDoesNotConvergeWithLargeResidual() {
    let mut aFunc: HugeJacobianConstantResidual = HugeJacobianConstantResidual;

    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.min = [-1.0, -1.0];
    aBounds.max = [1.0, 1.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-8;
    aOptions.solver.x_tolerance = 1.0e-16;
    aOptions.solver.max_iterations = 10;

    let aResult: NewtonResultN<2> = solve_2d_symmetric(&mut aFunc, [0.0, 0.0], &aBounds, &aOptions);

    assert!(!(aResult.is_done()));
    assert_ne!(aResult.status, Status::Ok);
    assert!(aResult.residual_norm > 1.0e-2);
}
