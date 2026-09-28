//=================================================================================================
// CSLib_NormalPolyDef Tests
//=================================================================================================

// Test Value function
#[test]
fn CSLibNormalPolyDefTest__Value_ConstantPolynomial() {
    // k0 = 0, lambda_0 = 1.0 -> f(X) = 1 * cos^0 * sin^0 = 1 (constant)
    let aLambda = coefficients(&[1.0]);

    let mut aPoly = NormalPolyDef::new(0, &aLambda).unwrap();

    let mut aValue: f64 = 0.0;
    assert!(aPoly.value(0.5, &mut aValue));
    assert_near(aValue, 1.0, 1e-10, "value");

    assert!(aPoly.value(1.0, &mut aValue));
    assert_near(aValue, 1.0, 1e-10, "value");
}

#[test]
fn CSLibNormalPolyDefTest__Value_LinearPolynomial() {
    // k0 = 1, lambda = [1, 1] -> f(X) = cos(X) + sin(X)
    let aLambda = coefficients(&[1.0, 1.0]);

    let mut aPoly = NormalPolyDef::new(1, &aLambda).unwrap();

    let mut aValue: f64 = 0.0;

    // At X = pi/4, cos(X) = sin(X) = sqrt(2)/2
    // f(pi/4) = cos(pi/4) + sin(pi/4) = sqrt(2)
    assert!(aPoly.value(PI / 4.0, &mut aValue));
    assert_near(aValue, 2.0_f64.sqrt(), 1e-10, "value");
}

#[test]
fn CSLibNormalPolyDefTest__Value_AtSingularPoints() {
    // At singular points (X=0, pi/2, pi, 3*pi/2), the function returns 0
    // as a safety measure when computing would involve 0^n terms.
    // The early return protects against numerical issues.
    let aLambda = coefficients(&[1.0, 1.0, 1.0]);

    let mut aPoly = NormalPolyDef::new(2, &aLambda).unwrap();

    let mut aValue: f64 = 0.0;

    // At X = 0, sin(0) = 0; at X = pi/2, cos(pi/2) ~ 0
    // The function is designed to return 0 for these edge cases
    // to avoid numerical issues, but actual return depends on
    // the tolerance check RealSmall().
    // Test that the function doesn't crash at these points.
    assert!(aPoly.value(0.0, &mut aValue));
    assert!(!aValue.is_nan());
    assert!(!is_infinite(aValue));

    assert!(aPoly.value(PI / 2.0, &mut aValue));
    assert!(!aValue.is_nan());
    assert!(!is_infinite(aValue));
}

// Test Derivative function
#[test]
fn CSLibNormalPolyDefTest__Derivative_AtRegularPoint() {
    // k0 = 2, test derivative at non-singular point
    let aLambda = coefficients(&[1.0, 0.0, 1.0]);

    let mut aPoly = NormalPolyDef::new(2, &aLambda).unwrap();

    let mut aDeriv: f64 = 0.0;
    assert!(aPoly.derivative(PI / 4.0, &mut aDeriv));
    // Derivative should be computed without crash
    assert!(!aDeriv.is_nan());
    assert!(!is_infinite(aDeriv));
}

#[test]
fn CSLibNormalPolyDefTest__Derivative_AtSingularPoint() {
    let aLambda = coefficients(&[1.0, 1.0, 1.0]);

    let mut aPoly = NormalPolyDef::new(2, &aLambda).unwrap();

    let mut aDeriv: f64 = 0.0;

    // At singular points, derivative should be 0
    assert!(aPoly.derivative(0.0, &mut aDeriv));
    assert_near(aDeriv, 0.0, 1e-10, "derivative");
}

// Test Values function (combined value and derivative)
#[test]
fn CSLibNormalPolyDefTest__Values_Consistency() {
    let aLambda = coefficients(&[1.0, 2.0, 1.0]);

    let mut aPoly = NormalPolyDef::new(2, &aLambda).unwrap();

    let (mut aF1, mut aD1): (f64, f64) = (0.0, 0.0);
    let (mut aF2, mut aD2): (f64, f64) = (0.0, 0.0);

    // Get values using separate functions
    assert!(aPoly.value(0.7, &mut aF1));
    assert!(aPoly.derivative(0.7, &mut aD1));

    // Get values using combined function
    assert!(aPoly.values(0.7, &mut aF2, &mut aD2));

    // Results should be consistent
    assert_near(aF1, aF2, 1e-10, "value");
    assert_near(aD1, aD2, 1e-10, "derivative");
}

// Test numerical derivative approximation
#[test]
fn CSLibNormalPolyDefTest__Derivative_NumericalApproximation() {
    let aLambda = coefficients(&[1.0, 2.0, 3.0]);

    let mut aPoly = NormalPolyDef::new(2, &aLambda).unwrap();

    let aX: f64 = 0.8; // Away from singular points
    let aH: f64 = 1e-6;

    let (mut aFMinus, mut aFPlus, mut aAnalyticDeriv): (f64, f64, f64) = (0.0, 0.0, 0.0);

    assert!(aPoly.value(aX - aH, &mut aFMinus));
    assert!(aPoly.value(aX + aH, &mut aFPlus));
    assert!(aPoly.derivative(aX, &mut aAnalyticDeriv));

    let aNumericDeriv: f64 = (aFPlus - aFMinus) / (2.0 * aH);

    // Analytical derivative should match numerical approximation
    assert_near(aAnalyticDeriv, aNumericDeriv, 1e-4, "derivative");
}

