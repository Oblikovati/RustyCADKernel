#[test]
fn MathSys_Newton2DTest__Solve2DSymmetric_Target_Converges() {
    let mut aFunc: SymmetricDistFunc = SymmetricDistFunc {
        myTargetU: 3.5,
        myTargetV: 7.2,
    };

    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.min = [-10.0, -10.0];
    aBounds.max = [10.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-10;
    aOptions.solver.max_iterations = 50;

    let aResult: NewtonResultN<2> = solve_2d_symmetric(&mut aFunc, [0.0, 0.0], &aBounds, &aOptions);

    assert!(aResult.is_done());
    assert_eq!(aResult.status, Status::Ok);
    near(aResult.x[0], 3.5, 1.0e-9);
    near(aResult.x[1], 7.2, 1.0e-9);
    assert!(aResult.residual_norm < 1.0e-10);
}
