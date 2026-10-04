#[test]
fn MathMatrixTest__InitAndAccess() {
    let mut aMatrix = Matrix::with_bounds(1, 3, 1, 3).unwrap();

    // Test Init
    aMatrix.init(5.0);

    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(aMatrix[(anI, aJ)], 5.0);
        }
    }

    // Test direct value access and modification
    aMatrix[(2, 2)] = 10.0;
    assert_eq!(aMatrix[(2, 2)], 10.0);

    // Test the native mutable and const Value overloads.
    *aMatrix.change_value(3, 3).unwrap() = 15.0;
    assert_eq!(*aMatrix.value(3, 3).unwrap(), 15.0);
}

// Tests for matrix operations
#[test]
fn MathMatrixTest__MatrixOperations() {
    let mut aMatrix1 = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    let mut aMatrix2 = Matrix::with_bounds(1, 3, 1, 3).unwrap();

    // Initialize matrices
    aMatrix1.init(2.0);
    aMatrix2.init(3.0);

    // Test addition
    let mut anAddResult = aMatrix1.added(&aMatrix2).unwrap();
    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(anAddResult[(anI, aJ)], 5.0);
        }
    }

    // Test subtraction
    let mut aSubResult = aMatrix1.subtracted(&aMatrix2).unwrap();
    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(aSubResult[(anI, aJ)], -1.0);
        }
    }

    // Test multiplication by scalar
    let mut aMulResult = aMatrix1.multiplied(2.5);
    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(aMulResult[(anI, aJ)], 5.0);
        }
    }
    // Test division by scalar
    let mut aDivResult = aMatrix1.divided(0.5).unwrap();
    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(aDivResult[(anI, aJ)], 4.0);
        }
    }

    // Test in-place multiplication by scalar using Multiply method
    let mut aInPlaceMul = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aInPlaceMul.init(3.0);
    aInPlaceMul.multiply(2.0);

    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(aInPlaceMul[(anI, aJ)], 6.0);
        }
    }

    // Test in-place multiplication by scalar using operator*=
    let mut aInPlaceOp = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    aInPlaceOp.init(2.0);
    aInPlaceOp *= 3.0;

    for anI in 1..=3 {
        for aJ in 1..=3 {
            assert_eq!(aInPlaceOp[(anI, aJ)], 6.0);
        }
    }
}

// Tests for matrix multiplication
