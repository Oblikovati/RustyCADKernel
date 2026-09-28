#[test]
fn MathSys_Newton3DTest__Solve3D_InvalidInput() {
    let mut aFunc = |x: &[f64; 3], theF: &mut [f64; 3], theJ: &mut [[f64; 3]; 3]| -> bool {
        let [theX1, theX2, theX3] = *x;
        theF[0] = 0.0;
        theF[1] = 0.0;
        theF[2] = 0.0;
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
    aBounds.min = [1.0, 0.0, 0.0];
    aBounds.max = [0.0, 1.0, 1.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = THE_TOL;

    let aResult: NewtonResultN<3> = solve_3d(&mut aFunc, [0.0, 0.0, 0.0], &aBounds, &aOptions);

    assert_eq!(aResult.status, Status::InvalidInput);
    assert!(!(aResult.is_done()));
}
