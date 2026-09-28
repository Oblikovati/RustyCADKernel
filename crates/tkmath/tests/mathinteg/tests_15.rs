#[test]
fn MathInteg_CorrectnessTest__SingularSampleIsNumericalError() {
    #[derive(Default)]
    struct SingularFunc;
    impl Function for SingularFunc {
        fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
            let _ = (&theX, &theF);
            *theF = 1.0 / theX;
            true
        }
    }

    let mut aFunc: SingularFunc = SingularFunc;
    assert_eq!(
        gauss(&mut aFunc, -1.0, 1.0, 3).status,
        Status::NumericalError
    );
}

#[test]
fn MathInteg_MultipleTest__ValidatesDimensionsAndPropagatesCallbackFailure() {
    #[derive(Default)]
    struct MultipleFunc;
    impl MultipleVarFunction for MultipleFunc {
        fn variables(&self) -> i32 {
            2
        }
        fn value(&mut self, _point: &Vector<'_>, theF: &mut f64) -> bool {
            let _ = &theF;
            false
        }
    }

    let mut aFunc: MultipleFunc = MultipleFunc;
    let mut aLower: Vector = Vector::filled(3, 4, 0.0).unwrap();
    let mut anUpper: Vector = Vector::filled(5, 6, 1.0).unwrap();
    let mut anOrders: Array1<i32> = Array1::with_bounds(7, 8).unwrap();
    anOrders.init(2);
    let mut aResult: IntegResult = gauss_multiple(
        &mut aFunc,
        2,
        &aLower,
        &anUpper,
        &anOrders,
        &MultipleConfig::default(),
    );
    assert_eq!(aResult.status, Status::CallbackError);
    assert_eq!(aResult.number_of_points, 1);

    anUpper[5] = 0.0;
    aResult = gauss_multiple(
        &mut aFunc,
        2,
        &aLower,
        &anUpper,
        &anOrders,
        &MultipleConfig::default(),
    );
    assert!(aResult.is_done());
    assert!(aResult.value.is_some());
    assert_eq!(aResult.value.unwrap(), 0.0);
    assert_eq!(aResult.number_of_points, 0);
}

#[test]
fn MathInteg_SetTest__UsesArbitraryBoundsAndRejectsNonFiniteComponents() {
    #[derive(Default)]
    struct SetFunc;
    impl FunctionSet for SetFunc {
        fn variables(&self) -> i32 {
            1
        }
        fn equations(&self) -> i32 {
            2
        }
        fn value(&mut self, _point: &Vector<'_>, theValues: &mut Vector<'_>) -> bool {
            let _ = &theValues;
            *theValues.change_at(0).unwrap() = 1.0;
            *theValues.change_at(1).unwrap() = f64::INFINITY;
            true
        }
    }

    let mut aFunc: SetFunc = SetFunc;
    let mut aResult: SetResult = gauss_set(&mut aFunc, 0.0, 1.0, 3);
    assert_eq!(aResult.status, Status::NumericalError);
    assert_eq!(aResult.nb_points, 1);
}
