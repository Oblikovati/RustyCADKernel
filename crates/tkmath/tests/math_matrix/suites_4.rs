#[test]
fn MathMatrixTest__SetDiag() {
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix.init(0.0);

    // Set diagonal elements to 5.0
    aMatrix.set_diag(5.0).unwrap();

    assert_eq!(aMatrix[(1, 1)], 5.0);
    assert_eq!(aMatrix[(2, 2)], 5.0);
    assert_eq!(aMatrix[(3, 3)], 5.0);

    // Check non-diagonal elements remain 0
    assert_eq!(aMatrix[(1, 2)], 0.0);
    assert_eq!(aMatrix[(1, 3)], 0.0);
    assert_eq!(aMatrix[(2, 1)], 0.0);
    assert_eq!(aMatrix[(2, 3)], 0.0);
    assert_eq!(aMatrix[(3, 1)], 0.0);
    assert_eq!(aMatrix[(3, 2)], 0.0);
}

// Tests for vector-matrix operations
#[test]
fn MathMatrixTest__VectorMatrixOperations() {
    // Create test matrix
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aMatrix[(1, 1)] = 1.0;
    aMatrix[(1, 2)] = 2.0;
    aMatrix[(1, 3)] = 3.0;
    aMatrix[(2, 1)] = 4.0;
    aMatrix[(2, 2)] = 5.0;
    aMatrix[(2, 3)] = 6.0;
    aMatrix[(3, 1)] = 7.0;
    aMatrix[(3, 2)] = 8.0;
    aMatrix[(3, 3)] = 9.0;

    // Create test vector
    let mut aVector = Vector::with_bounds(1, 3).unwrap();
    aVector[1] = 2.0;
    aVector[2] = 3.0;
    aVector[3] = 4.0;

    // Test matrix * vector
    let mut aResult = &aMatrix * &aVector;

    assert_eq!(aResult.length(), 3);
    // For row 1: 1*2 + 2*3 + 3*4 = 2 + 6 + 12 = 20
    assert_eq!(aResult[1], (20) as f64);
    // For row 2: 4*2 + 5*3 + 6*4 = 8 + 15 + 24 = 47
    assert_eq!(aResult[2], (47) as f64);
    // For row 3: 7*2 + 8*3 + 9*4 = 14 + 24 + 36 = 74
    assert_eq!(aResult[3], (74) as f64);

    // Test Multiply method with two vectors
    let mut aVec1 = Vector::with_bounds(1, 2).unwrap();
    let mut aVec2 = Vector::with_bounds(1, 3).unwrap();
    aVec1[1] = 1.0;
    aVec1[2] = 2.0;
    aVec2[1] = 3.0;
    aVec2[2] = 4.0;
    aVec2[3] = 5.0;

    let mut aMat = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    aMat.set_outer_product(&aVec1, &aVec2).unwrap();

    // The result should be the outer product of v1 and v2
    assert_eq!(aMat[(1, 1)], (1 * 3) as f64); // aVec1[1] * aVec2[1]
    assert_eq!(aMat[(1, 2)], (1 * 4) as f64); // aVec1[1] * aVec2[2]
    assert_eq!(aMat[(1, 3)], (1 * 5) as f64); // aVec1[1] * aVec2[3]
    assert_eq!(aMat[(2, 1)], (2 * 3) as f64); // aVec1[2] * aVec2[1]
    assert_eq!(aMat[(2, 2)], (2 * 4) as f64); // aVec1[2] * aVec2[2]
    assert_eq!(aMat[(2, 3)], (2 * 5) as f64); // aVec1[2] * aVec2[3]
}

// Special test to ensure that the matrix multiplication bug is fixed
// Bug description: In-place matrix multiplication modifies matrix elements
// that are still needed for the computation of remaining elements
