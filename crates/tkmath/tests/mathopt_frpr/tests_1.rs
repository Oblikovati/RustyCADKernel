#[test]
fn MathOpt_FRPRTest__Rosenbrock() {
    let mut aFunc: RosenbrockFunc = RosenbrockFunc::default();

    let mut aStart: Vector = Vector::with_size(2);
    *aStart.change_at(0).unwrap() = -1.0;
    *aStart.change_at(1).unwrap() = 1.0;

    let mut aConfig: FrprConfig = FrprConfig::default();
    aConfig.base.max_iterations = 2000;
    aConfig.base.tolerance = 1.0e-6;
    aConfig.formula = ConjugateGradientFormula::PolakRibiere;

    let aResult: VectorResult = frpr(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1.0e-3,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        1.0e-3,
    );
    near(aResult.value.unwrap(), 0.0, 1.0e-4);
}

#[test]
fn MathOpt_FRPRTest__Sphere3D() {
    let mut aFunc: SphereFunc = SphereFunc::default();

    let mut aStart: Vector = Vector::with_size(3);
    *aStart.change_at(0).unwrap() = 3.0;
    *aStart.change_at(1).unwrap() = -2.0;
    *aStart.change_at(2).unwrap() = 4.0;

    let mut aConfig: FrprConfig = FrprConfig::default();
    aConfig.base.max_iterations = 100;
    aConfig.base.tolerance = 1.0e-8;

    let aResult: VectorResult = frpr(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    for i in 1..=3 {
        near(
            *aResult
                .solution
                .as_ref()
                .unwrap()
                .at((i - 1) as usize)
                .unwrap(),
            0.0,
            THE_TOLERANCE,
        );
    }
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}

#[test]
fn MathOpt_FRPRTest__Sphere5D() {
    let mut aFunc: SphereFunc = SphereFunc::default();

    let mut aStart: Vector = Vector::with_size(5);
    for i in 0..aStart.length() as usize {
        *aStart.change_at(i).unwrap() = (i + 1) as f64;
    }

    let mut aConfig: FrprConfig = FrprConfig::default();
    aConfig.base.max_iterations = 100;
    aConfig.base.tolerance = 1.0e-8;

    let aResult: VectorResult = frpr(&mut aFunc, &aStart, &aConfig);

    assert!(aResult.is_done());
    for i in 1..=5 {
        near(
            *aResult
                .solution
                .as_ref()
                .unwrap()
                .at((i - 1) as usize)
                .unwrap(),
            0.0,
            THE_TOLERANCE,
        );
    }
    near(aResult.value.unwrap(), 0.0, THE_TOLERANCE);
}
