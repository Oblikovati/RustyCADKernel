#[test]
fn MathCroutTest__SingularMatrixState() {
    // Test state handling for singular matrix decomposition
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 4.0;
    aMatrix[(2, 3)] = 6.0; // Singular
    aMatrix[(3, 1)] = 3.0;
    aMatrix[(3, 2)] = 6.0;
    aMatrix[(3, 3)] = 9.0;

    let aCrout = Crout::new(&aMatrix).unwrap();
    assert!(!aCrout.is_done());
}

#[test]
fn MathCroutTest__LargerMatrix() {
    // Test with larger 4x4 symmetric matrix
    let mut aMatrix = Matrix::with_bounds(1, 4, 1, 4).unwrap();
    aMatrix[(1, 1)] = 10.0;
    aMatrix[(1, 2)] = 1.0;
    aMatrix[(1, 3)] = 2.0;
    aMatrix[(1, 4)] = 3.0;
    aMatrix[(2, 1)] = 1.0;
    aMatrix[(2, 2)] = 10.0;
    aMatrix[(2, 3)] = 1.0;
    aMatrix[(2, 4)] = 4.0;
    aMatrix[(3, 1)] = 2.0;
    aMatrix[(3, 2)] = 1.0;
    aMatrix[(3, 3)] = 10.0;
    aMatrix[(3, 4)] = 1.0;
    aMatrix[(4, 1)] = 3.0;
    aMatrix[(4, 2)] = 4.0;
    aMatrix[(4, 3)] = 1.0;
    aMatrix[(4, 4)] = 10.0;

    let aCrout = Crout::new(&aMatrix).unwrap();

    assert!(aCrout.is_done());

    // Test solving
    let mut aB = Vector::with_bounds(1, 4).unwrap();
    aB[1] = 16.0;
    aB[2] = 16.0;
    aB[3] = 14.0;
    aB[4] = 18.0; // Should give solution approximately [1, 1, 1, 1]

    let mut aX = Vector::with_bounds(1, 4).unwrap();
    aCrout.solve_into(&aB, &mut aX).unwrap();

    // Verify solution by checking residual
    let mut aResidual = Vector::with_bounds(1, 4).unwrap();
    for i in 1..=4 {
        aResidual[i] = 0.0;
        for j in 1..=4 {
            aResidual[i] += aMatrix[(i, j)] * aX[j];
        }
        aResidual[i] -= aB[i];
    }

    let mut aResidualNorm: f64 = 0.0;
    for i in 1..=4 {
        aResidualNorm += aResidual[i] * aResidual[i];
    }

    assert!(aResidualNorm < 1.0e-20);
}

#[test]
fn MathCroutTest__CustomBounds() {
    // Test with custom matrix bounds
    let mut aMatrix = Matrix::with_bounds(2, 4, 3, 5).unwrap();
    aMatrix[(2, 3)] = 4.0;
    aMatrix[(2, 4)] = 2.0;
    aMatrix[(2, 5)] = 1.0;
    aMatrix[(3, 3)] = 2.0;
    aMatrix[(3, 4)] = 3.0;
    aMatrix[(3, 5)] = 0.5;
    aMatrix[(4, 3)] = 1.0;
    aMatrix[(4, 4)] = 0.5;
    aMatrix[(4, 5)] = 2.0;

    let aCrout = Crout::new(&aMatrix).unwrap();

    assert!(aCrout.is_done());

    let mut aB = Vector::with_bounds(2, 4).unwrap();
    aB[2] = 7.0;
    aB[3] = 5.5;
    aB[4] = 3.5;

    let mut aX = Vector::with_bounds(3, 5).unwrap();
    aCrout.solve_into(&aB, &mut aX).unwrap();

    // Verify the solution makes sense
    assert!(aX[3] > 0.0);
    assert!(aX[4] > 0.0);
    assert!(aX[5] > 0.0);
}
