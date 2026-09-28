#[test]
fn MathPoly_ComparisonTest__Cubic_SmallCoefficients() {
    let aA: f64 = 0.001;
    let aB: f64 = -0.006;
    let aC: f64 = 0.011;
    let aD: f64 = -0.006;

    let anOldSolver = DirectPolynomialRoots::cubic(aA, aB, aC, aD);
    let aNewResult = cubic(aA, aB, aC, aD);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    compare_roots(old_roots(&anOldSolver), new_roots(&aNewResult), 1.0e-6);
}

#[test]
fn MathPoly_ComparisonTest__Quadratic_NumericallyChallengingCase() {
    let aA: f64 = 1.0;
    let aB: f64 = -1.0e8;
    let aC: f64 = 1.0;

    let anOldSolver = DirectPolynomialRoots::quadratic(aA, aB, aC);
    let aNewResult = quadratic(aA, aB, aC);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    assert_eq!(anOldSolver.solutions().unwrap(), 2);
    assert_eq!(aNewResult.nb_roots, 2);

    let aOldRoots = old_roots(&anOldSolver);
    let aNewRoots = new_roots(&aNewResult);

    near(aOldRoots[0], aNewRoots[0], 1.0e-14);

    near(aOldRoots[1] / aNewRoots[1], 1.0, 1.0e-8);
}

#[test]
fn MathPoly_ComparisonTest__Quartic_SymmetricRoots() {
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

    let aNewRoots = new_roots(&aNewResult);
    assert_eq!(aNewRoots.len(), 4);
    near(aNewRoots[0], -2.0, THE_TOLERANCE);
    near(aNewRoots[1], -1.0, THE_TOLERANCE);
    near(aNewRoots[2], 1.0, THE_TOLERANCE);
    near(aNewRoots[3], 2.0, THE_TOLERANCE);
}
