#[test]
fn MathVectorTest__Normalization() {
    let mut aVec = Vector::with_bounds(1, 3).unwrap();
    aVec[1] = 3.0;
    aVec[2] = 4.0;
    aVec[3] = 0.0;

    let mut anOriginalNorm = aVec.norm().unwrap();
    assert_near(anOriginalNorm, 5.0, 1e-7);

    // Test Normalized (creates new vector)
    let mut aNormalizedVec = aVec.normalized().unwrap();
    assert_near(aNormalizedVec.norm().unwrap(), 1.0, 1e-7);
    assert_near(aNormalizedVec[1], 3.0 / 5.0, 1e-7);
    assert_near(aNormalizedVec[2], 4.0 / 5.0, 1e-7);
    assert_near(aNormalizedVec[3], 0.0, 1e-7);

    // Original vector should remain unchanged
    assert_near(aVec.norm().unwrap(), 5.0, 1e-7);

    // Test Normalize (modifies in-place)
    aVec.normalize().unwrap();
    assert_near(aVec.norm().unwrap(), 1.0, 1e-7);
    check_vectors_equal(&aVec, &aNormalizedVec);
}

// Tests for normalization exception
#[test]
fn MathVectorTest__ZeroVectorHandling() {
    let mut aZeroVec = Vector::filled(1, 3, 0.0).unwrap();

    // Test behavior with zero vector - verify it's actually zero
    assert_eq!(aZeroVec.norm().unwrap(), 0.0);

    // Test with non-zero vector for comparison
    let mut aNonZeroVec = Vector::with_bounds(1, 3).unwrap();
    aNonZeroVec[1] = 1.0;
    aNonZeroVec[2] = 0.0;
    aNonZeroVec[3] = 0.0;

    assert_eq!(aNonZeroVec.norm().unwrap(), 1.0);
    aNonZeroVec.normalize().unwrap();
    assert_eq!(aNonZeroVec.norm().unwrap(), 1.0);
}

// Tests for inversion
#[test]
fn MathVectorTest__Inversion() {
    let mut aVec = Vector::with_bounds(1, 5).unwrap();
    aVec[1] = 1.0;
    aVec[2] = 2.0;
    aVec[3] = 3.0;
    aVec[4] = 4.0;
    aVec[5] = 5.0;

    // Test Inverse (creates new vector)
    let mut anInverseVec = aVec.inverse().unwrap();
    assert_eq!(anInverseVec[1], 5.0);
    assert_eq!(anInverseVec[2], 4.0);
    assert_eq!(anInverseVec[3], 3.0);
    assert_eq!(anInverseVec[4], 2.0);
    assert_eq!(anInverseVec[5], 1.0);

    // Original vector should remain unchanged
    assert_eq!(aVec[1], 1.0);
    assert_eq!(aVec[5], 5.0);

    // Test Invert (modifies in-place)
    aVec.invert().unwrap();
    check_vectors_equal(&aVec, &anInverseVec);
}

// Tests for scalar operations
#[test]
fn MathVectorTest__ScalarOperations() {
    let mut aVec = Vector::with_bounds(1, 3).unwrap();
    aVec[1] = 2.0;
    aVec[2] = 4.0;
    aVec[3] = 6.0;

    // Test multiplication by scalar
    let mut aMulResult = aVec.multiplied(2.5);
    assert_eq!(aMulResult[1], 5.0);
    assert_eq!(aMulResult[2], 10.0);
    assert_eq!(aMulResult[3], 15.0);

    // Test TMultiplied (should be same as Multiplied for scalar)
    let mut aTMulResult = aVec.multiplied(2.5);
    check_vectors_equal(&aMulResult, &aTMulResult);

    // Test in-place multiplication
    aVec.multiply(0.5);
    assert_eq!(aVec[1], 1.0);
    assert_eq!(aVec[2], 2.0);
    assert_eq!(aVec[3], 3.0);

    // Test operator*= for scalar
    aVec *= 3.0;
    assert_eq!(aVec[1], 3.0);
    assert_eq!(aVec[2], 6.0);
    assert_eq!(aVec[3], 9.0);

    // Test division by scalar
    let mut aDivResult = aVec.divided(3.0).unwrap();
    assert_eq!(aDivResult[1], 1.0);
    assert_eq!(aDivResult[2], 2.0);
    assert_eq!(aDivResult[3], 3.0);

    // Test in-place division
    aVec.divide(3.0).unwrap();
    check_vectors_equal(&aVec, &aDivResult);

    // Test operator/= for scalar
    aVec *= 6.0; // Set to [6, 12, 18]
    aVec /= 2.0;
    assert_eq!(aVec[1], 3.0);
    assert_eq!(aVec[2], 6.0);
    assert_eq!(aVec[3], 9.0);
}

// Tests for division by zero
#[test]
fn MathVectorTest__DivisionOperations() {
    let mut aVec = Vector::filled(1, 3, 2.0).unwrap();

    // Test normal division operations
    aVec.divide(2.0).unwrap();
    assert_eq!(aVec[1], 1.0);

    let mut aVec2 = Vector::filled(1, 3, 4.0).unwrap();
    let mut aResult = aVec2.divided(2.0).unwrap();
    assert_eq!(aResult[1], 2.0);
}

// Tests for vector addition and subtraction
