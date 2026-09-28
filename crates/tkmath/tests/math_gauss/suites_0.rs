#[test]
fn MathGaussTest__WellConditionedMatrix() {
    // Test with a simple 3x3 well-conditioned matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 2.0;
    aMatrix[(1, 2)] = 1.0;
    aMatrix[(1, 3)] = 1.0;
    aMatrix[(2, 1)] = 1.0;
    aMatrix[(2, 2)] = 3.0;
    aMatrix[(2, 3)] = 2.0;
    aMatrix[(3, 1)] = 1.0;
    aMatrix[(3, 2)] = 2.0;
    aMatrix[(3, 3)] = 3.0;

    let aGauss = Gauss::new(&aMatrix).unwrap();

    assert!(aGauss.is_done());

    // Verify by solving with known RHS and checking A*x = b
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;
    aB[3] = 3.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aGauss.solve_into(&aB, &mut aX).unwrap();

    // Verify solution by checking A * x = b
    let mut aVerify = Vector::with_bounds(1, 3).unwrap();
    for i in 1..=3 {
        aVerify[i] = 0.0;
        for j in 1..=3 {
            aVerify[i] += aMatrix[(i, j)] * aX[j];
        }
    }

    assert_near(aVerify[1], aB[1], 1.0e-10);
    assert_near(aVerify[2], aB[2], 1.0e-10);
    assert_near(aVerify[3], aB[3], 1.0e-10);
}

#[test]
fn MathGaussTest__IdentityMatrix() {
    // Test with identity matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(1, 3)] = 0.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 1.0;
    aMatrix[(2, 3)] = 0.0;
    aMatrix[(3, 1)] = 0.0;
    aMatrix[(3, 2)] = 0.0;
    aMatrix[(3, 3)] = 1.0;

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss.is_done());

    // For identity matrix, solution should equal RHS
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 5.0;
    aB[2] = 7.0;
    aB[3] = 9.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aGauss.solve_into(&aB, &mut aX).unwrap();

    assert_near(aX[1], 5.0, 1e-7);
    assert_near(aX[2], 7.0, 1e-7);
    assert_near(aX[3], 9.0, 1e-7);
}
