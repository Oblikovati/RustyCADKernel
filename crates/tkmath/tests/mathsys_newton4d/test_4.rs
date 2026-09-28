#[test]
fn MathSys_Newton4DTest__Solve4D_InvalidInput() {
    let mut aFunc = |x: &[f64; 4], theF: &mut [f64; 4], theJ: &mut [[f64; 4]; 4]| -> bool {
        let [theX1, theX2, theX3, theX4] = *x;
        for i in 0..4 {
            theF[i] = 0.0;
            for j in 0..4 {
                theJ[i][j] = if i == j { 1.0 } else { 0.0 };
            }
        }
        true
    };

    let mut aBounds: NewtonBoundsN<4> = NewtonBoundsN::default();
    aBounds.min = [1.0, 0.0, 0.0, 0.0];
    aBounds.max = [0.0, 1.0, 1.0, 1.0];

    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = THE_TOL;

    let aResult: NewtonResultN<4> = solve_4d(&mut aFunc, [0.0, 0.0, 0.0, 0.0], &aBounds, &aOptions);

    assert_eq!(aResult.status, Status::InvalidInput);
    assert!(!(aResult.is_done()));
}
