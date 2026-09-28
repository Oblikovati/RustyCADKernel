#[test]
fn MathHouseholderTest__MultipleRightHandSides() {
    // Test solving A*X = B where B has multiple columns
    let mut aA = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 0.0;
    aA[(2, 1)] = 0.0;
    aA[(2, 2)] = 1.0;
    aA[(3, 1)] = 1.0;
    aA[(3, 2)] = 1.0;

    let mut aB = Matrix::with_bounds(1, 3, 1, 2).unwrap(); // Two right-hand sides
    aB[(1, 1)] = 1.0;
    aB[(1, 2)] = 2.0; // First RHS: [1, 3, 4]
    aB[(2, 1)] = 3.0;
    aB[(2, 2)] = 4.0; // Second RHS: [2, 4, 6]
    aB[(3, 1)] = 4.0;
    aB[(3, 2)] = 6.0;

    let aHouseholder = Householder::from_matrix(&aA, &aB, 1e-20).unwrap();
    assert!(aHouseholder.is_done());

    // Get first solution
    let mut aSol1 = Vector::with_bounds(1, 2).unwrap();
    aHouseholder.value(&mut aSol1, 1).unwrap();

    // Get second solution
    let mut aSol2 = Vector::with_bounds(1, 2).unwrap();
    aHouseholder.value(&mut aSol2, 2).unwrap();

    // Expected solutions: first RHS gives approximately [1, 3], second gives [2, 4]
    assert_near(aSol1[1], 1.0, 1.0e-6);
    assert_near(aSol1[2], 3.0, 1.0e-6);

    assert_near(aSol2[1], 2.0, 1.0e-6);
    assert_near(aSol2[2], 4.0, 1.0e-6);
}

#[test]
fn MathHouseholderTest__NearSingularMatrix() {
    // Test with nearly singular matrix
    let mut aA = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 2.0;
    aA[(1, 3)] = 3.0;
    aA[(2, 1)] = 2.0;
    aA[(2, 2)] = 4.0;
    aA[(2, 3)] = 6.0 + 1.0e-15; // Nearly dependent
    aA[(3, 1)] = 3.0;
    aA[(3, 2)] = 6.0;
    aA[(3, 3)] = 9.0 + 2.0e-15; // Nearly dependent

    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;
    aB[3] = 3.0;

    // With default EPS, should handle near-singularity
    let aHouseholder = Householder::from_vector(&aA, &aB, 1e-20).unwrap();

    // Near-singular matrix should fail with default EPS
    assert!(!aHouseholder.is_done());
}

#[test]
fn MathHouseholderTest__CustomEpsilon() {
    // Test with custom epsilon threshold using well-conditioned values
    let mut aA = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aA[(1, 1)] = 1.0e-3;
    aA[(1, 2)] = 1.0;
    aA[(2, 1)] = 2.0e-3;
    aA[(2, 2)] = 2.0;

    let mut aB = Vector::with_bounds(1, 2).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;

    // Test that default EPS works with this matrix
    let aHouseholder1 = Householder::from_vector(&aA, &aB, 1e-20).unwrap();
    assert!(aHouseholder1.is_done());

    // Test with very restrictive EPS that should fail
    let aHouseholder2 = Householder::from_vector(&aA, &aB, 1.0e-2).unwrap();
    assert!(!aHouseholder2.is_done());
}
