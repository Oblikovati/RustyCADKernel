#[test]
fn MathInteg_SetTest__CallbackFailureIsDistinctFromNonFiniteComponents() {
    #[derive(Default)]
    struct FailingSetFunc;
    impl FunctionSet for FailingSetFunc {
        fn variables(&self) -> i32 {
            1
        }
        fn equations(&self) -> i32 {
            1
        }
        fn value(&mut self, _point: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
            let _ = &theF;
            false
        }
    }

    let mut aFunc: FailingSetFunc = FailingSetFunc;
    let mut aResult: SetResult = gauss_set(&mut aFunc, 0.0, 1.0, 3);
    assert_eq!(aResult.status, Status::CallbackError);
    assert_eq!(aResult.nb_points, 1);

    aResult = gauss_set(&mut aFunc, 1.0, 1.0, 100);
    assert!(aResult.is_done());
    assert!(aResult.values.is_some());
    assert_eq!(*aResult.values.as_ref().unwrap().at(0).unwrap(), 0.0);
    assert_eq!(aResult.nb_points, 0);
}

#[test]
fn MathInteg_BoundsTest__FiniteRulesPreserveIntervalOrientation() {
    #[derive(Default)]
    struct MultipleConstantFunc;
    impl MultipleVarFunction for MultipleConstantFunc {
        fn variables(&self) -> i32 {
            2
        }
        fn value(&mut self, _point: &Vector<'_>, theF: &mut f64) -> bool {
            let _ = &theF;
            *theF = 1.0;
            true
        }
    }
    #[derive(Default)]
    struct SetConstantFunc;
    impl FunctionSet for SetConstantFunc {
        fn variables(&self) -> i32 {
            1
        }
        fn equations(&self) -> i32 {
            1
        }
        fn value(&mut self, _point: &Vector<'_>, theF: &mut Vector<'_>) -> bool {
            let _ = &theF;
            *theF.change_at(0).unwrap() = 1.0;
            true
        }
    }

    let mut aSinFunc: SinFunc = SinFunc;
    let mut aKronrod: IntegResult = kronrod_rule(&mut aSinFunc, std::f64::consts::PI, 0.0, 7);
    let mut aTanhSinh: IntegResult = tanh_sinh(
        &mut aSinFunc,
        std::f64::consts::PI,
        0.0,
        &DoubleExpConfig::default(),
    );
    let mut aMultipleFunc: MultipleConstantFunc = MultipleConstantFunc;
    let mut aLower: Vector = Vector::with_size(2);
    let mut anUpper: Vector = Vector::with_size(2);
    let mut anOrders: Array1<i32> = Array1::with_size(2);
    anOrders.init(2);
    *aLower.change_at(0).unwrap() = 1.0;
    *aLower.change_at(1).unwrap() = 0.0;
    *anUpper.change_at(0).unwrap() = 0.0;
    *anUpper.change_at(1).unwrap() = 2.0;
    let mut aMultiple: IntegResult = gauss_multiple(
        &mut aMultipleFunc,
        2,
        &aLower,
        &anUpper,
        &anOrders,
        &MultipleConfig::default(),
    );
    let mut aSetFunc: SetConstantFunc = SetConstantFunc;
    let mut aSet: SetResult = gauss_set(&mut aSetFunc, 2.0, 0.0, 3);

    assert!(aKronrod.is_done());
    assert!(aTanhSinh.is_done());
    assert!(aMultiple.is_done());
    assert!(aSet.is_done());
    assert!(aSet.values.is_some());
    near(aKronrod.value.unwrap(), -2.0, THE_TOLERANCE);
    near(aTanhSinh.value.unwrap(), -2.0, 1.0e-6);
    near(aMultiple.value.unwrap(), -2.0, THE_TOLERANCE);
    near(
        *aSet.values.as_ref().unwrap().at(0).unwrap(),
        -2.0,
        THE_TOLERANCE,
    );
}

#[test]
fn MathInteg_DoubleExpTest__TinyStepIsBoundedPerLevel() {
    #[derive(Default)]
    struct ZeroFunc;
    impl Function for ZeroFunc {
        fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
            let _ = (&theX, &theF);
            *theF = 0.0;
            true
        }
    }

    let mut aFunc: ZeroFunc = ZeroFunc;
    let mut aConfig: DoubleExpConfig = DoubleExpConfig::default();
    aConfig.step_factor = f64::from_bits(1);
    aConfig.max_points_per_level = 4;

    let mut aTanhResult: IntegResult = tanh_sinh(&mut aFunc, 0.0, 1.0, &aConfig);
    let mut anExpResult: IntegResult = exp_sinh(&mut aFunc, 0.0, &aConfig);
    let mut aSinhResult: IntegResult = sinh_sinh(&mut aFunc, &aConfig);

    assert_eq!(aTanhResult.status, Status::MaxIterations);
    assert_eq!(anExpResult.status, Status::MaxIterations);
    assert_eq!(aSinhResult.status, Status::MaxIterations);
    assert_eq!(aTanhResult.number_of_points, 4);
    assert_eq!(anExpResult.number_of_points, 4);
    assert_eq!(aSinhResult.number_of_points, 4);
}
