#[test]
fn MathPoly_ComparisonTest__Cubic_OneRealRoot() {
    let aA: f64 = 1.0;
    let aB: f64 = 0.0;
    let aC: f64 = 1.0;
    let aD: f64 = 2.0;

    let anOldSolver = DirectPolynomialRoots::cubic(aA, aB, aC, aD);
    let aNewResult = cubic(aA, aB, aC, aD);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    assert_eq!(
        (anOldSolver.solutions().unwrap()) as usize,
        aNewResult.nb_roots
    );

    if anOldSolver.solutions().unwrap() > 0 && aNewResult.nb_roots > 0 {
        compare_roots(
            old_roots(&anOldSolver),
            new_roots(&aNewResult),
            THE_TOLERANCE,
        );
    }
}

#[test]
fn MathPoly_ComparisonTest__Cubic_TripleRoot() {
    let aA: f64 = 1.0;
    let aB: f64 = -3.0;
    let aC: f64 = 3.0;
    let aD: f64 = -1.0;

    let anOldSolver = DirectPolynomialRoots::cubic(aA, aB, aC, aD);
    let aNewResult = cubic(aA, aB, aC, aD);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    assert_eq!(aNewResult.nb_roots, 1);
    assert_eq!(aNewResult.multiplicities[0], 3);
    let aNewRoots = new_roots(&aNewResult);
    assert_eq!(aNewRoots.len(), 3);
    for aRoot in aNewRoots {
        near(aRoot, 1.0, 1.0e-6);
    }
}

#[test]
fn MathPoly_ComparisonTest__Cubic_NegativeLeadingCoeff() {
    let aA: f64 = -1.0;
    let aB: f64 = 6.0;
    let aC: f64 = -11.0;
    let aD: f64 = 6.0;

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
