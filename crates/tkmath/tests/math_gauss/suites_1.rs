#[test]
fn MathGaussTest__DiagonalMatrix() {
    // Test with diagonal matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 2.0;
    aMatrix[(1, 2)] = 0.0;
    aMatrix[(1, 3)] = 0.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 3.0;
    aMatrix[(2, 3)] = 0.0;
    aMatrix[(3, 1)] = 0.0;
    aMatrix[(3, 2)] = 0.0;
    aMatrix[(3, 3)] = 4.0;

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss.is_done());

    // For diagonal matrix: x_i = b_i / a_ii
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 8.0;
    aB[2] = 15.0;
    aB[3] = 20.0; // Expected solution: [4, 5, 5]

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aGauss.solve_into(&aB, &mut aX).unwrap();

    assert_near(aX[1], 4.0, 1e-7);
    assert_near(aX[2], 5.0, 1e-7);
    assert_near(aX[3], 5.0, 1e-7);
}

#[test]
fn MathGaussTest__InPlaceSolve() {
    // Test the in-place solve method where B is overwritten with solution
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 5.0;
    aMatrix[(2, 3)] = 8.0;
    aMatrix[(3, 1)] = 3.0;
    aMatrix[(3, 2)] = 8.0;
    aMatrix[(3, 3)] = 14.0;

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss.is_done());

    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 14.0;
    aB[2] = 31.0;
    aB[3] = 53.0; // Should give solution [13, -7, 5]

    aGauss.solve(&mut aB).unwrap(); // In-place solve

    // Now B contains the solution
    assert_near(aB[1], 13.0, 1.0e-10);
    assert_near(aB[2], -7.0, 1.0e-10);
    assert_near(aB[3], 5.0, 1.0e-10);
}

#[test]
fn MathGaussTest__Determinant() {
    // Test determinant calculation
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 0.0;
    aMatrix[(2, 2)] = 1.0;
    aMatrix[(2, 3)] = 4.0;
    aMatrix[(3, 1)] = 5.0;
    aMatrix[(3, 2)] = 6.0;
    aMatrix[(3, 3)] = 0.0;

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss.is_done());

    let aDet: f64 = aGauss.determinant().unwrap();
    // Expected determinant: 1*(1*0 - 4*6) - 2*(0*0 - 4*5) + 3*(0*6 - 1*5) = -24 + 40 - 15 = 1
    assert_near(aDet, 1.0, 1.0e-12);
}
