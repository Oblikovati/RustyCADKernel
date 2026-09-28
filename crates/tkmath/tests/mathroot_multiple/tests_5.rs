#[test]
fn MathRoot_Multiple__FindAllRoots_ConvenienceOverload() {
    let mut aFunc = Polynomial::new(&[-4.0, 0.0, 1.0]);

    let mut aResult = find_all_roots_samples(&mut aFunc, -5.0, 5.0, 50);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 2);
    near(aResult.roots[0], -2.0, THE_TOLERANCE);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_Multiple__FindAllRoots_Exponential() {
    let mut aFunc = make_scalar(|x: f64, y: &mut f64| {
        *y = (x).exp() - 2.0;
        true
    });

    let mut aResult = find_all_roots_samples(&mut aFunc, -1.0, 2.0, 50);
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 1);
    near(aResult.roots[0], (2.0_f64).ln(), THE_TOLERANCE);
}

#[test]
fn MathRoot_Multiple__FindAllRootsWithDerivative_NegativeExtrema_BothProbeDirections() {
    let mut aFunc = make_scalar_with_derivative(|theX: f64, theY: &mut f64, theDY: &mut f64| {
        *theY = -(theX - 1.0) * (theX - 3.0);
        *theDY = -2.0 * theX + 4.0;
        true
    });

    let mut aConfig = MultipleConfig::default();
    aConfig.offset = 1.0;

    let mut aResult = find_all_roots_with_derivative(&mut aFunc, 0.0, 4.0, &aConfig);
    assert!(aResult.is_done());
    assert!(aResult.nb_roots() >= 1);

    let mut isFound: bool = false;
    for i in 0..aResult.nb_roots() {
        if (aResult.roots[i] - 2.0).abs() < THE_TOLERANCE {
            isFound = true;
            break;
        }
    }
    assert!(isFound);
}
