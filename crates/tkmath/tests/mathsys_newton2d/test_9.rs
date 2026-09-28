#[test]
fn MathSys_Newton2DTest__Solve2DSymmetric_ConsistentFromDifferentStarts() {
    let mut aFunc: SymmetricDistFunc = SymmetricDistFunc {
        myTargetU: 5.0,
        myTargetV: 5.0,
    };

    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.min = [0.0, 0.0];
    aBounds.max = [10.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-10;
    aOptions.solver.max_iterations = 50;

    let aStarts: [[f64; 2]; 4] = [[0.0, 0.0], [10.0, 10.0], [2.0, 8.0], [8.0, 2.0]];

    for aStart in aStarts {
        let aResult: NewtonResultN<2> = solve_2d_symmetric(&mut aFunc, aStart, &aBounds, &aOptions);
        assert!(aResult.is_done());
        near(aResult.x[0], 5.0, 1.0e-8);
        near(aResult.x[1], 5.0, 1.0e-8);
    }
}
