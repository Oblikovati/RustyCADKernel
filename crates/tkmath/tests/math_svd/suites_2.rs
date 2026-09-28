#[test]
fn MathSVDTest__RankDeficientMatrix() {
    // Rank deficient matrix (rank 2, but 3x3)
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 4.0;
    aMatrix[(2, 3)] = 6.0; // 2 * row 1
    aMatrix[(3, 1)] = 1.0;
    aMatrix[(3, 2)] = 1.0;
    aMatrix[(3, 3)] = 1.0;

    let mut aSVD = Svd::new(&aMatrix).unwrap();
    assert!(aSVD.is_done());

    // Consistent RHS (b2 = 2*b1, compatible with rank deficiency)
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 6.0;
    aB[2] = 12.0; // 2 * b1
    aB[3] = 3.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // Verify the solution
    check_solution(&aMatrix, &aX, &aB, 1.0e-6);
}

#[test]
fn MathSVDTest__SingleRowMatrix() {
    // Single equation, multiple unknowns (1x3)
    let mut aMatrix = Matrix::with_bounds(1, 1, 1, 3).unwrap();
    aMatrix[(1, 1)] = 2.0;
    aMatrix[(1, 2)] = 3.0;
    aMatrix[(1, 3)] = 4.0;

    let mut aSVD = Svd::new(&aMatrix).unwrap();
    assert!(aSVD.is_done());

    let mut aB = Vector::with_bounds(1, 1).unwrap();
    aB[1] = 20.0; // 2*2 + 3*3 + 4*3 = 20 (one possible solution: x=[2,3,3])

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // Verify the solution
    check_solution(&aMatrix, &aX, &aB, 1e-10);
}

#[test]
fn MathSVDTest__SingleColumnMatrix() {
    // Multiple equations, single unknown (3x1)
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 1).unwrap();
    aMatrix[(1, 1)] = 2.0;
    aMatrix[(2, 1)] = 3.0;
    aMatrix[(3, 1)] = 4.0;

    let mut aSVD = Svd::new(&aMatrix).unwrap();
    assert!(aSVD.is_done());

    // Least squares problem: find x that minimizes ||Ax - b||^2
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 4.0; // 2x approximately 4 -> x approximately 2
    aB[2] = 6.0; // 3x approximately 6 -> x approximately 2
    aB[3] = 8.0; // 4x approximately 8 -> x approximately 2

    let mut aX = Vector::with_bounds(1, 1).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    assert_near(aX[1], 2.0, 1.0e-10);
}
