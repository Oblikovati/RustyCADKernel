#[test]
fn MathSys_LM_Test__DifferentStartingPoints() {
    let mut aFunc: CircleHyperbola = CircleHyperbola;

    let aStarts: [[f64; 2]; 5] = [[1.5, 0.5], [2.0, 0.5], [1.0, 1.0], [2.5, 0.5], [0.5, 2.0]];

    for aStartPt in aStarts {
        let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
        aStart[1] = aStartPt[0];
        aStart[2] = aStartPt[1];

        let mut aConfig: LMConfig = LMConfig::default();
        aConfig.solver.tolerance = 1.0e-10;
        aConfig.solver.f_tolerance = 1.0e-10;
        aConfig.solver.max_iterations = 100;

        let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

        assert!(aResult.is_done());

        let aX: f64 = *aResult.solution.as_ref().unwrap().at(0).unwrap();
        let aY: f64 = *aResult.solution.as_ref().unwrap().at(1).unwrap();

        near(aX * aX + aY * aY, 4.0, THE_TOLERANCE);
        near(aX * aY, 1.0, THE_TOLERANCE);
    }
}

#[test]
fn MathSys_LM_Test__ConvergenceIterations() {
    let mut aFunc: LinearSystem2D = LinearSystem2D;

    let mut aStart: Vector = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;

    let mut aConfig: LMConfig = LMConfig::default();
    aConfig.solver.tolerance = 1.0e-12;
    aConfig.solver.f_tolerance = 1.0e-12;
    aConfig.solver.max_iterations = 100;

    let mut aResult: SystemResult = levenberg_marquardt(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());

    assert!(aResult.iterations < 20);

    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        2.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
}
