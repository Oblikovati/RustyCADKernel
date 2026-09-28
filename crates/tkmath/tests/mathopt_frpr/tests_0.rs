#[test]
fn MathOpt_FRPRTest__Quadratic_FletcherReeves() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfig: FrprConfig = FrprConfig::new(1.0e-8, 100);
    aConfig.formula = ConjugateGradientFormula::FletcherReeves;

    let aResult: VectorResult = frpr(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_FRPRTest__Quadratic_PolakRibiere() {
    let mut aFunc: QuadraticFunc = QuadraticFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 5.0;
    *aStart.change_at(1).unwrap() = 7.0;

    let mut aConfig: FrprConfig = FrprConfig::default();
    aConfig.base.max_iterations = 100;
    aConfig.base.tolerance = 1.0e-8;
    aConfig.formula = ConjugateGradientFormula::PolakRibiere;

    let aResult: VectorResult = frpr(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        2.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_FRPRTest__Booth() {
    let mut aFunc: BoothFunc = BoothFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = 0.0;
    *aStart.change_at(1).unwrap() = 0.0;

    let mut aConfig: FrprConfig = FrprConfig::new(1.0e-8, 100);

    let aResult: VectorResult = frpr(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        THE_TOLERANCE,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        3.0,
        THE_TOLERANCE,
    );
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}
