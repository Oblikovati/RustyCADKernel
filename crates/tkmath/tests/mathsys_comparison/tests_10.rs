#[test]
fn MathSys_ComparisonTest__Newton_MixedSpaceSystem() {
    let mut aFunction: MixedSpaceFunctionSet = MixedSpaceFunctionSet;
    let mut aStart: Vector = Vector::with_size(3);
    *aStart.change_at(0).unwrap() = 0.8;
    *aStart.change_at(1).unwrap() = 1.0;
    *aStart.change_at(2).unwrap() = 0.7;

    let mut aResult: SystemResult = newton_uniform(&mut aFunction, &aStart, 1.0e-12, 1.0e-12, 100);
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.968_891_565_512_260_1,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.909_606_514_261_007_2,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(2).unwrap(),
        0.651_642_620_018_194_4,
        1.0e-10,
    );
    assert!(aResult.residual_norm <= 1.0e-12);
    let mut aResidual: Vector = Vector::with_size(3);
    assert!(aFunction.value(aResult.solution.as_ref().unwrap(), &mut aResidual));
    for anIndex in 0..aResidual.length() as usize {
        near(*aResidual.at(anIndex).unwrap(), 0.0, 1.0e-12);
    }
}

#[test]
fn MathSys_ComparisonTest__Newton_CubicSpaceSystem() {
    let mut aFunction: CubicSpaceFunctionSet = CubicSpaceFunctionSet;
    let mut aStart: Vector = Vector::filled(0, 3 - 1, 0.8).unwrap();

    let mut aResult: SystemResult = newton_uniform(&mut aFunction, &aStart, 1.0e-12, 1.0e-12, 100);
    assert_eq!(aResult.status, Status::Ok);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.852_342_944_260_091,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.831_375_803_289_385_1,
        1.0e-10,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(2).unwrap(),
        0.889_596_931_220_947_5,
        1.0e-10,
    );
    assert!(aResult.residual_norm <= 1.0e-12);
    let mut aResidual: Vector = Vector::with_size(3);
    assert!(aFunction.value(aResult.solution.as_ref().unwrap(), &mut aResidual));
    for anIndex in 0..aResidual.length() as usize {
        near(*aResidual.at(anIndex).unwrap(), 0.0, 1.0e-12);
    }
}
