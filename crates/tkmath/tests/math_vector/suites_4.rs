#[test]
fn MathVectorTest__FriendOperators() {
    let mut aVec = Vector::with_bounds(1, 3).unwrap();
    aVec[1] = 2.0;
    aVec[2] = 4.0;
    aVec[3] = 6.0;

    // Test scalar * vector
    let mut aResult1 = 3.0 * &aVec;
    assert_eq!(aResult1[1], 6.0);
    assert_eq!(aResult1[2], 12.0);
    assert_eq!(aResult1[3], 18.0);

    // Test vector * scalar
    let mut aResult2 = &aVec * 2.5;
    assert_eq!(aResult2[1], 5.0);
    assert_eq!(aResult2[2], 10.0);
    assert_eq!(aResult2[3], 15.0);

    // Test vector / scalar
    let mut aResult3 = &aVec / 2.0;
    assert_eq!(aResult3[1], 1.0);
    assert_eq!(aResult3[2], 2.0);
    assert_eq!(aResult3[3], 3.0);
}

// Tests for edge cases and boundary conditions
#[test]
fn MathVectorTest__EdgeCases() {
    // Test single element vector
    let mut aSingleVec = Vector::filled(5, 5, 42.0).unwrap();
    assert_eq!(aSingleVec.length(), 1);
    assert_eq!(aSingleVec[5], 42.0);
    assert_eq!(aSingleVec.max_index(), 5);
    assert_eq!(aSingleVec.min_index(), 5);

    // Test negative indices
    let mut aNegVec = Vector::with_bounds(-2, 1).unwrap();
    aNegVec[-2] = 10.0;
    aNegVec[-1] = 20.0;
    aNegVec[0] = 30.0;
    aNegVec[1] = 40.0;

    assert_eq!(aNegVec.length(), 4);
    assert_eq!(aNegVec.lower(), -2);
    assert_eq!(aNegVec.upper(), 1);
    assert_eq!(aNegVec.max_index(), 1);
    assert_eq!(aNegVec.min_index(), -2);
}

// Tests for Move Semantics
#[test]
fn MathVectorTest__MoveSemantics() {
    // --- Move Constructor ---

    // Large vector (heap allocated)
    let aLen: i32 = 100;
    let mut aVec1 = Vector::with_bounds(1, aLen).unwrap();
    for i in 1..=aLen {
        aVec1[i] = (i) as f64;
    }

    // Move aVec1 to aVec2
    let mut aVec2 = Vector::move_construct(&mut aVec1);

    assert_eq!(aVec2.length(), aLen);
    assert_eq!(aVec2[1], 1.0);
    assert_eq!(aVec2[aLen], (aLen) as f64);

    // Verify source state (length should be 0 after move for NCollection_Array1)
    // Note: calling Length() is safe as it just returns size.
    assert_eq!(aVec1.length(), 0);

    // Small vector (buffer allocated)
    let aSmallLen: i32 = 10;
    let mut aSmallVec1 = Vector::with_bounds(1, aSmallLen).unwrap();
    for i in 1..=aSmallLen {
        aSmallVec1[i] = (i) as f64;
    }

    // Move aSmallVec1 to aSmallVec2 (should copy because of buffer)
    let mut aSmallVec2 = Vector::move_construct(&mut aSmallVec1);

    assert_eq!(aSmallVec2.length(), aSmallLen);
    assert_eq!(aSmallVec2[1], 1.0);

    // Source remains valid for buffer-based vector
    assert_eq!(aSmallVec1.length(), aSmallLen);
    assert_eq!(aSmallVec1[1], 1.0);

    // --- Move Assignment ---

    // Large vector move assignment
    let mut aVecAssign1 = Vector::with_bounds(1, aLen).unwrap();
    for i in 1..=aLen {
        aVecAssign1[i] = (i) as f64;
    }

    let mut aVecAssign2 = Vector::with_bounds(1, aLen).unwrap();
    aVecAssign2.init(0.0);

    aVecAssign2.move_assign(&mut aVecAssign1).unwrap();

    assert_eq!(aVecAssign2.length(), aLen);
    assert_eq!(aVecAssign2[1], 1.0);

    assert_eq!(aVecAssign1.length(), 0);
}

// Tests for Resize operation
#[test]
fn MathVectorTest__Resize_StackToStack_SameSize() {
    // Small vector that fits in stack buffer (THE_BUFFER_SIZE = 32)
    let mut aVec = Vector::with_bounds(1, 10).unwrap();
    for i in 1..=10 {
        aVec[i] = (i) as f64;
    }

    // Resize to same size - data should be preserved
    aVec.resize(10).unwrap();

    assert_eq!(aVec.length(), 10);
    assert_eq!(aVec.lower(), 1);
    assert_eq!(aVec.upper(), 10);
    for i in 1..=10 {
        assert_eq!(aVec[i], (i) as f64);
    }
}

#[test]
fn MathVectorTest__Resize_StackToStack_Grow() {
    // Small vector
    let mut aVec = Vector::with_bounds(1, 5).unwrap();
    for i in 1..=5 {
        aVec[i] = (i * 10) as f64;
    }

    // Grow but still within stack buffer
    aVec.resize(20).unwrap();

    assert_eq!(aVec.length(), 20);
    assert_eq!(aVec.lower(), 1);
    assert_eq!(aVec.upper(), 20);

    // Original data should be preserved
    for i in 1..=5 {
        assert_eq!(aVec[i], (i * 10) as f64);
    }
}
