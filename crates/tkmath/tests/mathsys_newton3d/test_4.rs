#[test]
fn MathSys_Newton3DTest__Solve3D_Bounded() {
    let mut aFunc = |x: &[f64; 3], theF: &mut [f64; 3], theJ: &mut [[f64; 3]; 3]| -> bool {
        let [theX1, theX2, theX3] = *x;
        theF[0] = 2.0 * theX1 + theX2 + theX3 - 4.0;
        theF[1] = theX1 + 3.0 * theX2 + theX3 - 5.0;
        theF[2] = theX1 + theX2 + 4.0 * theX3 - 6.0;

        theJ[0][0] = 2.0;
        theJ[0][1] = 1.0;
        theJ[0][2] = 1.0;
        theJ[1][0] = 1.0;
        theJ[1][1] = 3.0;
        theJ[1][2] = 1.0;
        theJ[2][0] = 1.0;
        theJ[2][1] = 1.0;
        theJ[2][2] = 4.0;
        true
    };

    let mut aBounds: NewtonBoundsN<3> = NewtonBoundsN::default();
    aBounds.min = [-10.0, -10.0, -10.0];
    aBounds.max = [10.0, 10.0, 10.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = THE_TOL;

    let aResult: NewtonResultN<3> = solve_3d(&mut aFunc, [0.0, 0.0, 0.0], &aBounds, &aOptions);

    assert!(aResult.is_done());
    near(aResult.x[0], 1.0, 1.0e-8);
    near(aResult.x[1], 1.0, 1.0e-8);
    near(aResult.x[2], 1.0, 1.0e-8);
}
