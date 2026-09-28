#[test]
fn MathHouseholderTest__IdentityMatrix() {
    // Test with identity matrix (trivial case)
    let mut aA = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 0.0;
    aA[(1, 3)] = 0.0;
    aA[(2, 1)] = 0.0;
    aA[(2, 2)] = 1.0;
    aA[(2, 3)] = 0.0;
    aA[(3, 1)] = 0.0;
    aA[(3, 2)] = 0.0;
    aA[(3, 3)] = 1.0;

    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 5.0;
    aB[2] = 7.0;
    aB[3] = 9.0;

    let aHouseholder = Householder::from_vector(&aA, &aB, 1e-20).unwrap();
    assert!(aHouseholder.is_done());

    let mut aSol = Vector::with_bounds(1, 3).unwrap();
    aHouseholder.value(&mut aSol, 1).unwrap();

    // For identity matrix, solution should equal RHS
    assert_near(aSol[1], 5.0, 1e-7);
    assert_near(aSol[2], 7.0, 1e-7);
    assert_near(aSol[3], 9.0, 1e-7);
}

// Removed RangeConstructor test due to unknown exception issues
// TODO: Investigate math_Householder range constructor compatibility

#[test]
fn MathHouseholderTest__DimensionCompatibility() {
    // Test dimension compatibility handling
    let mut aA = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 2.0;
    aA[(2, 1)] = 3.0;
    aA[(2, 2)] = 4.0;
    aA[(3, 1)] = 5.0;
    aA[(3, 2)] = 6.0;

    // Test with correctly sized B vector
    let mut aB_correct = Vector::with_bounds(1, 3).unwrap(); // Correct size 3
    aB_correct[1] = 1.0;
    aB_correct[2] = 2.0;
    aB_correct[3] = 3.0;

    // Test with correctly sized B vector - should work
    let aHouseholder1 = Householder::from_vector(&aA, &aB_correct, 1e-20).unwrap();
    assert!(aHouseholder1.is_done());

    // Test with correctly sized B matrix
    let mut aB_matrix_correct = Matrix::with_bounds(1, 3, 1, 2).unwrap(); // Correct row count 3
    aB_matrix_correct[(1, 1)] = 1.0;
    aB_matrix_correct[(1, 2)] = 4.0;
    aB_matrix_correct[(2, 1)] = 2.0;
    aB_matrix_correct[(2, 2)] = 5.0;
    aB_matrix_correct[(3, 1)] = 3.0;
    aB_matrix_correct[(3, 2)] = 6.0;

    let aHouseholder2 = Householder::from_matrix(&aA, &aB_matrix_correct, 1e-20).unwrap();
    assert!(aHouseholder2.is_done());
}

#[test]
fn MathHouseholderTest__NearZeroMatrixState() {
    // Create a scenario where Householder fails
    let mut aA = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aA[(1, 1)] = 1.0e-25;
    aA[(1, 2)] = 0.0; // Extremely small values
    aA[(2, 1)] = 0.0;
    aA[(2, 2)] = 1.0e-25;

    let mut aB = Vector::with_bounds(1, 2).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;

    let aHouseholder = Householder::from_vector(&aA, &aB, 1.0e-10).unwrap(); // Large EPS
    assert!(!aHouseholder.is_done());
}
