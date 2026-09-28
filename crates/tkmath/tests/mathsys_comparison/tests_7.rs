#[test]
fn MathSys_ComparisonTest__Newton_NonFiniteCallbackIsNumericalError() {
    let mut aSystem: FailingSystem = FailingSystem;
    let mut aStart: Vector = Vector::filled(1, 2, 0.0).unwrap();
    let mut aResult: SystemResult = newton_uniform(&mut aSystem, &aStart, 1.0e-12, 1.0e-12, 100);
    assert_eq!(aResult.status, Status::NumericalError);
    assert!(!(aResult.is_done()));
}

#[test]
fn MathSys_ComparisonTest__Newton_CallbackFalseIsCallbackError() {
    let mut aSystem: CallbackFailingSystem = CallbackFailingSystem;
    let mut aStart: Vector = Vector::filled(1, 2, 0.0).unwrap();
    let mut aResult: SystemResult = newton_uniform(&mut aSystem, &aStart, 1.0e-12, 1.0e-12, 100);
    assert_eq!(aResult.status, Status::CallbackError);
}
