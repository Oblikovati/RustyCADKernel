#[test]
fn MathSys_Newton4DTest__Solve4D_TinyStepLargeResidual_ReturnsMaxIterations() {
    let mut aFunc = |x: &[f64; 4], theF: &mut [f64; 4], theJ: &mut [[f64; 4]; 4]| -> bool {
        let [theX1, theX2, theX3, theX4] = *x;
        theF[0] = 1.0;
        theF[1] = 1.0;
        theF[2] = 1.0;
        theF[3] = 1.0;

        for r in 0..4 {
            for c in 0..4 {
                theJ[r][c] = if r == c { 1.0e20 } else { 0.0 };
            }
        }
        true
    };

    let mut aBounds: NewtonBoundsN<4> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-8;
    aOptions.solver.x_tolerance = 1.0e-16;
    aOptions.solver.max_iterations = 10;
    aOptions.enable_line_search = false;

    let aResult: NewtonResultN<4> = solve_4d(&mut aFunc, [0.0, 0.0, 0.0, 0.0], &aBounds, &aOptions);
    assert!(!(aResult.is_done()));
    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.residual_norm > 1.0e-2);
}
