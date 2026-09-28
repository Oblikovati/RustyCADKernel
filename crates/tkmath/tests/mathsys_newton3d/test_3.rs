#[test]
fn MathSys_Newton3DTest__Solve3D_TinyStepLargeResidual_ReturnsMaxIterations() {
    let mut aFunc = |x: &[f64; 3], theF: &mut [f64; 3], theJ: &mut [[f64; 3]; 3]| -> bool {
        let [theX1, theX2, theX3] = *x;
        theF[0] = 1.0;
        theF[1] = 1.0;
        theF[2] = 1.0;
        theJ[0][0] = 1.0e20;
        theJ[0][1] = 0.0;
        theJ[0][2] = 0.0;
        theJ[1][0] = 0.0;
        theJ[1][1] = 1.0e20;
        theJ[1][2] = 0.0;
        theJ[2][0] = 0.0;
        theJ[2][1] = 0.0;
        theJ[2][2] = 1.0e20;
        true
    };

    let mut aBounds: NewtonBoundsN<3> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-8;
    aOptions.solver.x_tolerance = 1.0e-16;
    aOptions.solver.max_iterations = 10;
    aOptions.enable_line_search = false;

    let aResult: NewtonResultN<3> = solve_3d(&mut aFunc, [0.0, 0.0, 0.0], &aBounds, &aOptions);
    assert!(!(aResult.is_done()));
    assert_eq!(aResult.status, Status::MaxIterations);
    assert!(aResult.residual_norm > 1.0e-2);
}
