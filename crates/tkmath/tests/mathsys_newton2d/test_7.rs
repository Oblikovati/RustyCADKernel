#[test]
fn MathSys_Newton2DTest__Solve2DSymmetric_HighPrecision() {
    let mut aFunc: SymmetricDistFunc = SymmetricDistFunc {
        myTargetU: 5.123456789,
        myTargetV: 2.987654321,
    };

    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.min = [-10.0, -10.0];
    aBounds.max = [10.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 5.0e-8;
    aOptions.solver.max_iterations = 100;

    let aResult: NewtonResultN<2> = solve_2d_symmetric(&mut aFunc, [0.0, 0.0], &aBounds, &aOptions);

    assert!(aResult.is_done());
    near(aResult.x[0], 5.123456789, 1.0e-7);
    near(aResult.x[1], 2.987654321, 1.0e-7);
    assert!(aResult.residual_norm < 5.0e-8);
}
