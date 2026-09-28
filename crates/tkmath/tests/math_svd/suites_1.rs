#[test]
fn MathSVDTest__DiagonalMatrix() {
    // Diagonal matrix test
    let mut aDiagonal = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aDiagonal[(1, 1)] = 3.0;
    aDiagonal[(1, 2)] = 0.0;
    aDiagonal[(1, 3)] = 0.0;
    aDiagonal[(2, 1)] = 0.0;
    aDiagonal[(2, 2)] = 5.0;
    aDiagonal[(2, 3)] = 0.0;
    aDiagonal[(3, 1)] = 0.0;
    aDiagonal[(3, 2)] = 0.0;
    aDiagonal[(3, 3)] = 2.0;

    let mut aSVD = Svd::new(&aDiagonal).unwrap();
    assert!(aSVD.is_done());

    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 12.0;
    aB[2] = 20.0;
    aB[3] = 8.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // For diagonal matrix, X[i] = B[i] / D[i,i]
    assert_near(aX[1], 4.0, 1e-7);
    assert_near(aX[2], 4.0, 1e-7);
    assert_near(aX[3], 4.0, 1e-7);
}

#[test]
fn MathSVDTest__OverdeterminedSystem() {
    // Overdetermined system: more equations than unknowns (4x3)
    let mut aMatrix = Matrix::with_bounds(1, 4, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(1, 3)] = 0.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 1.0;
    aMatrix[(2, 3)] = 0.0;
    aMatrix[(3, 1)] = 0.0;
    aMatrix[(3, 2)] = 0.0;
    aMatrix[(3, 3)] = 1.0;
    aMatrix[(4, 1)] = 1.0;
    aMatrix[(4, 2)] = 1.0;
    aMatrix[(4, 3)] = 1.0;

    let mut aSVD = Svd::new(&aMatrix).unwrap();
    assert!(aSVD.is_done());

    // Consistent system: b4 = b1 + b2 + b3
    let mut aB = Vector::with_bounds(1, 4).unwrap();
    aB[1] = 2.0;
    aB[2] = 3.0;
    aB[3] = 4.0;
    aB[4] = 9.0; // 2 + 3 + 4 = 9

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // Verify the solution
    check_solution(&aMatrix, &aX, &aB, 1.0e-8);
}

#[test]
fn MathSVDTest__UnderdeterminedSystem() {
    // Underdetermined system: fewer equations than unknowns (2x3)
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 4.0;
    aMatrix[(2, 2)] = 5.0;
    aMatrix[(2, 3)] = 6.0;

    let mut aSVD = Svd::new(&aMatrix).unwrap();
    assert!(aSVD.is_done());

    let mut aB = Vector::with_bounds(1, 2).unwrap();
    aB[1] = 14.0; // 1*1 + 2*2 + 3*3 = 14
    aB[2] = 32.0; // 4*1 + 5*2 + 6*3 = 32

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // Verify the solution (should be one of many possible solutions)
    check_solution(&aMatrix, &aX, &aB, 1.0e-8);
}
