//=================================================================================================
// CSLib Normal Calculation Tests
//=================================================================================================

// Test Normal calculation from D1U and D1V (first overload)
#[test]
fn CSLibNormalTest__Normal_FromD1U_D1V_OrthogonalVectors() {
    // D1U along X, D1V along Y -> Normal should be along Z
    let aD1U = Vector3::new(1.0, 0.0, 0.0);
    let aD1V = Vector3::new(0.0, 1.0, 0.0);

    let aResult = cslib::normal(aD1U, aD1V, 1e-10).unwrap();

    assert_eq!(aResult.status, DerivativeStatus::Done);
    checkDirEqual(outputNormal(aResult.normal), dir(0.0, 0.0, 1.0), 1e-10);
}

#[test]
fn CSLibNormalTest__Normal_FromD1U_D1V_ScaledVectors() {
    // Non-unit vectors should still produce correct normal
    let aD1U = Vector3::new(3.0, 0.0, 0.0);
    let aD1V = Vector3::new(0.0, 5.0, 0.0);

    let aResult = cslib::normal(aD1U, aD1V, 1e-10).unwrap();

    assert_eq!(aResult.status, DerivativeStatus::Done);
    checkDirEqual(outputNormal(aResult.normal), dir(0.0, 0.0, 1.0), 1e-10);
}

#[test]
fn CSLibNormalTest__Normal_FromD1U_D1V_ParallelVectors() {
    // Parallel vectors should return D1uIsParallelD1v status
    let aD1U = Vector3::new(1.0, 0.0, 0.0);
    let aD1V = Vector3::new(2.0, 0.0, 0.0);

    let aResult = cslib::normal(aD1U, aD1V, 1e-6).unwrap();

    assert_eq!(aResult.status, DerivativeStatus::D1uIsParallelD1v);
}

#[test]
fn CSLibNormalTest__Normal_FromD1U_D1V_D1UIsNull() {
    let aD1U = Vector3::new(0.0, 0.0, 0.0);
    let aD1V = Vector3::new(0.0, 1.0, 0.0);

    let aResult = cslib::normal(aD1U, aD1V, 1e-10).unwrap();

    assert_eq!(aResult.status, DerivativeStatus::D1uIsNull);
}

#[test]
fn CSLibNormalTest__Normal_FromD1U_D1V_D1VIsNull() {
    let aD1U = Vector3::new(1.0, 0.0, 0.0);
    let aD1V = Vector3::new(0.0, 0.0, 0.0);

    let aResult = cslib::normal(aD1U, aD1V, 1e-10).unwrap();

    assert_eq!(aResult.status, DerivativeStatus::D1vIsNull);
}

#[test]
fn CSLibNormalTest__Normal_FromD1U_D1V_BothNull() {
    let aD1U = Vector3::new(0.0, 0.0, 0.0);
    let aD1V = Vector3::new(0.0, 0.0, 0.0);

    let aResult = cslib::normal(aD1U, aD1V, 1e-10).unwrap();

    assert_eq!(aResult.status, DerivativeStatus::D1IsNull);
}

// Test Normal with MagTol (third overload)
#[test]
fn CSLibNormalTest__Normal_WithMagTol_Defined() {
    let aD1U = Vector3::new(1.0, 0.0, 0.0);
    let aD1V = Vector3::new(0.0, 1.0, 0.0);

    let aResult = cslib::normal_with_magnitude(aD1U, aD1V, 1e-10).unwrap();

    assert_eq!(aResult.status, NormalStatus::Defined);
    checkDirEqual(outputNormal(aResult.normal), dir(0.0, 0.0, 1.0), 1e-10);
}

#[test]
fn CSLibNormalTest__Normal_WithMagTol_Singular() {
    // Very small vectors below tolerance
    let aD1U = Vector3::new(1e-12, 0.0, 0.0);
    let aD1V = Vector3::new(0.0, 1e-12, 0.0);

    let aResult = cslib::normal_with_magnitude(aD1U, aD1V, 1e-10).unwrap();

    assert_eq!(aResult.status, NormalStatus::Singular);
}

// Test DNNUV function
#[test]
fn CSLibNormalTest__DNNUV_ZeroDerivative() {
    // Create a simple surface derivative array
    let mut aDerSurf: Array2<'static, Vector3> = Array2::with_bounds(0, 2, 0, 2).unwrap();

    // Set D1U and D1V at index (1,0) and (0,1)
    aDerSurf.set_value(1, 0, Vector3::new(1.0, 0.0, 0.0)).unwrap(); // D1U
    aDerSurf.set_value(0, 1, Vector3::new(0.0, 1.0, 0.0)).unwrap(); // D1V
    aDerSurf.set_value(0, 0, Vector3::new(0.0, 0.0, 0.0)).unwrap(); // Position (not used)
    aDerSurf.set_value(1, 1, Vector3::new(0.0, 0.0, 0.0)).unwrap(); // D2UV
    aDerSurf.set_value(2, 0, Vector3::new(0.0, 0.0, 0.0)).unwrap(); // D2U
    aDerSurf.set_value(0, 2, Vector3::new(0.0, 0.0, 0.0)).unwrap(); // D2V

    // DNNUV(0,0) should give D1U x D1V
    let aResult = cslib::dnnuv(0, 0, &aDerSurf).unwrap();

    // D1U x D1V = (1,0,0) x (0,1,0) = (0,0,1)
    checkVecEqual(aResult, Vector3::new(0.0, 0.0, 1.0), 1e-10);
}
