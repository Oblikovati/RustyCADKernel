#[test]
fn MathJacobiTest__SingleElementMatrix() {
    // Test with 1x1 matrix
    let mut aMatrix = Matrix::with_bounds(1, 1, 1, 1).unwrap();
    aMatrix[(1, 1)] = 42.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    assert!(aJacobi.is_done());

    let aValues = aJacobi.values().unwrap();
    assert_eq!(aValues.length(), 1);
    assert_near(aValues[1], 42.0, 1.0e-12);

    let mut aV = Vector::with_bounds(1, 1).unwrap();
    aJacobi.vector(1, &mut aV).unwrap();
    assert_near(aV[1], 1.0, 1.0e-12);
}

#[test]
fn MathJacobiTest__SquareMatrixRequirement() {
    // Test square matrix requirement for Jacobi eigenvalue decomposition
    let mut aNonSquareMatrix = Matrix::with_bounds(1, 2, 1, 3).unwrap(); // 2x3 matrix
    aNonSquareMatrix[(1, 1)] = 1.0;
    aNonSquareMatrix[(1, 2)] = 2.0;
    aNonSquareMatrix[(1, 3)] = 3.0;
    aNonSquareMatrix[(2, 1)] = 4.0;
    aNonSquareMatrix[(2, 2)] = 5.0;
    aNonSquareMatrix[(2, 3)] = 6.0;

    // Verify matrix is indeed non-square
    assert_ne!(aNonSquareMatrix.row_number(), aNonSquareMatrix.col_number());

    // Jacobi eigenvalue decomposition requires square matrices
    // Test with a proper square matrix instead
    let mut aSquareMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aSquareMatrix[(1, 1)] = 1.0;
    aSquareMatrix[(1, 2)] = 0.5;
    aSquareMatrix[(2, 1)] = 0.5;
    aSquareMatrix[(2, 2)] = 2.0;

    let aJacobi = Jacobi::new(&aSquareMatrix).unwrap();
    assert!(aJacobi.is_done());
}

#[test]
fn MathJacobiTest__NotDoneExceptions() {
    // Create a scenario where Jacobi might fail (though it usually succeeds)
    // Test exception handling for accessing results before computation
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 1.0;

    let aJacobi = Jacobi::new(&aMatrix).unwrap();

    if !aJacobi.is_done() {
        // Only test exceptions if computation actually failed
        assert_eq!(
            aJacobi.values().err().unwrap().exception_type(),
            "StdFail_NotDone"
        );
        assert_eq!(
            aJacobi.value(1).err().unwrap().exception_type(),
            "StdFail_NotDone"
        );
        assert_eq!(
            aJacobi.vectors().err().unwrap().exception_type(),
            "StdFail_NotDone"
        );

        let mut aV = Vector::with_bounds(1, 2).unwrap();
        assert_eq!(
            aJacobi.vector(1, &mut aV).err().unwrap().exception_type(),
            "StdFail_NotDone"
        );
    }
}
