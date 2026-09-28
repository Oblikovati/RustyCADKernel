#[test]
fn MathGaussTest__CustomBounds() {
    // Test with non-standard matrix bounds
    let mut aMatrix = Matrix::with_bounds(2, 4, 3, 5).unwrap();
    aMatrix[(2, 3)] = 2.0;
    aMatrix[(2, 4)] = 1.0;
    aMatrix[(2, 5)] = 1.0;
    aMatrix[(3, 3)] = 1.0;
    aMatrix[(3, 4)] = 3.0;
    aMatrix[(3, 5)] = 2.0;
    aMatrix[(4, 3)] = 1.0;
    aMatrix[(4, 4)] = 2.0;
    aMatrix[(4, 5)] = 3.0;

    let aGauss = Gauss::new(&aMatrix).unwrap();
    assert!(aGauss.is_done());

    let mut aB = Vector::with_bounds(2, 4).unwrap();
    aB[2] = 6.0;
    aB[3] = 11.0;
    aB[4] = 13.0; // Should give solution [0.75, 1.25, 3.25]

    let mut aX = Vector::with_bounds(3, 5).unwrap();
    aGauss.solve_into(&aB, &mut aX).unwrap();

    assert_near(aX[3], 0.75, 1.0e-10);
    assert_near(aX[4], 1.25, 1.0e-10);
    assert_near(aX[5], 3.25, 1.0e-10);
}
