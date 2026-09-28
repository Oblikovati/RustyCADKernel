#[test]
fn MathVectorTest__VectorMatrixOperations() {
    // Create test matrix [2x3]
    let mut aMat = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 2.0;
    aMat[(1, 3)] = 3.0;
    aMat[(2, 1)] = 4.0;
    aMat[(2, 2)] = 5.0;
    aMat[(2, 3)] = 6.0;

    // Create test vectors
    let mut aVec1 = Vector::with_bounds(1, 2).unwrap(); // For left multiplication
    aVec1[1] = 2.0;
    aVec1[2] = 3.0;

    let mut aVec2 = Vector::with_bounds(1, 3).unwrap(); // For right multiplication
    aVec2[1] = 1.0;
    aVec2[2] = 2.0;
    aVec2[3] = 3.0;

    // Test vector * matrix (should produce vector of size 3)
    let mut aResult1 = aVec1.multiplied_matrix(&aMat).unwrap();
    assert_eq!(aResult1.length(), 3);
    // aResult1 = [2, 3] * [[1,2,3],[4,5,6]] = [2*1+3*4, 2*2+3*5, 2*3+3*6] = [14, 19, 24]
    assert_eq!(aResult1[1], 14.0);
    assert_eq!(aResult1[2], 19.0);
    assert_eq!(aResult1[3], 24.0);

    // Test vector multiplication with matrix using Multiply method
    let mut aResult2 = Vector::with_bounds(1, 3).unwrap();
    aResult2.set_vector_matrix(&aVec1, &aMat).unwrap();
    check_vectors_equal(&aResult1, &aResult2);

    // Test matrix * vector using Multiply method
    let mut aResult3 = Vector::with_bounds(1, 2).unwrap();
    aResult3.set_matrix_vector(&aMat, &aVec2).unwrap();
    // aResult3 = [[1,2,3],[4,5,6]] * [1,2,3] = [1*1+2*2+3*3, 4*1+5*2+6*3] = [14, 32]
    assert_eq!(aResult3[1], 14.0);
    assert_eq!(aResult3[2], 32.0);
}

// Tests for transpose matrix operations
#[test]
fn MathVectorTest__TransposeMatrixOperations() {
    // Create test matrix [2x3]
    let mut aMat = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    aMat[(1, 1)] = 1.0;
    aMat[(1, 2)] = 2.0;
    aMat[(1, 3)] = 3.0;
    aMat[(2, 1)] = 4.0;
    aMat[(2, 2)] = 5.0;
    aMat[(2, 3)] = 6.0;

    let mut aVec1 = Vector::with_bounds(1, 2).unwrap();
    aVec1[1] = 2.0;
    aVec1[2] = 3.0;

    let mut aVec2 = Vector::with_bounds(1, 3).unwrap();
    aVec2[1] = 1.0;
    aVec2[2] = 2.0;
    aVec2[3] = 3.0;

    // Test TMultiply (matrix^T * vector)
    let mut aResult1 = Vector::with_bounds(1, 3).unwrap();
    aResult1.set_transpose_matrix_vector(&aMat, &aVec1).unwrap();
    // aMat^T * aVec1 = [[1,4],[2,5],[3,6]] * [2,3] = [1*2+4*3, 2*2+5*3, 3*2+6*3] = [14, 19, 24]
    assert_eq!(aResult1[1], 14.0);
    assert_eq!(aResult1[2], 19.0);
    assert_eq!(aResult1[3], 24.0);

    // Test TMultiply (vector * matrix^T)
    let mut aResult2 = Vector::with_bounds(1, 2).unwrap();
    aResult2.set_vector_transpose_matrix(&aVec2, &aMat).unwrap();
    // aVec2 * aMat^T = [1,2,3] * [[1,4],[2,5],[3,6]] = [1*1+2*2+3*3, 1*4+2*5+3*6] = [14, 32]
    assert_eq!(aResult2[1], 14.0);
    assert_eq!(aResult2[2], 32.0);
}

// Tests for matrix dimension errors

// Tests for three-operand operations
#[test]
fn MathVectorTest__ThreeOperandOperations() {
    let mut aVec1 = Vector::with_bounds(1, 3).unwrap();
    aVec1[1] = 1.0;
    aVec1[2] = 2.0;
    aVec1[3] = 3.0;

    let mut aVec2 = Vector::with_bounds(1, 3).unwrap();
    aVec2[1] = 4.0;
    aVec2[2] = 5.0;
    aVec2[3] = 6.0;

    let mut aResult = Vector::with_bounds(1, 3).unwrap();

    // Test Add(left, right)
    aResult.set_sum(&aVec1, &aVec2).unwrap();
    assert_eq!(aResult[1], 5.0);
    assert_eq!(aResult[2], 7.0);
    assert_eq!(aResult[3], 9.0);

    // Test Subtract(left, right)
    aResult.set_difference(&aVec1, &aVec2).unwrap();
    assert_eq!(aResult[1], -3.0);
    assert_eq!(aResult[2], -3.0);
    assert_eq!(aResult[3], -3.0);

    // Test Multiply(scalar, vector)
    aResult.set_scaled(2.5, &aVec1).unwrap();
    assert_eq!(aResult[1], 2.5);
    assert_eq!(aResult[2], 5.0);
    assert_eq!(aResult[3], 7.5);
}

// Tests for Opposite operation
#[test]
fn MathVectorTest__OppositeOperation() {
    let mut aVec = Vector::with_bounds(1, 3).unwrap();
    aVec[1] = 1.0;
    aVec[2] = -2.0;
    aVec[3] = 3.0;

    let mut anOpposite = aVec.opposite();
    assert_eq!(anOpposite[1], -1.0);
    assert_eq!(anOpposite[2], 2.0);
    assert_eq!(anOpposite[3], -3.0);

    // Test unary minus operator
    let mut anOpposite2 = -&aVec;
    check_vectors_equal(&anOpposite, &anOpposite2);
}

// Tests for assignment operations
#[test]
fn MathVectorTest__AssignmentOperations() {
    let mut aVec1 = Vector::with_bounds(1, 3).unwrap();
    aVec1[1] = 1.0;
    aVec1[2] = 2.0;
    aVec1[3] = 3.0;

    let mut aVec2 = Vector::with_bounds(1, 3).unwrap();
    aVec2.init(0.0);

    // Test Initialized
    aVec2.initialized(&aVec1).unwrap();
    check_vectors_equal(&aVec1, &aVec2);

    // Test operator=
    let mut aVec3 = Vector::with_bounds(1, 3).unwrap();
    aVec3.initialized(&aVec1).unwrap();
    check_vectors_equal(&aVec1, &aVec3);
}

// Tests for friend operators
