#[test]
fn MathSys_Newton2DTest__Solve2DSymmetric_InvalidInput() {
    let mut aFunc: SymmetricDistFunc = SymmetricDistFunc {
        myTargetU: 0.0,
        myTargetV: 0.0,
    };

    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.min = [10.0, 0.0];
    aBounds.max = [0.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-10;

    let aResult: NewtonResultN<2> = solve_2d_symmetric(&mut aFunc, [0.0, 0.0], &aBounds, &aOptions);

    assert_eq!(aResult.status, Status::InvalidInput);
    assert!(!(aResult.is_done()));
}
