#[test]
fn MathSys_Newton2DTest__Solve2D_ScaleRelativeJacobianAndFiniteValidation() {
    let mut aScaled = |x: &[f64; 2], theF: &mut [f64; 2], theJ: &mut [[f64; 2]; 2]| -> bool {
        let [theU, theV] = *x;
        const THE_SCALE: f64 = 1.0e-150;
        theF[0] = THE_SCALE * (theU - 1.0);
        theF[1] = THE_SCALE * (theV + 2.0);
        theJ[0][0] = THE_SCALE;
        theJ[0][1] = 0.0;
        theJ[1][0] = 0.0;
        theJ[1][1] = THE_SCALE;
        true
    };
    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.has_bounds = false;
    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-200;
    aOptions.max_step_ratio = 10.0;
    let aResult: NewtonResultN<2> = solve_2d(&mut aScaled, [0.0, 0.0], &aBounds, &aOptions);
    assert!(aResult.is_done());
    near(aResult.x[0], 1.0, 1.0e-12);
    near(aResult.x[1], -2.0, 1.0e-12);

    let mut aNaN = |x: &[f64; 2], theF: &mut [f64; 2], theJ: &mut [[f64; 2]; 2]| -> bool {
        let [_, _] = *x;
        theF[0] = f64::NAN;
        theF[1] = 0.0;
        theJ[1][1] = 1.0;
        theJ[0][0] = 1.0;
        theJ[1][0] = 0.0;
        theJ[0][1] = 0.0;
        true
    };
    assert_eq!(
        solve_2d(&mut aNaN, [0.0, 0.0], &aBounds, &NewtonOptions::default()).status,
        Status::NumericalError
    );
}
