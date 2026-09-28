#[test]
fn MathLin_ComparisonTest__LU_Decomposition() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 2.0;
    aA[(1, 2)] = 1.0;
    aA[(1, 3)] = 1.0;
    aA[(2, 1)] = 4.0;
    aA[(2, 2)] = 3.0;
    aA[(2, 3)] = 3.0;
    aA[(3, 1)] = 8.0;
    aA[(3, 2)] = 7.0;
    aA[(3, 3)] = 9.0;

    let aLUResult = lu(&aA, 1e-15);
    assert!(aLUResult.is_done());
    assert!(aLUResult.lu.is_some());
    assert!(aLUResult.pivot.is_some());
    assert!(aLUResult.determinant.is_some());

    let aLUMat: &Matrix = aLUResult.lu.as_ref().unwrap();

    let mut aL: Matrix = Matrix::filled(0, 3 - 1, 0, 3 - 1, 0.0).unwrap();
    let mut aU: Matrix = Matrix::filled(0, 3 - 1, 0, 3 - 1, 0.0).unwrap();

    for i in 0..aLUMat.row_number() as usize {
        *aL.change_at(i, i).unwrap() = 1.0;
        for j in 0..i {
            *aL.change_at(i, j).unwrap() = *aLUMat.at(i, j).unwrap();
        }
        for j in i..aLUMat.col_number() as usize {
            *aU.change_at(i, j).unwrap() = *aLUMat.at(i, j).unwrap();
        }
    }

    let mut aLU: Matrix = aL.multiplied_matrix(&aU).unwrap();
    let _ = aLU;

    let mut aB: Vector = Vector::with_bounds(1, 3).unwrap();
    aB[1] = 1.0;
    aB[2] = 2.0;
    aB[3] = 3.0;

    let aSolveResult = solve(&aA, &aB, 1e-15);
    assert!(aSolveResult.is_done());

    let mut aResidual: Vector = aA
        .multiplied_vector(aSolveResult.solution.as_ref().unwrap())
        .unwrap();
    for i in 0..aResidual.length() as usize {
        *aResidual.change_at(i).unwrap() -= *aB.at(i).unwrap();
        near(*aResidual.at(i).unwrap(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn MathLin_ComparisonTest__SingularMatrix_Detection() {
    let mut aA: Matrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(1, 2)] = 2.0;
    aA[(1, 3)] = 3.0;
    aA[(2, 1)] = 4.0;
    aA[(2, 2)] = 5.0;
    aA[(2, 3)] = 6.0;
    aA[(3, 1)] = 1.0;
    aA[(3, 2)] = 2.0;
    aA[(3, 3)] = 3.0;

    let aNewResult = lu(&aA, 1e-15);
    assert!(!(aNewResult.is_done()));

    let aOldGauss = Gauss::new(&aA).unwrap();
    assert!(!(aOldGauss.is_done()));
}

#[test]
fn MathLin_ComparisonTest__IllConditioned_HilbertMatrix() {
    let aN: i32 = 4;
    let mut aH: Matrix = Matrix::with_bounds(1, aN, 1, aN).unwrap();
    for i in 1..=aN {
        for j in 1..=aN {
            aH[(i, j)] = 1.0 / (i + j - 1) as f64;
        }
    }

    let mut aB: Vector = Vector::with_bounds(1, aN).unwrap();
    for i in 1..=aN {
        aB[i] = 1.0;
    }

    let aNewResult = solve(&aH, &aB, 1e-15);
    assert!(aNewResult.is_done());

    let aOldGauss = Gauss::new(&aH).unwrap();
    assert!(aOldGauss.is_done());
    let mut aOldX: Vector = Vector::with_bounds(1, aN).unwrap();
    aOldGauss.solve_into(&aB, &mut aOldX).unwrap();

    let aLooseTol: f64 = 1.0e-6;
    for i in 0..aNewResult.solution.as_ref().unwrap().length() as usize {
        near(
            *aNewResult.solution.as_ref().unwrap().at(i).unwrap(),
            *aOldX.at(i).unwrap(),
            aLooseTol,
        );
    }
}
