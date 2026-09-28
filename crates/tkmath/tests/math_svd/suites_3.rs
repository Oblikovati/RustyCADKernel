#[test]
fn MathSVDTest__PseudoInverseMethod() {
    let aMatrix = create_well_conditioned_matrix();
    let mut aSVD = Svd::new(&aMatrix).unwrap();

    assert!(aSVD.is_done());

    // Test PseudoInverse method - compute A^+ (pseudoinverse of A)
    let mut aPseudoInv = Matrix::with_bounds(
        aMatrix.lower_col(),
        aMatrix.upper_col(),
        aMatrix.lower_row(),
        aMatrix.upper_row(),
    )
    .unwrap();
    aSVD.pseudo_inverse(&mut aPseudoInv, 1e-6).unwrap();

    // For a well-conditioned square matrix, pseudoinverse should behave like regular inverse
    // Test: A^+ * A should be approximately identity
    let mut aProduct = Matrix::with_bounds(
        aMatrix.lower_row(),
        aMatrix.upper_row(),
        aMatrix.lower_row(),
        aMatrix.upper_row(),
    )
    .unwrap();

    for anI in aMatrix.lower_row()..=aMatrix.upper_row() {
        for aJ in aMatrix.lower_row()..=aMatrix.upper_row() {
            let mut aSum: f64 = 0.0;
            for aK in aMatrix.lower_col()..=aMatrix.upper_col() {
                aSum += aPseudoInv[(anI, aK)] * aMatrix[(aK, aJ)];
            }
            aProduct[(anI, aJ)] = aSum;
        }
    }

    // Check if result is approximately identity
    for anI in aMatrix.lower_row()..=aMatrix.upper_row() {
        for aJ in aMatrix.lower_row()..=aMatrix.upper_row() {
            let anExpected: f64 = if anI == aJ { 1.0 } else { 0.0 };
            assert_near(aProduct[(anI, aJ)], anExpected, 1.0e-10);
        }
    }
}

// Tests for exception handling
#[test]
fn MathSVDTest__DimensionCompatibility() {
    let aMatrix = create_well_conditioned_matrix();
    let mut aSVD = Svd::new(&aMatrix).unwrap();

    assert!(aSVD.is_done());

    // Test with correctly dimensioned vectors
    let mut aCorrectB = Vector::with_bounds(1, 3).unwrap(); // Correct size for 3x3 matrix
    aCorrectB[1] = 1.0;
    aCorrectB[2] = 2.0;
    aCorrectB[3] = 3.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap(); // Correct size for solution
    aSVD.solve(&aCorrectB, &mut aX, 1e-6).unwrap();

    // Verify the results make sense
    assert_eq!(aX.length(), 3);
    assert_eq!(aCorrectB.length(), 3);

    // Verify matrix dimensions are consistent
    assert_eq!(aMatrix.row_number(), 3);
    assert_eq!(aMatrix.col_number(), 3);
}

#[test]
fn MathSVDTest__SingularValues() {
    // Test a matrix where we can predict singular values
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 3.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 4.0;

    let mut aSVD = Svd::new(&aMatrix).unwrap();
    assert!(aSVD.is_done());

    // For a diagonal matrix, singular values should be the absolute values of diagonal elements
    // We can't directly access singular values in this interface, but we can verify through solutions
    let mut aB1 = Vector::with_bounds(1, 2).unwrap();
    aB1[1] = 6.0;
    aB1[2] = 0.0;
    let mut aX1 = Vector::with_bounds(1, 2).unwrap();
    aSVD.solve(&aB1, &mut aX1, 1e-6).unwrap();
    assert_near(aX1[1], 2.0, 1.0e-10);
    assert_near(aX1[2], 0.0, 1.0e-10);

    let mut aB2 = Vector::with_bounds(1, 2).unwrap();
    aB2[1] = 0.0;
    aB2[2] = 12.0;
    let mut aX2 = Vector::with_bounds(1, 2).unwrap();
    aSVD.solve(&aB2, &mut aX2, 1e-6).unwrap();
    assert_near(aX2[1], 0.0, 1.0e-10);
    assert_near(aX2[2], 3.0, 1.0e-10);
}
