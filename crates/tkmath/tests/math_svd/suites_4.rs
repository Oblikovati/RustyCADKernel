#[test]
fn MathSVDTest__DifferentMatrixBounds() {
    // Test with non-standard matrix bounds
    let mut aMatrix = Matrix::with_bounds(2, 4, 3, 5).unwrap(); // 3x3 matrix with custom bounds

    aMatrix[(2, 3)] = 1.0;
    aMatrix[(2, 4)] = 0.0;
    aMatrix[(2, 5)] = 0.0;
    aMatrix[(3, 3)] = 0.0;
    aMatrix[(3, 4)] = 1.0;
    aMatrix[(3, 5)] = 0.0;
    aMatrix[(4, 3)] = 0.0;
    aMatrix[(4, 4)] = 0.0;
    aMatrix[(4, 5)] = 1.0;

    let mut aSVD = Svd::new(&aMatrix).unwrap();
    assert!(aSVD.is_done());

    let mut aB = Vector::with_bounds(2, 4).unwrap(); // Matching row bounds
    aB[2] = 5.0;
    aB[3] = 7.0;
    aB[4] = 9.0;

    let mut aX = Vector::with_bounds(3, 5).unwrap(); // Matching column bounds
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // For identity matrix, solution should equal RHS
    assert_near(aX[3], 5.0, 1e-7);
    assert_near(aX[4], 7.0, 1e-7);
    assert_near(aX[5], 9.0, 1e-7);
}

#[test]
fn MathSVDTest__LargerMatrix() {
    // Test with a larger well-conditioned matrix (5x5)
    let mut aMatrix = Matrix::with_bounds(1, 5, 1, 5).unwrap();

    // Create a symmetric positive definite matrix
    for anI in 1i32..=5 {
        for aJ in 1i32..=5 {
            if anI == aJ {
                aMatrix[(anI, aJ)] = 10.0; // Diagonal dominance
            } else {
                aMatrix[(anI, aJ)] = 1.0 / (f64::from((anI - aJ).abs()) + 1.0);
            }
        }
    }

    let mut aSVD = Svd::new(&aMatrix).unwrap();
    assert!(aSVD.is_done());

    let mut aB = Vector::with_bounds(1, 5).unwrap();
    for anI in 1i32..=5 {
        aB[anI] = f64::from(anI) * 2.0;
    }

    let mut aX = Vector::with_bounds(1, 5).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // Verify the solution
    check_solution(&aMatrix, &aX, &aB, 1.0e-8);
}
