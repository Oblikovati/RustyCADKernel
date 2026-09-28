#[test]
fn MathLin_Gauss_Test__TwoByTwoNonsymmetricSystem() {
    let mut aA: Matrix = Matrix::with_bounds(0, 2 - 1, 0, 2 - 1).unwrap();
    *aA.change_at(0_usize, 0_usize).unwrap() = 0.2;
    *aA.change_at(0_usize, 1_usize).unwrap() = 3.1;
    *aA.change_at(1_usize, 0_usize).unwrap() = -2.4;
    *aA.change_at(1_usize, 1_usize).unwrap() = 1.7;

    let mut aB: Vector = Vector::with_size(2usize);
    *aB.change_at(0_usize).unwrap() = 7.3;
    *aB.change_at(1_usize).unwrap() = -1.2;

    let aResult = solve(&aA, &aB, 1e-15);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    assert!(aResult.determinant.is_some());

    let THE_EXPECTED_SOLUTION: [f64; 2] = [2.0732647814910026, 2.2210796915167097];
    for i in 0..2 {
        near(
            *aResult.solution.as_ref().unwrap().at(i).unwrap(),
            THE_EXPECTED_SOLUTION[i],
            1.0e-13,
        );
        let mut aResidual: f64 = -*aB.at(i).unwrap();
        for j in 0..2 {
            aResidual += *aA.at(i, (j) as usize).unwrap()
                * *aResult.solution.as_ref().unwrap().at((j) as usize).unwrap();
        }
        near(aResidual, 0.0, 1.0e-13);
    }
    near(aResult.determinant.unwrap(), 7.78, 1.0e-13);
}

#[test]
fn MathLin_Gauss_Test__FourByFourNonsymmetricSystem() {
    let THE_A: [[f64; 4]; 4] = [
        [3.0, -2.0, 0.5, 1.0],
        [1.0, 4.0, -1.0, 2.0],
        [-2.0, 0.75, 5.0, -1.0],
        [0.5, -3.0, 2.0, 6.0],
    ];
    let THE_B: [f64; 4] = [4.5, -2.0, 7.25, 1.0];
    let mut aA: Matrix = Matrix::with_bounds(0, 4 - 1, 0, 4 - 1).unwrap();
    let mut aB: Vector = Vector::with_size(4usize);
    for i in 0..4 {
        *aB.change_at(i).unwrap() = THE_B[i];
        for j in 0..4 {
            *aA.change_at(i, j).unwrap() = THE_A[i][j];
        }
    }

    let aResult = solve(&aA, &aB, 1e-15);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    assert!(aResult.determinant.is_some());

    let THE_EXPECTED_SOLUTION: [f64; 4] = [
        1.3449494949494951,
        -0.065_151_515_151_515_11,
        1.877020202020202,
        -0.603_661_616_161_616_2,
    ];
    for i in 0..4 {
        near(
            *aResult.solution.as_ref().unwrap().at(i).unwrap(),
            THE_EXPECTED_SOLUTION[i],
            1.0e-13,
        );
        let mut aResidual: f64 = -*aB.at(i).unwrap();
        for j in 0..4 {
            aResidual += *aA.at(i, (j) as usize).unwrap()
                * *aResult.solution.as_ref().unwrap().at((j) as usize).unwrap();
        }
        near(aResidual, 0.0, 1.0e-12);
    }
    near(aResult.determinant.unwrap(), 495.0, 1.0e-11);
}
