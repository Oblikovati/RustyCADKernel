#[test]
fn MathSys_Newton4DTest__HeterogeneousUnboundedSystemTakesFullNewtonStep() {
    let mut aFunc = |x: &[f64; 4], theF: &mut [f64; 4], theJ: &mut [[f64; 4]; 4]| -> bool {
        let [theX0, theX1, theX2, theX3] = *x;
        let aX: [f64; 4] = [theX0, theX1, theX2, theX3];
        let aTarget: [f64; 4] = [100.0, -200.0, 300.0, -400.0];
        let aScale: [f64; 4] = [1.0e20, 1.0, 1.0, 1.0];
        for aRow in 0..4 {
            theF[aRow] = aScale[aRow] * (aX[aRow] - aTarget[aRow]);
            for aCol in 0..4 {
                theJ[aRow][aCol] = if aRow == aCol { aScale[aRow] } else { 0.0 };
            }
        }
        true
    };
    let mut aBounds: NewtonBoundsN<4> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let aResult: NewtonResultN<4> = solve_4d(
        &mut aFunc,
        [0.0, 0.0, 0.0, 0.0],
        &aBounds,
        &NewtonOptions::default(),
    );
    assert!(aResult.is_done());
    assert_eq!(aResult.x[0], 100.0);
    assert_eq!(aResult.x[1], -200.0);
    assert_eq!(aResult.x[2], 300.0);
    assert_eq!(aResult.x[3], -400.0);
}
