#[test]
fn MathMatrixTest__InPlaceMatrixMultiplication() {
    // Create a test matrix with specific values to reveal the bug
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(2, 1)] = 3.0;
    aMatrix[(2, 2)] = 4.0;

    // Create a copy of the matrix for comparison
    let mut aMatrixCopy = aMatrix.clone();

    // Expected result of matrix * matrix (calculated by hand)
    // [1 2] * [1 2] = [7 10]
    // [3 4]   [3 4]   [15 22]
    let mut aExpectedResult = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aExpectedResult[(1, 1)] = 7.0;
    aExpectedResult[(1, 2)] = 10.0;
    aExpectedResult[(2, 1)] = 15.0;
    aExpectedResult[(2, 2)] = 22.0;

    // Non-in-place multiplication result (known to be correct)
    let mut aCorrectResult = aMatrixCopy.multiplied_matrix(&aMatrixCopy).unwrap();

    // Verify that the non-in-place multiplication gives the correct result
    assert_eq!(aCorrectResult[(1, 1)], 7.0);
    assert_eq!(aCorrectResult[(1, 2)], 10.0);
    assert_eq!(aCorrectResult[(2, 1)], 15.0);
    assert_eq!(aCorrectResult[(2, 2)], 22.0);

    // Perform in-place multiplication with self
    aMatrix.square().unwrap();

    // If the bug is fixed, both results should be identical
    assert_near(aMatrix[(1, 1)], aCorrectResult[(1, 1)], 1e-7);
    assert_near(aMatrix[(1, 2)], aCorrectResult[(1, 2)], 1e-7);
    assert_near(aMatrix[(2, 1)], aCorrectResult[(2, 1)], 1e-7);
    assert_near(aMatrix[(2, 2)], aCorrectResult[(2, 2)], 1e-7);

    // Also test the operator*= version with self
    aMatrixCopy *= &aMatrixCopy.clone();

    // Verify the operator*= with self gives correct results
    assert_near(aMatrixCopy[(1, 1)], aCorrectResult[(1, 1)], 1e-7);
    assert_near(aMatrixCopy[(1, 2)], aCorrectResult[(1, 2)], 1e-7);
    assert_near(aMatrixCopy[(2, 1)], aCorrectResult[(2, 1)], 1e-7);
    assert_near(aMatrixCopy[(2, 2)], aCorrectResult[(2, 2)], 1e-7);

    // Create two different matrices
    let mut aMatrixA = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrixA[(1, 1)] = 1.0;
    aMatrixA[(1, 2)] = 2.0;
    aMatrixA[(2, 1)] = 3.0;
    aMatrixA[(2, 2)] = 4.0;

    let mut aMatrixB = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrixB[(1, 1)] = 5.0;
    aMatrixB[(1, 2)] = 6.0;
    aMatrixB[(2, 1)] = 7.0;
    aMatrixB[(2, 2)] = 8.0;

    // Create copies for non-in-place calculation
    let mut aMatrixACopy = aMatrixA.clone();
    let mut aMatrixBCopy = aMatrixB.clone();

    // Calculate expected result using non-in-place multiplication
    let mut aExpectedAB = aMatrixACopy.multiplied_matrix(&aMatrixBCopy).unwrap();

    // Expected result: [1 2] * [5 6] = [19 22]
    //                  [3 4]   [7 8]   [43 50]
    assert_eq!(aExpectedAB[(1, 1)], 19.0);
    assert_eq!(aExpectedAB[(1, 2)], 22.0);
    assert_eq!(aExpectedAB[(2, 1)], 43.0);
    assert_eq!(aExpectedAB[(2, 2)], 50.0);

    // Test in-place multiplication with different matrix
    aMatrixA.multiply_matrix(&aMatrixB).unwrap();

    // Check results match expected values
    assert_near(aMatrixA[(1, 1)], aExpectedAB[(1, 1)], 1e-7);
    assert_near(aMatrixA[(1, 2)], aExpectedAB[(1, 2)], 1e-7);
    assert_near(aMatrixA[(2, 1)], aExpectedAB[(2, 1)], 1e-7);
    assert_near(aMatrixA[(2, 2)], aExpectedAB[(2, 2)], 1e-7);

    // Test operator*= with different matrices
    aMatrixACopy *= &aMatrixBCopy;

    // Check results match expected values
    assert_near(aMatrixACopy[(1, 1)], aExpectedAB[(1, 1)], 1e-7);
    assert_near(aMatrixACopy[(1, 2)], aExpectedAB[(1, 2)], 1e-7);
    assert_near(aMatrixACopy[(2, 1)], aExpectedAB[(2, 1)], 1e-7);
    assert_near(aMatrixACopy[(2, 2)], aExpectedAB[(2, 2)], 1e-7);
}

// Tests for Move Semantics
#[test]
fn MathMatrixTest__MoveSemantics() {
    // --- Move Constructor ---

    // Large matrix (heap allocated)
    let aRows: i32 = 10;
    let aCols: i32 = 10;
    let mut aMat1 = Matrix::with_bounds(1, aRows, 1, aCols).unwrap();
    aMat1.init(1.0);
    aMat1[(1, 1)] = 2.0;

    // Move aMat1 to aMat2
    let mut aMat2 = Matrix::move_construct(&mut aMat1);

    assert_eq!(aMat2.row_number(), aRows);
    assert_eq!(aMat2.col_number(), aCols);
    assert_eq!(aMat2[(1, 1)], 2.0);

    // Verify source state (should be empty after move)
    assert_eq!(aMat1.row_number(), 0);

    // Small matrix (buffer allocated)
    let aSmallRows: i32 = 4;
    let aSmallCols: i32 = 4;
    let mut aSmallMat1 = Matrix::with_bounds(1, aSmallRows, 1, aSmallCols).unwrap();
    aSmallMat1.init(1.0);

    // Move aSmallMat1 to aSmallMat2 (should copy because of buffer)
    let mut aSmallMat2 = Matrix::move_construct(&mut aSmallMat1);

    assert_eq!(aSmallMat2.row_number(), aSmallRows);
    assert_eq!(aSmallMat2[(1, 1)], 1.0);

    // Source remains valid for buffer-based matrix
    assert_eq!(aSmallMat1.row_number(), aSmallRows);
    assert_eq!(aSmallMat1[(1, 1)], 1.0);

    // --- Move Assignment ---

    // Large matrix move assignment
    let mut aMatAssign1 = Matrix::with_bounds(1, aRows, 1, aCols).unwrap();
    aMatAssign1.init(5.0);

    let mut aMatAssign2 = Matrix::with_bounds(1, aRows, 1, aCols).unwrap();
    aMatAssign2.init(0.0);

    aMatAssign2.move_assign(&mut aMatAssign1).unwrap();

    assert_eq!(aMatAssign2[(1, 1)], 5.0);
    assert_eq!(aMatAssign1.row_number(), 0);
}
