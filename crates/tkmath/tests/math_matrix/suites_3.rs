#[test]
fn MathMatrixTest__Determinant() {
    // Test determinant of identity matrix
    let mut anIdentity = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    anIdentity.init(0.0);
    anIdentity[(1, 1)] = 1.0;
    anIdentity[(2, 2)] = 1.0;
    anIdentity[(3, 3)] = 1.0;

    assert_near(anIdentity.determinant().unwrap(), 1.0, 1e-7);

    // Test determinant of diagonal matrix
    let mut aDiagonal = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aDiagonal.init(0.0);
    aDiagonal[(1, 1)] = 2.0;
    aDiagonal[(2, 2)] = 3.0;
    aDiagonal[(3, 3)] = 4.0;

    assert_near(aDiagonal.determinant().unwrap(), 24.0, 1e-7);

    // Test determinant of a non-singular matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 3.0;
    aMatrix[(1, 2)] = 8.0;
    aMatrix[(1, 3)] = 5.0;
    aMatrix[(2, 1)] = 6.0;
    aMatrix[(2, 2)] = 2.0;
    aMatrix[(2, 3)] = 7.0;
    aMatrix[(3, 1)] = 4.0;
    aMatrix[(3, 2)] = 1.0;
    aMatrix[(3, 3)] = 9.0;

    // det = 3*(2*9-7*1) - 8*(6*9-7*4) + 5*(6*1-2*4) = 3*11 - 8*26 + 5*-2 = 33 - 208 - 10 = -185
    assert_near(aMatrix.determinant().unwrap(), -185.0, 1e-7);
}

// Tests for Row and Column operations
#[test]
fn MathMatrixTest__RowAndColumnOperations() {
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix.init(0.0);

    // Setup matrix rows
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 4.0;
    aMatrix[(2, 2)] = 5.0;
    aMatrix[(2, 3)] = 6.0;
    aMatrix[(3, 1)] = 7.0;
    aMatrix[(3, 2)] = 8.0;
    aMatrix[(3, 3)] = 9.0;

    // Test Row method
    let mut aRow = aMatrix.row(2).unwrap();
    assert_eq!(aRow.length(), 3);
    assert_eq!(aRow[1], 4.0);
    assert_eq!(aRow[2], 5.0);
    assert_eq!(aRow[3], 6.0);

    // Test Col method
    let mut aCol = aMatrix.col(3).unwrap();
    assert_eq!(aCol.length(), 3);
    assert_eq!(aCol[1], 3.0);
    assert_eq!(aCol[2], 6.0);
    assert_eq!(aCol[3], 9.0);

    // Test SetRow
    let mut aNewRow = Vector::with_bounds(1, 3).unwrap();
    aNewRow[1] = 10.0;
    aNewRow[2] = 11.0;
    aNewRow[3] = 12.0;
    aMatrix.set_row(2, &aNewRow).unwrap();

    assert_eq!(aMatrix[(2, 1)], 10.0);
    assert_eq!(aMatrix[(2, 2)], 11.0);
    assert_eq!(aMatrix[(2, 3)], 12.0);

    // Test SetCol
    let mut aNewCol = Vector::with_bounds(1, 3).unwrap();
    aNewCol[1] = 13.0;
    aNewCol[2] = 14.0;
    aNewCol[3] = 15.0;
    aMatrix.set_col(1, &aNewCol).unwrap();

    assert_eq!(aMatrix[(1, 1)], 13.0);
    assert_eq!(aMatrix[(2, 1)], 14.0);
    assert_eq!(aMatrix[(3, 1)], 15.0);

    // Test SwapRow
    aMatrix.swap_row(1, 3).unwrap();
    assert_eq!(aMatrix[(1, 1)], 15.0);
    assert_eq!(aMatrix[(1, 2)], 8.0);
    assert_eq!(aMatrix[(1, 3)], 9.0);
    assert_eq!(aMatrix[(3, 1)], 13.0);
    assert_eq!(aMatrix[(3, 2)], 2.0);
    assert_eq!(aMatrix[(3, 3)], 3.0);

    // Test SwapCol
    aMatrix.swap_col(2, 3).unwrap();
    assert_eq!(aMatrix[(1, 2)], 9.0);
    assert_eq!(aMatrix[(1, 3)], 8.0);
    assert_eq!(aMatrix[(2, 2)], 12.0);
    assert_eq!(aMatrix[(2, 3)], 11.0);
    assert_eq!(aMatrix[(3, 2)], 3.0);
    assert_eq!(aMatrix[(3, 3)], 2.0);
}

// Tests for SetDiag method
