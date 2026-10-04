// Tests for constructors
#[test]
fn MathVectorTest__Constructors() {
    // Test standard constructor
    let mut aVec1 = Vector::with_bounds(1, 5).unwrap();
    assert_eq!(aVec1.length(), 5);
    assert_eq!(aVec1.lower(), 1);
    assert_eq!(aVec1.upper(), 5);

    // Test constructor with initial value
    let mut aVec2 = Vector::filled(-2, 3, 2.5).unwrap();
    assert_eq!(aVec2.length(), 6);
    assert_eq!(aVec2.lower(), -2);
    assert_eq!(aVec2.upper(), 3);

    for anI in -2..=3 {
        assert_eq!(aVec2[anI], 2.5);
    }

    // Test constructor with external array
    let mut anArray = [1.0, 2.0, 3.0, 4.0, 5.0];
    let expectedArray = anArray;
    let mut aVec3 = Vector::from_buffer(&mut anArray, 0, 4).unwrap();
    assert_eq!(aVec3.length(), 5);
    assert_eq!(aVec3.lower(), 0);
    assert_eq!(aVec3.upper(), 4);

    for anI in 0..=4 {
        assert_eq!(aVec3[anI], expectedArray[anI as usize]);
    }

    // Test copy constructor
    let mut aVec4 = aVec2.clone();
    check_vectors_equal(&aVec2, &aVec4);
}

// Tests for gp_XY and gp_XYZ constructors
#[test]
fn MathVectorTest__GeometryConstructors() {
    // Test gp_XY constructor
    let anXY = Xy::new(3.5, 4.5);
    let mut aVecXY = Vector::from_xy(anXY);
    assert_eq!(aVecXY.length(), 2);
    assert_eq!(aVecXY.lower(), 1);
    assert_eq!(aVecXY.upper(), 2);
    assert_eq!(aVecXY[1], 3.5);
    assert_eq!(aVecXY[2], 4.5);

    // Test gp_XYZ constructor
    let anXYZ = Xyz::new(1.1, 2.2, 3.3);
    let mut aVecXYZ = Vector::from_xyz(anXYZ);
    assert_eq!(aVecXYZ.length(), 3);
    assert_eq!(aVecXYZ.lower(), 1);
    assert_eq!(aVecXYZ.upper(), 3);
    assert_eq!(aVecXYZ[1], 1.1);
    assert_eq!(aVecXYZ[2], 2.2);
    assert_eq!(aVecXYZ[3], 3.3);
}

// Tests for initialization and access
#[test]
fn MathVectorTest__InitAndAccess() {
    let mut aVec = Vector::with_bounds(1, 4).unwrap();

    // Test Init
    aVec.init(7.0);
    for anI in 1..=4 {
        assert_eq!(aVec[anI], 7.0);
    }

    // Test direct value access and modification
    aVec[2] = 15.0;
    assert_eq!(aVec[2], 15.0);

    // Test Value method
    *aVec.change_value(3).unwrap() = 25.0;
    assert_eq!(aVec[3], 25.0);
    assert_eq!(*aVec.value(3).unwrap(), 25.0);
}

// Tests for vector properties
#[test]
fn MathVectorTest__VectorProperties() {
    let mut aVec = Vector::with_bounds(1, 4).unwrap();
    aVec[1] = 3.0;
    aVec[2] = 4.0;
    aVec[3] = 0.0;
    aVec[4] = -2.0;

    // Test Norm (should be sqrt(3^2 + 4^2 + 0^2 + (-2)^2) = sqrt(29))
    let mut anExpectedNorm = 29.0_f64.sqrt();
    assert_near(aVec.norm().unwrap(), anExpectedNorm, 1e-7);

    // Test Norm2 (should be 29)
    assert_near(aVec.norm2().unwrap(), 29.0, 1e-7);

    // Test Max (should return index 2, value 4.0)
    assert_eq!(aVec.max_index(), 2);

    // Test Min (should return index 4, value -2.0)
    assert_eq!(aVec.min_index(), 4);
}

// Tests for normalization
