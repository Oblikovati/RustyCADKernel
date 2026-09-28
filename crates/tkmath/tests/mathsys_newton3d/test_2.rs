#[test]
fn MathSys_Newton3DTest__Solve3D_SmallStepAtRoot_ReturnsOK() {
    let mut aFunc = |x: &[f64; 3], theF: &mut [f64; 3], theJ: &mut [[f64; 3]; 3]| -> bool {
        let [theX1, theX2, theX3] = *x;
        theF[0] = theX1 - 1.0;
        theF[1] = theX2 - 2.0;
        theF[2] = theX3 - 3.0;

        theJ[0][0] = 1.0;
        theJ[0][1] = 0.0;
        theJ[0][2] = 0.0;
        theJ[1][0] = 0.0;
        theJ[1][1] = 1.0;
        theJ[1][2] = 0.0;
        theJ[2][0] = 0.0;
        theJ[2][1] = 0.0;
        theJ[2][2] = 1.0;
        true
    };

    let mut aBounds: NewtonBoundsN<3> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-12;
    aOptions.solver.x_tolerance = 100.0;
    aOptions.solver.max_iterations = 5;
    aOptions.max_step_ratio = 100.0;

    let aResult: NewtonResultN<3> = solve_3d(&mut aFunc, [0.0, 0.0, 0.0], &aBounds, &aOptions);
    assert!(aResult.is_done());
    assert_eq!(aResult.status, Status::Ok);
    near(aResult.x[0], 1.0, 1.0e-14);
    near(aResult.x[1], 2.0, 1.0e-14);
    near(aResult.x[2], 3.0, 1.0e-14);
}
