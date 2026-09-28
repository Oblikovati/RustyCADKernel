#[test]
fn MathOpt_PSOTest__AllRejectedParticlesPreserveCallbackStatus() {
    let mut aFunc: RejectAllFunc = RejectAllFunc;
    let mut aLower: Vector = Vector::filled(0, 1 - 1, -1.0).unwrap();
    let mut anUpper: Vector = Vector::filled(0, 1 - 1, 1.0).unwrap();
    let mut aConfig: PSOConfig = PSOConfig::default();
    aConfig.particles = 4;

    assert_eq!(
        pso(&mut aFunc, &aLower, &anUpper, &aConfig).status,
        Status::CallbackError
    );
}
