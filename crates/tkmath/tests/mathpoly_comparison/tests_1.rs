#[test]
fn MathPoly_ComparisonTest__Quadratic_NegativeRoots() {
    let aA: f64 = 1.0;
    let aB: f64 = 5.0;
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
fn MathPoly_ComparisonTest__Quadratic_LargeCoefficients() {
    let aA: f64 = 1000.0;
    let aB: f64 = -3000.0;
    let aC: f64 = 2000.0;

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
fn MathPoly_ComparisonTest__Cubic_ThreeDistinctRoots() {
    let aA: f64 = 1.0;
    let aB: f64 = -6.0;
    let aC: f64 = 11.0;
    let aD: f64 = -6.0;

    let anOldSolver = DirectPolynomialRoots::cubic(aA, aB, aC, aD);
    let aNewResult = cubic(aA, aB, aC, aD);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    compare_roots(
        old_roots(&anOldSolver),
        new_roots(&aNewResult),
        THE_TOLERANCE,
    );
}
