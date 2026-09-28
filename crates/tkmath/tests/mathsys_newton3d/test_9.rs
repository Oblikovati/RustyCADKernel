#[test]
fn MathSys_Newton3DTest__HeterogeneousUnboundedSystemTakesFullNewtonStep() {
    let mut aFunc = |x: &[f64; 3], theF: &mut [f64; 3], theJ: &mut [[f64; 3]; 3]| -> bool {
        let [theX, theY, theZ] = *x;
        let aX: [f64; 3] = [theX, theY, theZ];
        let aTarget: [f64; 3] = [100.0, -200.0, 300.0];
        let aScale: [f64; 3] = [1.0e20, 1.0, 1.0];
        for aRow in 0..3 {
            theF[aRow] = aScale[aRow] * (aX[aRow] - aTarget[aRow]);
            for aCol in 0..3 {
                theJ[aRow][aCol] = if aRow == aCol { aScale[aRow] } else { 0.0 };
            }
        }
        true
    };
    let mut aBounds: NewtonBoundsN<3> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let aResult: NewtonResultN<3> = solve_3d(
        &mut aFunc,
        [0.0, 0.0, 0.0],
        &aBounds,
        &NewtonOptions::default(),
    );
    assert!(aResult.is_done());
    assert_eq!(aResult.x[0], 100.0);
    assert_eq!(aResult.x[1], -200.0);
    assert_eq!(aResult.x[2], 300.0);
}
