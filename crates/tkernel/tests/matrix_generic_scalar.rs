//! Native matrix scalar paths use element assignment operators, including copied results.
use rustycad_tkernel::ncollection::{Mat3, Mat4};
use std::ops::{DivAssign, MulAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
struct MatrixTagged(i32);

impl MulAssign for MatrixTagged {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = 1000 + self.0 * 10 + rhs.0;
    }
}
impl DivAssign for MatrixTagged {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = 2000 + self.0 * 10 + rhs.0;
    }
}

macro_rules! matrix_scalar_cases {
    ($multiply:ident, $divide:ident, $matrix:ident, $count:literal) => {
        #[test]
        fn $multiply() {
            let original = $matrix::from_data([MatrixTagged(2); $count]);
            let expected = [MatrixTagged(1023); $count];
            assert_eq!((original * MatrixTagged(3)).data(), expected);
            let mut value = original;
            value *= MatrixTagged(3);
            assert_eq!(value.data(), expected);
            assert_eq!(original.multiplied_scalar(MatrixTagged(3)).data(), expected);
        }
        #[test]
        fn $divide() {
            let original = $matrix::from_data([MatrixTagged(2); $count]);
            let expected = [MatrixTagged(2023); $count];
            assert_eq!((original / MatrixTagged(3)).data(), expected);
            let mut value = original;
            value /= MatrixTagged(3);
            assert_eq!(value.data(), expected);
            assert_eq!(original.divided(MatrixTagged(3)).data(), expected);
        }
    };
}
matrix_scalar_cases!(mat3_scalar_multiplication, mat3_scalar_division, Mat3, 9);
matrix_scalar_cases!(mat4_scalar_multiplication, mat4_scalar_division, Mat4, 16);

#[test]
fn integer_matrices_support_scalar_operators() {
    assert_eq!((Mat3::from_data([12_i64; 9]) / 3).data(), [4; 9]);
    assert_eq!((Mat4::from_data([12_i64; 16]) * 3).data(), [36; 16]);
}
