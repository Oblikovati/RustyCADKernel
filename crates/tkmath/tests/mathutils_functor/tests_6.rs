#[test]
fn MathUtils_Functor_Vector__LinearResidual_NonZero() {
    let mut aA = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    aA[(1, 1)] = 1.0;
    aA[(2, 2)] = 1.0;

    let mut aB = Vector::with_bounds(1, 2).unwrap();
    aB[1] = 3.0;
    aB[2] = 4.0;

    let mut aRes = LinearResidual::new(&aA, &aB);

    let mut aX = Vector::with_bounds(1, 2).unwrap();
    aX[1] = 0.0;
    aX[2] = 0.0;

    let mut aValue: f64 = 0.0;
    assert!(aRes.value(&aX, &mut aValue));
    near(aValue, 25.0, THE_TOLERANCE);
}

#[test]
fn MathUtils_Functor_Integration__Polynomial_WithBrent() {
    let mut aPoly = Polynomial::new(&[-2.0, -1.0, 0.0, 1.0]);

    let mut aResult = brent(&mut aPoly, 1.0, 2.0, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.root.is_some());

    let mut aValue: f64 = 0.0;
    aPoly.value(aResult.root.unwrap(), &mut aValue);
    near(aValue, 0.0, 1e-10);
}

#[test]
fn MathUtils_Functor_Integration__Sine_RootFinding() {
    let mut aSine = Sine::default();

    let mut aResult = brent(&mut aSine, 2.0, 4.0, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.root.is_some());
    near(aResult.root.unwrap(), std::f64::consts::PI, 1e-10);
}

#[test]
fn MathUtils_Functor_Integration__Sphere_WithPowell() {
    let mut aSphere = Sphere::new(2);

    let mut aStart = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 5.0;
    aStart[2] = 5.0;

    let mut aResult = powell(&mut aSphere, &aStart, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        0.0,
        1e-5,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        0.0,
        1e-5,
    );
}

#[test]
fn MathUtils_Functor_Integration__Booth_WithBFGS() {
    let mut aBooth = Booth;

    let mut aStart = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;

    let mut aResult = bfgs(&mut aBooth, &aStart, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1e-5,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        3.0,
        1e-5,
    );
}

#[test]
fn MathUtils_Functor_Integration__Rosenbrock_WithBFGS() {
    let mut aRosen = Rosenbrock::default();

    let mut aStart = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = -1.0;
    aStart[2] = 1.0;

    let mut aConfig: NDimConfig = NDimConfig::default();
    aConfig.base.max_iterations = 500;
    aConfig.base.f_tolerance = 1e-10;
    aConfig.base.x_tolerance = 1e-10;

    let mut aResult = bfgs(&mut aRosen, &aStart, &aConfig.base);
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1e-4,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        1e-4,
    );
}

#[test]
fn MathUtils_Functor_Integration__Quadratic_WithNewton() {
    let mut aA = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    aA[(1, 1)] = 2.0;
    aA[(2, 2)] = 4.0;

    let mut aB = Vector::with_bounds(1, 2).unwrap();
    aB[1] = -4.0;
    aB[2] = -8.0;

    let mut aQuad = QuadraticForm::new(&aA, &aB, 10.0);

    let mut aStart = Vector::with_bounds(1, 2).unwrap();
    aStart[1] = 0.0;
    aStart[2] = 0.0;

    let mut aResult = bfgs(&mut aQuad, &aStart, &Config::default());
    assert!(aResult.is_done());
    assert!(aResult.solution.is_some());
    near(
        *aResult.solution.as_ref().unwrap().at(0).unwrap(),
        1.0,
        1e-5,
    );
    near(
        *aResult.solution.as_ref().unwrap().at(1).unwrap(),
        1.0,
        1e-5,
    );
}
