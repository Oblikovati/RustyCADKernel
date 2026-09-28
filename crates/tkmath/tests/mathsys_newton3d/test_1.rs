#[test]
fn MathSys_Newton3DTest__Solve3D_NonlinearSystem() {
    let mut aFunc = |x: &[f64; 3], theF: &mut [f64; 3], theJ: &mut [[f64; 3]; 3]| -> bool {
        let [theX1, theX2, theX3] = *x;
        theF[0] = theX1 * theX1 + theX2 * theX2 + theX3 * theX3 - 3.0;
        theF[1] = theX1 + theX2 + theX3 - 3.0;
        theF[2] = theX1 - theX2;

        theJ[0][0] = 2.0 * theX1;
        theJ[0][1] = 2.0 * theX2;
        theJ[0][2] = 2.0 * theX3;
        theJ[1][0] = 1.0;
        theJ[1][1] = 1.0;
        theJ[1][2] = 1.0;
        theJ[2][0] = 1.0;
        theJ[2][1] = -1.0;
        theJ[2][2] = 0.0;
        true
    };

    let mut aBounds: NewtonBoundsN<3> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-10;
    aOptions.solver.max_iterations = 50;

    let aResult: NewtonResultN<3> = solve_3d(&mut aFunc, [0.8, 0.8, 1.4], &aBounds, &aOptions);

    assert!(aResult.is_done());
    near(aResult.x[0], 1.0, 1.0e-5);
    near(aResult.x[1], 1.0, 1.0e-5);
    near(aResult.x[2], 1.0, 1.0e-5);
}
