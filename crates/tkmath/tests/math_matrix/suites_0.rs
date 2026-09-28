// Tests for constructors
#[test]
fn MathMatrixTest__Constructors() {
    // Test standard constructor
    let mut aMatrix1 = Matrix::with_bounds(1, 3, 1, 4).unwrap();
    assert_eq!(aMatrix1.row_number(), 3);
    assert_eq!(aMatrix1.col_number(), 4);
    assert_eq!(aMatrix1.lower_row(), 1);
    assert_eq!(aMatrix1.upper_row(), 3);
    assert_eq!(aMatrix1.lower_col(), 1);
    assert_eq!(aMatrix1.upper_col(), 4);

    // Test constructor with initial value
    let mut aMatrix2 = Matrix::filled(2, 4, 3, 5, 2.5).unwrap();
    assert_eq!(aMatrix2.row_number(), 3);
    assert_eq!(aMatrix2.col_number(), 3);

    for anI in 2..=4 {
        for aJ in 3..=5 {
            assert_eq!(aMatrix2[(anI, aJ)], 2.5);
        }
    }

    // Test copy constructor
    let mut aMatrix3 = aMatrix2.clone();
    check_matrices_equal(&aMatrix2, &aMatrix3);
}

// Tests for initialization and access
