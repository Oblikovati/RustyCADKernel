#[test]
fn MathCroutTest__DiagonalMatrix() {
    // Test with diagonal matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 2.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(1, 3)] = 0.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 3.0;
    aMatrix[(2, 3)] = 0.0;
    aMatrix[(3, 1)] = 0.0;
    aMatrix[(3, 2)] = 0.0;
    aMatrix[(3, 3)] = 4.0;

    let aCrout = Crout::new(&aMatrix).unwrap();

    assert!(aCrout.is_done());

    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 4.0;
    aB[2] = 9.0;
    aB[3] = 12.0; // Expected solution: [2, 3, 3]

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aCrout.solve_into(&aB, &mut aX).unwrap();

    assert_near(aX[1], 2.0, 1.0e-12);
    assert_near(aX[2], 3.0, 1.0e-12);
    assert_near(aX[3], 3.0, 1.0e-12);
}

#[test]
fn MathCroutTest__LowerTriangularInput() {
    // Test providing only the lower triangular part (as mentioned in documentation)
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    // Only fill lower triangular part
    aMatrix[(1, 1)] = 4.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(1, 3)] = 0.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 3.0;
    aMatrix[(2, 3)] = 0.0;
    aMatrix[(3, 1)] = 1.0;
    aMatrix[(3, 2)] = 0.5;
    aMatrix[(3, 3)] = 2.0;

    let aCrout = Crout::new(&aMatrix).unwrap();

    assert!(aCrout.is_done());
}

#[test]
fn MathCroutTest__CustomMinPivot() {
    // Test with custom minimum pivot threshold
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 1.0e-15;
    aMatrix[(1, 2)] = 1.0;
    aMatrix[(2, 1)] = 1.0;
    aMatrix[(2, 2)] = 1.0;

    // With large MinPivot, should fail
    let aCrout1 = Crout::with_min_pivot(&aMatrix, 1.0e-10).unwrap();
    assert!(!aCrout1.is_done());

    // With small MinPivot, should succeed
    let aCrout2 = Crout::with_min_pivot(&aMatrix, 1.0e-20).unwrap();
    assert!(aCrout2.is_done());
}
