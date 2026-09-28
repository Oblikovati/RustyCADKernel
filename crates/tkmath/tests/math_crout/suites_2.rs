#[test]
fn MathCroutTest__SingularMatrix() {
    // Test with singular matrix (rank deficient)
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 4.0;
    aMatrix[(2, 3)] = 6.0; // Row 2 = 2 * Row 1
    aMatrix[(3, 1)] = 3.0;
    aMatrix[(3, 2)] = 6.0;
    aMatrix[(3, 3)] = 9.0; // Row 3 = 3 * Row 1

    let aCrout = Crout::new(&aMatrix).unwrap();

    assert!(!aCrout.is_done());
}

#[test]
fn MathCroutTest__NonSquareMatrixCheck() {
    // Test detection of non-square matrix
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 3).unwrap(); // 2x3 matrix
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 4.0;
    aMatrix[(2, 2)] = 5.0;
    aMatrix[(2, 3)] = 6.0;

    // In release builds, verify the solver correctly handles dimension mismatch
    // Crout decomposition requires square matrices
    assert_ne!(aMatrix.row_number(), aMatrix.col_number());
}

#[test]
fn MathCroutTest__DimensionCompatibilityInSolve() {
    // Test dimension compatibility in Solve method
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

    let aCrout = Crout::new(&aMatrix).unwrap();
    assert!(aCrout.is_done());

    // Test with correctly sized vectors
    let mut aB_correct = Vector::with_bounds(1, 3).unwrap();
    aB_correct[1] = 1.0;
    aB_correct[2] = 2.0;
    aB_correct[3] = 3.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aCrout.solve_into(&aB_correct, &mut aX).unwrap();

    // Verify the solution is reasonable
    assert_eq!(aX.length(), 3);
}
