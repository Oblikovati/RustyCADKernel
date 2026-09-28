#[test]
fn MathRoot_Multiple__FindAllRoots_NoRoots() {
    let mut aFunc = Polynomial::new(&[1.0, 0.0, 1.0]);

    let mut aResult = find_all_roots(&mut aFunc, -10.0, 10.0, &MultipleConfig::default());
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 0);
}

#[test]
fn MathRoot_Multiple__FindAllRoots_SingleRoot() {
    let mut aFunc = Polynomial::new(&[-2.0, 1.0]);

    let mut aResult = find_all_roots(&mut aFunc, 0.0, 5.0, &MultipleConfig::default());
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 1);
    near(aResult.roots[0], 2.0, THE_TOLERANCE);
}

#[test]
fn MathRoot_Multiple__FindAllRoots_TwoRoots() {
    let mut aFunc = Polynomial::new(&[-4.0, 0.0, 1.0]);

    let mut aResult = find_all_roots(&mut aFunc, -5.0, 5.0, &MultipleConfig::default());
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_roots(), 2);
    near(aResult.roots[0], -2.0, THE_TOLERANCE);
    near(aResult.roots[1], 2.0, THE_TOLERANCE);
}
