#[test]
fn MathSys_Newton3DTest__Solve3D_ScaleRelativeJacobian() {
    let mut aFunc = |x: &[f64; 3], theF: &mut [f64; 3], theJ: &mut [[f64; 3]; 3]| -> bool {
        let [theX, theY, theZ] = *x;
        const THE_SCALE: f64 = 1.0e-150;
        theF[0] = THE_SCALE * (theX - 1.0);
        theF[1] = THE_SCALE * (theY - 2.0);
        theF[2] = THE_SCALE * (theZ - 3.0);
        for aRow in 0..3 {
            for aCol in 0..3 {
                theJ[aRow][aCol] = if aRow == aCol { THE_SCALE } else { 0.0 };
            }
        }
        true
    };
    let mut aBounds: NewtonBoundsN<3> = NewtonBoundsN::default();
    aBounds.has_bounds = false;
    let mut aOptions: NewtonOptions = NewtonOptions::default();
    aOptions.solver.f_tolerance = 1.0e-200;
    aOptions.max_step_ratio = 10.0;
    let aResult: NewtonResultN<3> = solve_3d(&mut aFunc, [0.0, 0.0, 0.0], &aBounds, &aOptions);
    assert!(aResult.is_done());
    near(aResult.x[0], 1.0, 1.0e-12);
    near(aResult.x[1], 2.0, 1.0e-12);
    near(aResult.x[2], 3.0, 1.0e-12);
}
