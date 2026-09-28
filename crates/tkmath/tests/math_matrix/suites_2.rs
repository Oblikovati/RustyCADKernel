#[test]
fn MathMatrixTest__MatrixMultiplication() {
    // Create identity matrix
    let mut anIdentity = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    anIdentity.init(0.0);
    anIdentity[(1, 1)] = 1.0;
    anIdentity[(2, 2)] = 1.0;
    anIdentity[(3, 3)] = 1.0;

    // Create test matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    for anI in 1..=3 {
        for aJ in 1..=3 {
            aMatrix[(anI, aJ)] = f64::from(anI + aJ);
        }
    }

    // Test matrix multiplication with identity
    let mut aResult = aMatrix.multiplied_matrix(&anIdentity).unwrap();
    check_matrices_equal(&aResult, &aMatrix);

    // Test matrix multiplication
    let mut aMatA = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    let mut aMatB = Matrix::with_bounds(1, 3, 1, 2).unwrap();

    aMatA[(1, 1)] = 1.0;
    aMatA[(1, 2)] = 2.0;
    aMatA[(1, 3)] = 3.0;
    aMatA[(2, 1)] = 4.0;
    aMatA[(2, 2)] = 5.0;
    aMatA[(2, 3)] = 6.0;

    aMatB[(1, 1)] = 7.0;
    aMatB[(1, 2)] = 8.0;
    aMatB[(2, 1)] = 9.0;
    aMatB[(2, 2)] = 10.0;
    aMatB[(3, 1)] = 11.0;
    aMatB[(3, 2)] = 12.0;

    let mut aMatC = aMatA.multiplied_matrix(&aMatB).unwrap();

    assert_eq!(aMatC[(1, 1)], (1 * 7 + 2 * 9 + 3 * 11) as f64);
    assert_eq!(aMatC[(1, 2)], (1 * 8 + 2 * 10 + 3 * 12) as f64);
    assert_eq!(aMatC[(2, 1)], (4 * 7 + 5 * 9 + 6 * 11) as f64);
    assert_eq!(aMatC[(2, 2)], (4 * 8 + 5 * 10 + 6 * 12) as f64);

    // Test in-place matrix multiplication using Multiply method
    let mut aInPlaceMulMat1 = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    let mut aInPlaceMulMat2 = Matrix::with_bounds(1, 2, 1, 2).unwrap();

    // Set matrix values
    aInPlaceMulMat1[(1, 1)] = 1.0;
    aInPlaceMulMat1[(1, 2)] = 2.0;
    aInPlaceMulMat1[(2, 1)] = 3.0;
    aInPlaceMulMat1[(2, 2)] = 4.0;

    aInPlaceMulMat2[(1, 1)] = 2.0;
    aInPlaceMulMat2[(1, 2)] = 0.0;
    aInPlaceMulMat2[(2, 1)] = 1.0;
    aInPlaceMulMat2[(2, 2)] = 3.0;

    // Store original matrix for comparison
    let mut aOrigMat = aInPlaceMulMat1.clone();

    // Perform in-place multiplication
    aInPlaceMulMat1.multiply_matrix(&aInPlaceMulMat2).unwrap();

    // Expected result: [1 2; 3 4] * [2 0; 1 3] = [4 6; 10 12]
    assert_eq!(aInPlaceMulMat1[(1, 1)], (1 * 2 + 2 * 1) as f64); // 4
    assert_eq!(aInPlaceMulMat1[(1, 2)], (1 * 0 + 2 * 3) as f64); // 6
    assert_eq!(aInPlaceMulMat1[(2, 1)], (3 * 2 + 4 * 1) as f64); // 10
    assert_eq!(aInPlaceMulMat1[(2, 2)], (3 * 0 + 4 * 3) as f64); // 12

    // Test in-place matrix multiplication using operator*=
    let mut aInPlaceOpMat = aOrigMat.clone();
    aInPlaceOpMat *= &aInPlaceMulMat2;

    // Check same result as with direct Multiply method
    assert_eq!(aInPlaceOpMat[(1, 1)], (4) as f64);
    assert_eq!(aInPlaceOpMat[(1, 2)], (6) as f64);
    assert_eq!(aInPlaceOpMat[(2, 1)], (10) as f64);
    assert_eq!(aInPlaceOpMat[(2, 2)], (12) as f64);
}

// Tests for transpose and inverse operations
#[test]
fn MathMatrixTest__TransposeAndInverse() {
    // Test transpose
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 4.0;
    aMatrix[(2, 2)] = 5.0;
    aMatrix[(2, 3)] = 6.0;

    let mut aTransposed = aMatrix.transposed();

    assert_eq!(aTransposed.row_number(), aMatrix.col_number());
    assert_eq!(aTransposed.col_number(), aMatrix.row_number());
    assert_eq!(aTransposed[(1, 1)], aMatrix[(1, 1)]);
    assert_eq!(aTransposed[(2, 1)], aMatrix[(1, 2)]);
    assert_eq!(aTransposed[(3, 1)], aMatrix[(1, 3)]);
    assert_eq!(aTransposed[(1, 2)], aMatrix[(2, 1)]);
    assert_eq!(aTransposed[(2, 2)], aMatrix[(2, 2)]);
    assert_eq!(aTransposed[(3, 2)], aMatrix[(2, 3)]);

    // Test inverse of a matrix
    let mut anInvertible = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    anInvertible[(1, 1)] = 1.0;
    anInvertible[(1, 2)] = 0.0;
    anInvertible[(1, 3)] = 0.0;
    anInvertible[(2, 1)] = 0.0;
    anInvertible[(2, 2)] = 2.0;
    anInvertible[(2, 3)] = 0.0;
    anInvertible[(3, 1)] = 0.0;
    anInvertible[(3, 2)] = 0.0;
    anInvertible[(3, 3)] = 4.0;

    let mut anInverse = anInvertible.inverse().unwrap();

    assert_eq!(anInverse[(1, 1)], 1.0);
    assert_eq!(anInverse[(2, 2)], 0.5);
    assert_eq!(anInverse[(3, 3)], 0.25);

    // Test identity after multiplication
    let mut anIdentity = anInvertible.multiplied_matrix(&anInverse).unwrap();

    assert_near(anIdentity[(1, 1)], 1.0, 1e-7);
    assert_near(anIdentity[(2, 2)], 1.0, 1e-7);
    assert_near(anIdentity[(3, 3)], 1.0, 1e-7);
    assert_near(anIdentity[(1, 2)], 0.0, 1e-7);
    assert_near(anIdentity[(1, 3)], 0.0, 1e-7);
    assert_near(anIdentity[(2, 1)], 0.0, 1e-7);
    assert_near(anIdentity[(2, 3)], 0.0, 1e-7);
    assert_near(anIdentity[(3, 1)], 0.0, 1e-7);
    assert_near(anIdentity[(3, 2)], 0.0, 1e-7);
}

// Tests for determinant calculation
