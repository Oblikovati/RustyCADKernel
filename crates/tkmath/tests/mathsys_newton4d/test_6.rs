#[test]
fn MathSys_Newton4DTest__Solve4D_ScaleRelativeJacobian() {
    let mut aFunc = |x: &[f64; 4], theF: &mut [f64; 4], theJ: &mut [[f64; 4]; 4]| -> bool {
        let [theX1, theX2, theX3, theX4] = *x;
        const THE_SCALE: f64 = 1.0e150;
        let aX: [f64; 4] = [theX1, theX2, theX3, theX4];
        for aRow in 0..4 {
            theF[aRow] = THE_SCALE * (aX[aRow] - (aRow + 1) as f64);
            for aCol in 0..4 {
                theJ[aRow][aCol] = if aRow == aCol { THE_SCALE } else { 0.0 };
            }
        }
        true
    };
    let mut aBounds: NewtonBoundsN<4> = NewtonBoundsN::default();
    aBounds.has_bounds = false;
    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-12;
    aOptions.max_step_ratio = 10.0;
    let aResult: NewtonResultN<4> = solve_4d(&mut aFunc, [0.0, 0.0, 0.0, 0.0], &aBounds, &aOptions);
    assert!(aResult.is_done());
    for anIndex in 0..4 {
        near(aResult.x[anIndex], (anIndex + 1) as f64, 1.0e-12);
    }
}
