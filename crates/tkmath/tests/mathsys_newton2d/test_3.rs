#[test]
fn MathSys_Newton2DTest__Solve2D_LineSearchRejectsIncreasingFullStep() {
    let mut aFunc = |x: &[f64; 2], theF: &mut [f64; 2], theJ: &mut [[f64; 2]; 2]| -> bool {
        let [theU, theV] = *x;
        theF[0] = (theU).atan();
        theF[1] = theV;
        theJ[0][0] = 1.0 / (1.0 + theU * theU);
        theJ[0][1] = 0.0;
        theJ[1][0] = 0.0;
        theJ[1][1] = 1.0;
        true
    };

    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.has_bounds = false;
    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-12;
    aOptions.solver.x_tolerance = 1.0e-16;
    aOptions.solver.max_iterations = 1;
    aOptions.max_step_ratio = 10.0;

    aOptions.enable_line_search = false;
    let aFullStep: NewtonResultN<2> = solve_2d(&mut aFunc, [2.0, 0.0], &aBounds, &aOptions);
    aOptions.enable_line_search = true;
    let aBacktracked: NewtonResultN<2> = solve_2d(&mut aFunc, [2.0, 0.0], &aBounds, &aOptions);

    let anInitialResidual: f64 = (2.0_f64).atan();
    assert!(aFullStep.residual_norm > anInitialResidual);
    assert!(aBacktracked.residual_norm < anInitialResidual);
    assert_ne!(aBacktracked.x[0], aFullStep.x[0]);
}
