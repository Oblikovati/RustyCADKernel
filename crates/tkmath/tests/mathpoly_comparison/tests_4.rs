#[test]
fn MathPoly_ComparisonTest__Quartic_NoRealRoots() {
    let aA: f64 = 1.0;
    let aB: f64 = 0.0;
    let aC: f64 = 0.0;
    let aD: f64 = 0.0;
    let aE: f64 = 1.0;

    let anOldSolver = DirectPolynomialRoots::quartic(aA, aB, aC, aD, aE);
    let aNewResult = quartic(aA, aB, aC, aD, aE);

    assert_eq!(anOldSolver.solutions().unwrap(), 0);
    assert_eq!(aNewResult.nb_roots, 0);
}

#[test]
fn MathPoly_ComparisonTest__Quartic_QuadrupleRoot() {
    let aA: f64 = 1.0;
    let aB: f64 = -8.0;
    let aC: f64 = 24.0;
    let aD: f64 = -32.0;
    let aE: f64 = 16.0;

    let anOldSolver = DirectPolynomialRoots::quartic(aA, aB, aC, aD, aE);
    let aNewResult = quartic(aA, aB, aC, aD, aE);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    assert_eq!(aNewResult.nb_roots, 1);
    assert_eq!(aNewResult.multiplicities[0], 4);
    let aNewRoots = new_roots(&aNewResult);
    assert_eq!(aNewRoots.len(), 4);
    for aRoot in aNewRoots {
        near(aRoot, 2.0, 1.0e-5);
    }
}

#[test]
fn MathPoly_ComparisonTest__Quadratic_SmallDiscriminant() {
    let aA: f64 = 1.0;
    let aB: f64 = -2.0;
    let aC: f64 = 0.9999;

    let anOldSolver = DirectPolynomialRoots::quadratic(aA, aB, aC);
    let aNewResult = quadratic(aA, aB, aC);

    assert!(anOldSolver.is_done());
    assert!(aNewResult.is_done());

    assert_eq!(anOldSolver.solutions().unwrap(), 2);
    assert_eq!(aNewResult.nb_roots, 2);

    compare_roots(old_roots(&anOldSolver), new_roots(&aNewResult), 1.0e-4);
}
