#[test]
fn MathVectorTest__VectorAdditionSubtraction() {
    let mut aVec1 = Vector::with_bounds(1, 3).unwrap();
    aVec1[1] = 1.0;
    aVec1[2] = 2.0;
    aVec1[3] = 3.0;

    let mut aVec2 = Vector::with_bounds(1, 3).unwrap();
    aVec2[1] = 4.0;
    aVec2[2] = 5.0;
    aVec2[3] = 6.0;

    // Test Added
    let mut anAddResult = aVec1.added(&aVec2).unwrap();
    assert_eq!(anAddResult[1], 5.0);
    assert_eq!(anAddResult[2], 7.0);
    assert_eq!(anAddResult[3], 9.0);

    // Test operator+
    let mut anAddResult2 = &aVec1 + &aVec2;
    check_vectors_equal(&anAddResult, &anAddResult2);

    // Test Subtracted
    let mut aSubResult = aVec1.subtracted(&aVec2).unwrap();
    assert_eq!(aSubResult[1], -3.0);
    assert_eq!(aSubResult[2], -3.0);
    assert_eq!(aSubResult[3], -3.0);

    // Test operator-
    let mut aSubResult2 = &aVec1 - &aVec2;
    check_vectors_equal(&aSubResult, &aSubResult2);

    // Test in-place Add
    let mut aVecCopy1 = aVec1.clone();
    aVecCopy1.add(&aVec2).unwrap();
    check_vectors_equal(&aVecCopy1, &anAddResult);

    // Test operator+=
    let mut aVecCopy2 = aVec1.clone();
    aVecCopy2 += &aVec2;
    check_vectors_equal(&aVecCopy2, &anAddResult);

    // Test in-place Subtract
    let mut aVecCopy3 = aVec1.clone();
    aVecCopy3.subtract(&aVec2).unwrap();
    check_vectors_equal(&aVecCopy3, &aSubResult);

    // Test operator-=
    let mut aVecCopy4 = aVec1.clone();
    aVecCopy4 -= &aVec2;
    check_vectors_equal(&aVecCopy4, &aSubResult);
}

// Tests for vector operations with different bounds
#[test]
fn MathVectorTest__VectorOperationsDifferentBounds() {
    let mut aVec1 = Vector::with_bounds(0, 2).unwrap();
    aVec1[0] = 1.0;
    aVec1[1] = 2.0;
    aVec1[2] = 3.0;

    let mut aVec2 = Vector::with_bounds(-1, 1).unwrap();
    aVec2[-1] = 4.0;
    aVec2[0] = 5.0;
    aVec2[1] = 6.0;

    // Should work fine - same length, different bounds
    let mut anAddResult = aVec1.added(&aVec2).unwrap();
    assert_eq!(anAddResult[0], 5.0); // 1.0 + 4.0
    assert_eq!(anAddResult[1], 7.0); // 2.0 + 5.0
    assert_eq!(anAddResult[2], 9.0); // 3.0 + 6.0
}

// Tests for dimension errors

// Tests for dot product
#[test]
fn MathVectorTest__DotProduct() {
    let mut aVec1 = Vector::with_bounds(1, 3).unwrap();
    aVec1[1] = 1.0;
    aVec1[2] = 2.0;
    aVec1[3] = 3.0;

    let mut aVec2 = Vector::with_bounds(1, 3).unwrap();
    aVec2[1] = 4.0;
    aVec2[2] = 5.0;
    aVec2[3] = 6.0;

    // Test dot product (1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32)
    let mut aDotProduct = aVec1.dot(&aVec2).unwrap();
    assert_eq!(aDotProduct, 32.0);

    // Test operator*
    let mut aDotProduct2 = &aVec1 * &aVec2;
    assert_eq!(aDotProduct, aDotProduct2);
}

// Tests for Set operation
#[test]
fn MathVectorTest__SetOperation() {
    let mut aVec = Vector::with_bounds(1, 6).unwrap();
    aVec.init(0.0);

    let mut aSubVec = Vector::with_bounds(1, 3).unwrap();
    aSubVec[1] = 10.0;
    aSubVec[2] = 20.0;
    aSubVec[3] = 30.0;

    // Set elements from index 2 to 4
    aVec.set(2, 4, &aSubVec).unwrap();

    assert_eq!(aVec[1], 0.0);
    assert_eq!(aVec[2], 10.0);
    assert_eq!(aVec[3], 20.0);
    assert_eq!(aVec[4], 30.0);
    assert_eq!(aVec[5], 0.0);
    assert_eq!(aVec[6], 0.0);
}

// Tests for Slice operation
#[test]
fn MathVectorTest__SliceOperation() {
    let mut aVec = Vector::with_bounds(1, 6).unwrap();
    aVec[1] = 10.0;
    aVec[2] = 20.0;
    aVec[3] = 30.0;
    aVec[4] = 40.0;
    aVec[5] = 50.0;
    aVec[6] = 60.0;

    // Test normal slice (ascending)
    let mut aSlice1 = aVec.slice(2, 4).unwrap();
    assert_eq!(aSlice1.length(), 3);
    assert_eq!(aSlice1.lower(), 2);
    assert_eq!(aSlice1.upper(), 4);
    assert_eq!(aSlice1[2], 20.0);
    assert_eq!(aSlice1[3], 30.0);
    assert_eq!(aSlice1[4], 40.0);

    // Test reverse slice (descending indices)
    let mut aSlice2 = aVec.slice(4, 2).unwrap();
    assert_eq!(aSlice2.length(), 3);
    assert_eq!(aSlice2.lower(), 2);
    assert_eq!(aSlice2.upper(), 4);
    assert_eq!(aSlice2[2], 20.0); // Copy from original aVec[2]
    assert_eq!(aSlice2[3], 30.0); // Copy from original aVec[3]
    assert_eq!(aSlice2[4], 40.0); // Copy from original aVec[4]
}

// Tests for vector-matrix operations
