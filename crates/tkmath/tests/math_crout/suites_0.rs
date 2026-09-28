#[test]
fn MathCroutTest__SimpleSymmetricMatrix() {
    // Test with a simple 3x3 symmetric positive definite matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 4.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 1.0;
    aMatrix[(2, 1)] = 2.0;
    aMatrix[(2, 2)] = 3.0;
    aMatrix[(2, 3)] = 0.5;
    aMatrix[(3, 1)] = 1.0;
    aMatrix[(3, 2)] = 0.5;
    aMatrix[(3, 3)] = 2.0;

    let aCrout = Crout::new(&aMatrix).unwrap();

    assert!(aCrout.is_done());

    // Test solving a system
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 7.0;
    aB[2] = 5.5;
    aB[3] = 3.5; // Expected solution: [1, 1, 1]

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aCrout.solve_into(&aB, &mut aX).unwrap();

    assert_near(aX[1], 1.0, 1.0e-10);
    assert_near(aX[2], 1.0, 1.0e-10);
    assert_near(aX[3], 1.0, 1.0e-10);
}

#[test]
fn MathCroutTest__IdentityMatrix() {
    // Test with identity matrix
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

    let aCrout = Crout::new(&aMatrix).unwrap();

    assert!(aCrout.is_done());

    // Test solving with identity matrix
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 5.0;
    aB[2] = 7.0;
    aB[3] = 9.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aCrout.solve_into(&aB, &mut aX).unwrap();

    assert_near(aX[1], 5.0, 1.0e-12);
    assert_near(aX[2], 7.0, 1.0e-12);
    assert_near(aX[3], 9.0, 1.0e-12);

    // Check inverse matrix
    let aInverse = aCrout.inverse().unwrap();
    assert_near(aInverse[(1, 1)], 1.0, 1.0e-12);
    assert_near(aInverse[(2, 2)], 1.0, 1.0e-12);
    assert_near(aInverse[(3, 3)], 1.0, 1.0e-12);
}
