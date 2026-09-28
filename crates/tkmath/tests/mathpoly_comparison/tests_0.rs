#[test]
fn MathPoly_ComparisonTest__Quadratic_TwoDistinctRoots() {
    let aA: f64 = 1.0;
    let aB: f64 = -5.0;
    let aC: f64 = 6.0;

    let anOldSolver = DirectPolynomialRoots::quadratic(aA, aB, aC);
    let aNewResult = quadratic(aA, aB, aC);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    compare_roots(
        old_roots(&anOldSolver),
        new_roots(&aNewResult),
        THE_TOLERANCE,
    );
}

#[test]
fn MathPoly_ComparisonTest__Quadratic_DoubleRoot() {
    let aA: f64 = 1.0;
    let aB: f64 = -4.0;
    let aC: f64 = 4.0;

    let anOldSolver = DirectPolynomialRoots::quadratic(aA, aB, aC);
    let aNewResult = quadratic(aA, aB, aC);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    let aOldRoots = old_roots(&anOldSolver);
    let aNewRoots = new_roots(&aNewResult);

    for aRoot in aOldRoots {
        near(aRoot, 2.0, THE_TOLERANCE);
    }
    for aRoot in aNewRoots {
        near(aRoot, 2.0, THE_TOLERANCE);
    }
}

#[test]
fn MathPoly_ComparisonTest__Quadratic_NoRealRoots() {
    let aA: f64 = 1.0;
    let aB: f64 = 0.0;
    let aC: f64 = 1.0;

    let anOldSolver = DirectPolynomialRoots::quadratic(aA, aB, aC);
    let aNewResult = quadratic(aA, aB, aC);

    assert_eq!(anOldSolver.solutions().unwrap(), 0);
    assert_eq!(aNewResult.nb_roots, 0);
}
