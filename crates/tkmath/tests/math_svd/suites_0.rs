#[test]
fn MathSVDTest__WellConditionedSquareMatrix() {
    let aMatrix = create_well_conditioned_matrix();
    let mut aSVD = Svd::new(&aMatrix).unwrap();

    assert!(aSVD.is_done());

    // Test solving Ax = b where b = [6, 9, 8]
    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 6.0;
    aB[2] = 9.0;
    aB[3] = 8.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // Verify the solution
    check_solution(&aMatrix, &aX, &aB, 1e-10);
}

#[test]
fn MathSVDTest__IdentityMatrix() {
    // Identity matrix test
    let mut anIdentity = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    anIdentity[(1, 1)] = 1.0;
    anIdentity[(1, 2)] = 0.0;
    anIdentity[(1, 3)] = 0.0;
    anIdentity[(2, 1)] = 0.0;
    anIdentity[(2, 2)] = 1.0;
    anIdentity[(2, 3)] = 0.0;
    anIdentity[(3, 1)] = 0.0;
    anIdentity[(3, 2)] = 0.0;
    anIdentity[(3, 3)] = 1.0;

    let mut aSVD = Svd::new(&anIdentity).unwrap();
    assert!(aSVD.is_done());

    let mut aB = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 5.0;
    aB[2] = 7.0;
    aB[3] = 9.0;

    let mut aX = Vector::with_bounds(1, 3).unwrap();
    aSVD.solve(&aB, &mut aX, 1e-6).unwrap();

    // For identity matrix, X should equal B
    assert_near(aX[1], aB[1], 1e-7);
    assert_near(aX[2], aB[2], 1e-7);
    assert_near(aX[3], aB[3], 1e-7);
}
