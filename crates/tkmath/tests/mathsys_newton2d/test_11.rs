#[test]
fn MathSys_Newton2DTest__HeterogeneousUnboundedSystemTakesFullNewtonStep() {
    let mut aFunc = |x: &[f64; 2], theF: &mut [f64; 2], theJ: &mut [[f64; 2]; 2]| -> bool {
        let [theU, theV] = *x;
        theF[0] = 1.0e20 * (theU - 100.0);
        theF[1] = theV + 200.0;
        theJ[0][0] = 1.0e20;
        theJ[0][1] = 0.0;
        theJ[1][0] = 0.0;
        theJ[1][1] = 1.0;
        true
    };
    let mut aBounds: NewtonBoundsN<2> = NewtonBoundsN::default();
    aBounds.has_bounds = false;

    let aGeneral: NewtonResultN<2> =
        solve_2d(&mut aFunc, [0.0, 0.0], &aBounds, &NewtonOptions::default());
    assert!(aGeneral.is_done());
    assert_eq!(aGeneral.x[0], 100.0);
    assert_eq!(aGeneral.x[1], -200.0);

    let mut aSymmetricFunc: SymmetricHeterogeneousFunc = SymmetricHeterogeneousFunc;
    let aSymmetric: NewtonResultN<2> = solve_2d_symmetric(
        &mut aSymmetricFunc,
        [0.0, 0.0],
        &aBounds,
        &NewtonOptions::default(),
    );
    assert!(aSymmetric.is_done());
    assert_eq!(aSymmetric.x[0], 100.0);
    assert_eq!(aSymmetric.x[1], -200.0);
}
