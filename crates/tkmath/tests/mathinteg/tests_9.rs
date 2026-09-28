#[test]
fn MathInteg_BoundsTest__NegativeBounds() {
    let mut aFunc: QuadraticFunc = QuadraticFunc;

    let mut aResult: IntegResult = gauss(&mut aFunc, -1.0, 1.0, 7);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), 2.0 / 3.0, THE_TOLERANCE);
}

#[test]
fn MathInteg_BoundsTest__SmallInterval() {
    let mut aFunc: SinFunc = SinFunc;

    let mut aResult: IntegResult = gauss(&mut aFunc, 0.0, 0.001, 15);
    assert!(aResult.is_done());

    near(aResult.value.unwrap(), 0.0000005, 1.0e-10);
}

#[test]
fn MathInteg_BoundsTest__LargeInterval() {
    #[derive(Default)]
    struct LorentzianFunc;
    impl Function for LorentzianFunc {
        fn value(&mut self, theX: f64, theF: &mut f64) -> bool {
            let _ = (&theX, &theF);
            *theF = 1.0 / (1.0 + theX * theX);
            true
        }
    }

    let mut aFunc: LorentzianFunc = LorentzianFunc;

    let mut aResult: IntegResult = gauss_composite(&mut aFunc, -100.0, 100.0, 200, 15);
    assert!(aResult.is_done());
    near(aResult.value.unwrap(), std::f64::consts::PI, 0.05);
}