//=================================================================================================
// CSLib Enumeration Tests
//=================================================================================================

#[test]
fn CSLibEnumTest__DerivativeStatus_AllValues() {
    // Verify all enum values are distinct
    assert_ne!(DerivativeStatus::Done, DerivativeStatus::D1uIsNull);
    assert_ne!(DerivativeStatus::Done, DerivativeStatus::D1vIsNull);
    assert_ne!(DerivativeStatus::Done, DerivativeStatus::D1IsNull);
    assert_ne!(DerivativeStatus::Done, DerivativeStatus::D1uD1vRatioIsNull);
    assert_ne!(DerivativeStatus::Done, DerivativeStatus::D1vD1uRatioIsNull);
    assert_ne!(DerivativeStatus::Done, DerivativeStatus::D1uIsParallelD1v);
}

#[test]
fn CSLibEnumTest__NormalStatus_AllValues() {
    // Verify all enum values are distinct
    assert_ne!(NormalStatus::Singular, NormalStatus::Defined);
    assert_ne!(NormalStatus::Singular, NormalStatus::InfinityOfSolutions);
    assert_ne!(NormalStatus::Singular, NormalStatus::D1NuIsNull);
    assert_ne!(NormalStatus::Singular, NormalStatus::D1NvIsNull);
    assert_ne!(NormalStatus::Singular, NormalStatus::D1NIsNull);
    assert_ne!(NormalStatus::Singular, NormalStatus::D1NuNvRatioIsNull);
    assert_ne!(NormalStatus::Singular, NormalStatus::D1NvNuRatioIsNull);
    assert_ne!(NormalStatus::Singular, NormalStatus::D1NuIsParallelD1Nv);
}

//=================================================================================================
// CSLib Integration Tests
//=================================================================================================

// Test normal computation for a planar surface
#[test]
fn CSLibIntegrationTest__PlanarSurface_ConstantNormal() {
    // For a plane z = 0, D1U = (1,0,0), D1V = (0,1,0)
    // Normal should be (0,0,1) at all points
    let aD1U = Vector3::new(1.0, 0.0, 0.0);
    let aD1V = Vector3::new(0.0, 1.0, 0.0);

    let aResult = cslib::normal(aD1U, aD1V, 1e-10).unwrap();

    assert_eq!(aResult.status, DerivativeStatus::Done);
    checkDirEqual(outputNormal(aResult.normal), dir(0.0, 0.0, 1.0), 1e-10);
}

// Test normal computation for a cylindrical surface
#[test]
fn CSLibIntegrationTest__CylindricalSurface_VaryingNormal() {
    // For a cylinder x = cos(u), y = sin(u), z = v
    // At u = 0: D1U = (0,1,0), D1V = (0,0,1), Normal = (1,0,0)
    // At u = pi/2: D1U = (-1,0,0), D1V = (0,0,1), Normal = (0,1,0)

    // At u = 0
    {
        let aD1U = Vector3::new(0.0, 1.0, 0.0);
        let aD1V = Vector3::new(0.0, 0.0, 1.0);

        let aResult = cslib::normal(aD1U, aD1V, 1e-10).unwrap();

        assert_eq!(aResult.status, DerivativeStatus::Done);
        checkDirEqual(outputNormal(aResult.normal), dir(1.0, 0.0, 0.0), 1e-10);
    }

    // At u = pi/2
    {
        let aD1U = Vector3::new(-1.0, 0.0, 0.0);
        let aD1V = Vector3::new(0.0, 0.0, 1.0);

        let aResult = cslib::normal(aD1U, aD1V, 1e-10).unwrap();

        assert_eq!(aResult.status, DerivativeStatus::Done);
        checkDirEqual(outputNormal(aResult.normal), dir(0.0, 1.0, 0.0), 1e-10);
    }
}

// Test Class2d with complex polygon
#[test]
fn CSLibIntegrationTest__Class2d_ComplexPolygon() {
    // L-shaped polygon
    let mut aPnts = Array1::with_bounds(1, 6).unwrap();
    aPnts[1] = Point2::new(0.0, 0.0);
    aPnts[2] = Point2::new(2.0, 0.0);
    aPnts[3] = Point2::new(2.0, 1.0);
    aPnts[4] = Point2::new(1.0, 1.0);
    aPnts[5] = Point2::new(1.0, 2.0);
    aPnts[6] = Point2::new(0.0, 2.0);

    let aClassifier = Class2d::from_array(&aPnts, 1e-10, 1e-10, 0.0, 0.0, 2.0, 2.0);

    // Point inside the L
    assert_eq!(aClassifier.si_dans(Point2::new(0.5, 0.5)) as i32, 1);

    // Point in the "notch" of the L (outside)
    assert_eq!(aClassifier.si_dans(Point2::new(1.5, 1.5)) as i32, -1);

    // Point clearly outside
    assert_eq!(aClassifier.si_dans(Point2::new(3.0, 3.0)) as i32, -1);
}
