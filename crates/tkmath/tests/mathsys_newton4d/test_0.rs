#[test]
fn MathSys_Newton4DTest__Solve4D_LinearSystem() {
    let mut aFunc = |x: &[f64; 4], theF: &mut [f64; 4], theJ: &mut [[f64; 4]; 4]| -> bool {
        let [theX1, theX2, theX3, theX4] = *x;
        theF[0] = theX1 - 1.0;
        theF[1] = theX2 - 2.0;
        theF[2] = theX3 - 3.0;
        theF[3] = theX4 - 4.0;

        theJ[0][0] = 1.0;
        theJ[0][1] = 0.0;
        theJ[0][2] = 0.0;
        theJ[0][3] = 0.0;

        theJ[1][0] = 0.0;
        theJ[1][1] = 1.0;
        theJ[1][2] = 0.0;
        theJ[1][3] = 0.0;

        theJ[2][0] = 0.0;
        theJ[2][1] = 0.0;
        theJ[2][2] = 1.0;
        theJ[2][3] = 0.0;

        theJ[3][0] = 0.0;
        theJ[3][1] = 0.0;
        theJ[3][2] = 0.0;
        theJ[3][3] = 1.0;
        true
    };

    let mut aBounds: NewtonBoundsN<4> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = THE_TOL;

    let aResult: NewtonResultN<4> = solve_4d(&mut aFunc, [0.0, 0.0, 0.0, 0.0], &aBounds, &aOptions);

    assert!(aResult.is_done());
    near(aResult.x[0], 1.0, 1.0e-12);
    near(aResult.x[1], 2.0, 1.0e-12);
    near(aResult.x[2], 3.0, 1.0e-12);
    near(aResult.x[3], 4.0, 1.0e-12);
}
