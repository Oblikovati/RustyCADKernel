#[test]
fn MathLin_LeastSquares_Test__NoisyQuadraticFit() {
    let THE_A: [[f64; 3]; 7] = [
        [1.0, -2.0, 4.0],
        [1.0, -1.0, 1.0],
        [1.0, -0.25, 0.0625],
        [1.0, 0.5, 0.25],
        [1.0, 1.0, 1.0],
        [1.0, 1.75, 3.0625],
        [1.0, 2.5, 6.25],
    ];
    let THE_B: [f64; 7] = [7.1, 2.4, 0.85, 1.2, 2.05, 4.8, 9.7];
    let mut aA: Matrix = Matrix::with_bounds(0, 7 - 1, 0, 3 - 1).unwrap();
    let mut aB: Vector = Vector::with_size(7usize);
    for i in 0..7 {
        *aB.change_at(i).unwrap() = THE_B[i];
        for j in 0..3 {
            *aA.change_at(i, j).unwrap() = THE_A[i][j];
        }
    }

    let aResult = least_squares(&aA, &aB, LeastSquaresMethod::Qr, 1e-15);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    assert!(aResult.residual.is_some());
    assert!(aResult.residual_sq.is_some());
    assert_eq!(aResult.rank, 3);

    let THE_EXPECTED_SOLUTION: [f64; 3] = [
        0.749_410_127_172_167_5,
        -0.18756729578259318,
        1.492_675_030_352_084,
    ];
    for i in 0..3 {
        near(
            *aResult.solution.as_ref().unwrap().at(i).unwrap(),
            THE_EXPECTED_SOLUTION[i],
            1.0e-12,
        );
    }
    near(aResult.residual.unwrap(), 0.2774950969170783, 1.0e-12);
    near(
        aResult.residual_sq.unwrap(),
        0.077_003_528_813_018_68,
        1.0e-12,
    );

    for j in 0..3 {
        let mut aNormalResidual: f64 = 0.0;
        for i in 0..7 {
            let mut aRowResidual: f64 = -*aB.at((i) as usize).unwrap();
            for k in 0..3 {
                aRowResidual += *aA.at((i) as usize, (k) as usize).unwrap()
                    * *aResult.solution.as_ref().unwrap().at((k) as usize).unwrap();
            }
            aNormalResidual += *aA.at((i) as usize, (j) as usize).unwrap() * aRowResidual;
        }
        near(aNormalResidual, 0.0, 1.0e-11);
    }
}

#[test]
fn MathLin_LeastSquares_Test__MethodComparison() {
    let aMat: Matrix = CreateRandom(5, 3);
    let mut aB: Vector = Vector::with_bounds(1, 5).unwrap();
    for i in 1..=5 {
        aB[i] = (i) as f64;
    }

    let aResultNE = least_squares(&aMat, &aB, LeastSquaresMethod::NormalEquations, 1e-15);
    let aResultQR = least_squares(&aMat, &aB, LeastSquaresMethod::Qr, 1e-15);
    let aResultSVD = least_squares(&aMat, &aB, LeastSquaresMethod::Svd, 1e-15);

    assert!(aResultNE.is_done());
    assert!(aResultQR.is_done());
    assert!(aResultSVD.is_done());

    for i in 0..aResultNE.solution.as_ref().unwrap().length() as usize {
        near(
            *aResultNE.solution.as_ref().unwrap().at(i).unwrap(),
            *aResultQR.solution.as_ref().unwrap().at(i).unwrap(),
            1.0e-6,
        );
        near(
            *aResultQR.solution.as_ref().unwrap().at(i).unwrap(),
            *aResultSVD.solution.as_ref().unwrap().at(i).unwrap(),
            1.0e-6,
        );
    }
}
