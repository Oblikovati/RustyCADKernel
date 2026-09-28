#[test]
fn MathLin_Jacobi_Test__DenseFourByFourSymmetricSystem() {
    let THE_A: [[f64; 4]; 4] = [
        [6.0, 2.0, -1.0, 0.5],
        [2.0, 3.0, 0.75, -1.0],
        [-1.0, 0.75, 4.0, 1.5],
        [0.5, -1.0, 1.5, 2.0],
    ];
    let mut aA: Matrix = Matrix::with_bounds(0, 4 - 1, 0, 4 - 1).unwrap();
    for i in 0..4 {
        for j in 0..4 {
            *aA.change_at(i, j).unwrap() = THE_A[i][j];
        }
    }

    let aResult = jacobi(&aA, true);
    assert!(aResult.is_done());
    assert!(aResult.eigen_values.is_some());
    assert!(aResult.eigen_vectors.is_some());

    let THE_EXPECTED_EIGENVALUES: [f64; 4] = [
        7.122_313_076_817_515,
        4.781_749_934_638_442,
        3.0874669645313992,
        0.008_470_024_012_645_65,
    ];
    for j in 0..4 {
        let mut anEigenvalue: f64 = *aResult.eigen_values.as_ref().unwrap().at(j).unwrap();
        near(anEigenvalue, THE_EXPECTED_EIGENVALUES[j], 1.0e-11);
        let mut aResidualSq: f64 = 0.0;
        let mut aNormSq: f64 = 0.0;
        for i in 0..4 {
            let mut aResidual: f64 = -anEigenvalue
                * *aResult
                    .eigen_vectors
                    .as_ref()
                    .unwrap()
                    .at((i) as usize, j)
                    .unwrap();
            for k in 0..4 {
                aResidual += *aA.at((i) as usize, (k) as usize).unwrap()
                    * *aResult
                        .eigen_vectors
                        .as_ref()
                        .unwrap()
                        .at((k) as usize, j)
                        .unwrap();
            }
            aResidualSq += aResidual * aResidual;
            aNormSq += *aResult
                .eigen_vectors
                .as_ref()
                .unwrap()
                .at((i) as usize, j)
                .unwrap()
                * *aResult
                    .eigen_vectors
                    .as_ref()
                    .unwrap()
                    .at((i) as usize, j)
                    .unwrap();
        }
        near((aResidualSq).sqrt(), 0.0, 1.0e-11);
        near(aNormSq, 1.0, 1.0e-12);
    }
}

#[test]
fn MathLin_Jacobi_Test__EigenvectorsOrthogonal() {
    let aMat: Matrix = CreateSPD(3);

    let aResult = jacobi(&aMat, false);

    assert!(aResult.is_done());

    let aV: &Matrix = aResult.eigen_vectors.as_ref().unwrap();

    let aVtV: Matrix = matmul(&transpose(aV), aV);
    for i in 1..=3 {
        for j in 1..=3 {
            let mut aExpected: f64 = if i == j { 1.0 } else { 0.0 };
            near(aVtV[(i, j)], aExpected, 1.0e-8);
        }
    }
}
