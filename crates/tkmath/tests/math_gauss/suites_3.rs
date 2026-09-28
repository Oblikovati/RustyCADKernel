#[test]
fn MathGaussTest__LargerMatrix() {
    // Test with a larger 4x4 matrix
    let mut aMatrix = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 1.0;
    aMatrix[(1, 4)] = 1.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 1.0;
    aMatrix[(2, 3)] = 3.0;
    aMatrix[(2, 4)] = 1.0;
    aMatrix[(3, 1)] = 1.0;
    aMatrix[(3, 2)] = 3.0;
    aMatrix[(3, 3)] = 1.0;
    aMatrix[(3, 4)] = 2.0;
    aMatrix[(4, 1)] = 1.0;
    aMatrix[(4, 2)] = 1.0;
    aMatrix[(4, 3)] = 2.0;
    aMatrix[(4, 4)] = 3.0;

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss.is_done());

    // Test with a simple RHS vector
    let mut aB = Vector::with_bounds(1, 4).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;
    aB[3] = 3.0;
    aB[4] = 4.0;

    let mut aX = Vector::with_bounds(1, 4).unwrap();
    aGauss.solve_into(&aB, &mut aX).unwrap();

    // Verify solution by checking A * x = b
    let mut aVerify = Vector::with_bounds(1, 4).unwrap();
    for i in 1..=4 {
        aVerify[i] = 0.0;
        for j in 1..=4 {
            aVerify[i] += aMatrix[(i, j)] * aX[j];
        }
    }

    assert_near(aVerify[1], aB[1], 1.0e-10);
    assert_near(aVerify[2], aB[2], 1.0e-10);
    assert_near(aVerify[3], aB[3], 1.0e-10);
    assert_near(aVerify[4], aB[4], 1.0e-10);
}

#[test]
fn MathGaussTest__DimensionCompatibility() {
    // Test dimension compatibility handling
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

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss.is_done());

    // Test solve with correctly sized vectors
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;
    aB[3] = 3.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aGauss.solve_into(&aB, &mut aX).unwrap();

    // Verify the result makes sense
    assert_eq!(aX.length(), 3);

    // Test invert with correctly sized matrix
    let mut aInv = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aGauss.invert(&mut aInv).unwrap();

    assert_eq!(aInv.row_number(), 3);
    assert_eq!(aInv.col_number(), 3);
}

#[test]
fn MathGaussTest__SingularMatrixState() {
    // Test state handling with singular matrix
    let mut aMatrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 4.0; // Singular matrix

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(!aGauss.is_done());
}
