#[test]
fn MathHouseholderTest__ValidIndexRange() {
    // Test valid index range handling
    let mut aA = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 0.0;
    aA[(2, 1)] = 0.0;
    aA[(2, 2)] = 1.0;

    let mut aB = Matrix::with_bounds(1, 2, 1, 2).unwrap(); // Two columns
    aB[(1, 1)] = 1.0;
    aB[(1, 2)] = 2.0;
    aB[(2, 1)] = 3.0;
    aB[(2, 2)] = 4.0;

    let aHouseholder = Householder::from_matrix(&aA, &aB, 1e-20).unwrap();
    assert!(aHouseholder.is_done());

    let mut aSol = Vector::with_bounds(1, 2).unwrap();

    // Test valid indices
    aHouseholder.value(&mut aSol, 1).unwrap(); // Should work
    assert_eq!(aSol.length(), 2);

    aHouseholder.value(&mut aSol, 2).unwrap(); // Should work
    assert_eq!(aSol.length(), 2);

    // Verify we have the expected number of columns to work with
    assert_eq!(aB.col_number(), 2);
}

#[test]
fn MathHouseholderTest__RegressionTest() {
    // Regression test with known data
    let mut aA = Matrix::with_bounds(1, 3, 1, 2).unwrap();
    aA[(1, 1)] = 2.0;
    aA[(1, 2)] = 1.0; // 2x + y = 5
    aA[(2, 1)] = 1.0;
    aA[(2, 2)] = 1.0; // x + y = 3
    aA[(3, 1)] = 1.0;
    aA[(3, 2)] = 2.0; // x + 2y = 4

    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 5.0;
    aB[2] = 3.0;
    aB[3] = 4.0;

    let aHouseholder = Householder::from_vector(&aA, &aB, 1e-20).unwrap();
    assert!(aHouseholder.is_done());

    let mut aSol = Vector::with_bounds(1, 2).unwrap();
    aHouseholder.value(&mut aSol, 1).unwrap();

    // Expected least squares solution
    assert_near(aSol[1], 2.0, 1.0e-10);
    assert_near(aSol[2], 1.0, 1.0e-10);
}
