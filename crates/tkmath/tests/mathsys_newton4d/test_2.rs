#[test]
fn MathSys_Newton4DTest__Solve4D_SmallStepAtRoot_ReturnsOK() {
    let mut aFunc = |x: &[f64; 4], theF: &mut [f64; 4], theJ: &mut [[f64; 4]; 4]| -> bool {
        let [theX1, theX2, theX3, theX4] = *x;
        theF[0] = theX1 - 1.0;
        theF[1] = theX2 - 2.0;
        theF[2] = theX3 - 3.0;
        theF[3] = theX4 - 4.0;

        for r in 0..4 {
            for c in 0..4 {
                theJ[r][c] = if r == c { 1.0 } else { 0.0 };
            }
        }
        true
    };

    let mut aBounds: NewtonBoundsN<4> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-12;
    aOptions.solver.x_tolerance = 100.0;
    aOptions.solver.max_iterations = 5;
    aOptions.max_step_ratio = 100.0;

    let aResult: NewtonResultN<4> = solve_4d(&mut aFunc, [0.0, 0.0, 0.0, 0.0], &aBounds, &aOptions);
    assert!(aResult.is_done());
    assert_eq!(aResult.status, Status::Ok);
    near(aResult.x[0], 1.0, 1.0e-14);
    near(aResult.x[1], 2.0, 1.0e-14);
    near(aResult.x[2], 3.0, 1.0e-14);
    near(aResult.x[3], 4.0, 1.0e-14);
}
