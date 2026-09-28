#[test]
fn MathVectorTest__Resize_StackToStack_Shrink() {
    // Small vector
    let mut aVec = Vector::with_bounds(1, 20).unwrap();
    for i in 1..=20 {
        aVec[i] = (i) as f64;
    }

    // Shrink but still within stack buffer
    aVec.resize(10).unwrap();

    assert_eq!(aVec.length(), 10);
    assert_eq!(aVec.lower(), 1);
    assert_eq!(aVec.upper(), 10);

    // Data within new range should be preserved
    for i in 1..=10 {
        assert_eq!(aVec[i], (i) as f64);
    }
}

#[test]
fn MathVectorTest__Resize_StackToHeap() {
    // Small vector that fits in stack
    let mut aVec = Vector::with_bounds(1, 20).unwrap();
    for i in 1..=20 {
        aVec[i] = (i) as f64;
    }

    // Resize to larger than stack buffer (>32)
    aVec.resize(50).unwrap();

    assert_eq!(aVec.length(), 50);
    assert_eq!(aVec.lower(), 1);
    assert_eq!(aVec.upper(), 50);

    // Original data should be preserved
    for i in 1..=20 {
        assert_eq!(aVec[i], (i) as f64);
    }
}

#[test]
fn MathVectorTest__Resize_HeapToStack() {
    // Large vector on heap
    let mut aVec = Vector::with_bounds(1, 50).unwrap();
    for i in 1..=50 {
        aVec[i] = (i) as f64;
    }

    // Resize to fit in stack buffer
    aVec.resize(20).unwrap();

    assert_eq!(aVec.length(), 20);
    assert_eq!(aVec.lower(), 1);
    assert_eq!(aVec.upper(), 20);

    // Data within new range should be preserved
    for i in 1..=20 {
        assert_eq!(aVec[i], (i) as f64);
    }
}

#[test]
fn MathVectorTest__Resize_HeapToHeap() {
    // Large vector on heap
    let mut aVec = Vector::with_bounds(1, 50).unwrap();
    for i in 1..=50 {
        aVec[i] = (i) as f64;
    }

    // Resize to different heap size
    aVec.resize(100).unwrap();

    assert_eq!(aVec.length(), 100);
    assert_eq!(aVec.lower(), 1);
    assert_eq!(aVec.upper(), 100);

    // Original data should be preserved
    for i in 1..=50 {
        assert_eq!(aVec[i], (i) as f64);
    }
}

#[test]
fn MathVectorTest__Resize_NegativeLowerBound() {
    // Vector with negative lower bound
    let mut aVec = Vector::with_bounds(-5, 5).unwrap();
    for i in -5..=5 {
        aVec[i] = (i) as f64;
    }

    // Resize - lower bound preserved
    aVec.resize(8).unwrap();

    assert_eq!(aVec.length(), 8);
    assert_eq!(aVec.lower(), -5);
    assert_eq!(aVec.upper(), 2);

    // Original data should be preserved
    for i in -5..=2 {
        assert_eq!(aVec[i], (i) as f64);
    }
}

// OCC524: math_Vector::Multiply(vector, matrix) and TMultiply(vector, matrix).
// Reference values from the original Draw Harness test:
//   Vector(1..6) filled with 5.0, Matrix(1..6, 1..6) filled with 4.0
//   Vector1 = Vector * Matrix  -> each component = 6 * 5 * 4 = 120
//   After Matrix(2,1) += 1 (-> 5):
//   Vector2 = Vector * Matrix^T -> component 2 = 5*5 + 5*4*5 = 25+100 = 125, rest = 120
