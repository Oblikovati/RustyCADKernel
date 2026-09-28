#[test]
fn MathSys_Newton3DTest__Solve3D_HeterogeneousFullRankJacobian() {
    let mut aFunc = |x: &[f64; 3], theF: &mut [f64; 3], theJ: &mut [[f64; 3]; 3]| -> bool {
        let [theX, theY, theZ] = *x;
        const THE_SMALL_SCALE: f64 = 1.0e-7;
        theF[0] = theX - 1.0;
        theF[1] = THE_SMALL_SCALE * (theY - 2.0);
        theF[2] = THE_SMALL_SCALE * (theZ - 3.0);
        for aRow in 0..3 {
            for aCol in 0..3 {
                theJ[aRow][aCol] = 0.0;
            }
        }
        theJ[0][0] = 1.0;
        theJ[1][1] = THE_SMALL_SCALE;
        theJ[2][2] = THE_SMALL_SCALE;
        true
    };
    let mut aBounds: NewtonBoundsN<3> = NewtonBoundsN::default();
    aBounds.has_bounds = false;
    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-14;
    aOptions.max_step_ratio = 10.0;
    let aResult: NewtonResultN<3> = solve_3d(&mut aFunc, [0.0, 0.0, 0.0], &aBounds, &aOptions);
    assert!(aResult.is_done());
    near(aResult.x[0], 1.0, 1.0e-12);
    near(aResult.x[1], 2.0, 1.0e-12);
    near(aResult.x[2], 3.0, 1.0e-12);
}
