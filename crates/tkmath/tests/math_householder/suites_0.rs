#[test]
fn MathHouseholderTest__ExactlyDeterminedSystem() {
    // Test with a square matrix (exact solution exists)
    let mut aA = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 2.0;
    aA[(1, 3)] = 3.0;
    aA[(2, 1)] = 4.0;
    aA[(2, 2)] = 5.0;
    aA[(2, 3)] = 6.0;
    aA[(3, 1)] = 7.0;
    aA[(3, 2)] = 8.0;
    aA[(3, 3)] = 10.0; // Note: 9 would make it singular

    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 14.0;
    aB[2] = 32.0;
    aB[3] = 55.0; // Should give solution approximately [1, 2, 3]

    let aHouseholder = Householder::from_vector(&aA, &aB, 1e-20).unwrap();
    assert!(aHouseholder.is_done());

    let mut aSol = Vector::with_bounds(1, 3).unwrap();
    aHouseholder.value(&mut aSol, 1).unwrap();

    // Verify solution by checking A * x = b
    let mut aVerify = Vector::with_bounds(1, 3).unwrap();
    for i in 1..=3 {
        aVerify[i] = 0.0;
        for j in 1..=3 {
            aVerify[i] += aA[(i, j)] * aSol[j];
        }
    }

    assert_near(aVerify[1], aB[1], 1.0e-10);
    assert_near(aVerify[2], aB[2], 1.0e-10);
    assert_near(aVerify[3], aB[3], 1.0e-10);
}

#[test]
fn MathHouseholderTest__OverdeterminedSystem() {
    // Test with an overdetermined system (more equations than unknowns)
    let mut aA = Matrix::with_bounds(1, 4, 1, 2).unwrap(); // 4 equations, 2 unknowns
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 1.0; // x + y = 2 (approximately)
    aA[(2, 1)] = 1.0;
    aA[(2, 2)] = 2.0; // x + 2y = 3
    aA[(3, 1)] = 2.0;
    aA[(3, 2)] = 1.0; // 2x + y = 3
    aA[(4, 1)] = 1.0;
    aA[(4, 2)] = 3.0; // x + 3y = 4

    let mut aB = Vector::with_bounds(1, 4).unwrap();
    aB[1] = 2.0;
    aB[2] = 3.0;
    aB[3] = 3.0;
    aB[4] = 4.0;

    let aHouseholder = Householder::from_vector(&aA, &aB, 1e-20).unwrap();
    assert!(aHouseholder.is_done());

    let mut aSol = Vector::with_bounds(1, 2).unwrap();
    aHouseholder.value(&mut aSol, 1).unwrap();

    // For least squares, solution should minimize ||A*x - b||^2
    // Expected solution is approximately [1, 1]
    assert_near(aSol[1], 1.0, 1.0e-6);
    assert_near(aSol[2], 1.0, 1.0e-6);
}
