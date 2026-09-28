#[test]
fn MathPoly_ComparisonTest__Quartic_FourDistinctRoots() {
    let aA: f64 = 1.0;
    let aB: f64 = -10.0;
    let aC: f64 = 35.0;
    let aD: f64 = -50.0;
    let aE: f64 = 24.0;

    let anOldSolver = DirectPolynomialRoots::quartic(aA, aB, aC, aD, aE);
    let aNewResult = quartic(aA, aB, aC, aD, aE);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    compare_roots(
        old_roots(&anOldSolver),
        new_roots(&aNewResult),
        THE_TOLERANCE,
    );
}

#[test]
fn MathPoly_ComparisonTest__Quartic_TwoRealRoots() {
    let aA: f64 = 1.0;
    let aB: f64 = 0.0;
    let aC: f64 = 0.0;
    let aD: f64 = 0.0;
    let aE: f64 = -1.0;

    let anOldSolver = DirectPolynomialRoots::quartic(aA, aB, aC, aD, aE);
    let aNewResult = quartic(aA, aB, aC, aD, aE);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    assert_eq!(anOldSolver.solutions().unwrap(), 2);
    assert_eq!(aNewResult.nb_roots, 2);

    compare_roots(
        old_roots(&anOldSolver),
        new_roots(&aNewResult),
        THE_TOLERANCE,
    );
}

#[test]
fn MathPoly_ComparisonTest__Quartic_Biquadratic() {
    let aA: f64 = 1.0;
    let aB: f64 = 0.0;
    let aC: f64 = -5.0;
    let aD: f64 = 0.0;
    let aE: f64 = 4.0;

    let anOldSolver = DirectPolynomialRoots::quartic(aA, aB, aC, aD, aE);
    let aNewResult = quartic(aA, aB, aC, aD, aE);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    compare_roots(
        old_roots(&anOldSolver),
        new_roots(&aNewResult),
        THE_TOLERANCE,
    );
}
