//! Compound matrix operators return the destination and retain element arithmetic.
use rustycad_tkernel::ncollection::{Mat3, Mat4};

macro_rules! matrix_operator_cases {
    ($matrix:ident, $returns:ident, $aliases:ident) => {
        #[test]
        fn $returns() {
            let mut matrix = $matrix::<i32>::identity();
            let origin = std::ptr::from_mut(&mut matrix);
            let identity = $matrix::<i32>::identity();
            assert_eq!(std::ptr::from_mut(matrix.add_returning(&identity)), origin);
            let subtracted = std::ptr::from_mut(matrix.subtract_returning(&identity));
            assert_eq!(subtracted, origin);
            let multiplied = std::ptr::from_mut(matrix.multiply_returning(&identity));
            assert_eq!(multiplied, origin);
            let scaled = std::ptr::from_mut(matrix.multiply_scalar_returning(6));
            assert_eq!(scaled, origin);
            assert_eq!(std::ptr::from_mut(matrix.divide_returning(3)), origin);
            assert_eq!(matrix, identity.multiplied_scalar(2));
        }

        #[test]
        fn $aliases() {
            let mut matrix = $matrix::<f64>::identity();
            let origin = std::ptr::from_mut(&mut matrix);
            assert_eq!(std::ptr::from_mut(matrix.add_self()), origin);
            assert_eq!(std::ptr::from_mut(matrix.multiply_self()), origin);
            assert_eq!(matrix, $matrix::<f64>::identity().multiplied_scalar(4.0));
            matrix.set_value(0, 0, f64::INFINITY);
            assert_eq!(std::ptr::from_mut(matrix.subtract_self()), origin);
            assert!(matrix.get_value(0, 0).is_nan());
            assert!(matrix.as_slice()[1..].iter().all(|value| *value == 0.0));
        }
    };
}

matrix_operator_cases!(Mat3, mat3_compounds_return_destination, mat3_self_operands);
matrix_operator_cases!(Mat4, mat4_compounds_return_destination, mat4_self_operands);
