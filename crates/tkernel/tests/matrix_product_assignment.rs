//! Native matrix products accumulate using element `+=`, independently of binary addition.
use rustycad_tkernel::ncollection::{Mat3, Mat4};
use std::ops::{AddAssign, Mul};

#[derive(Clone, Copy, Debug, PartialEq)]
struct ProductElement(i32);

impl From<u8> for ProductElement {
    fn from(value: u8) -> Self {
        Self(i32::from(value))
    }
}

impl Mul for ProductElement {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

impl AddAssign for ProductElement {
    fn add_assign(&mut self, other: Self) {
        self.0 = self.0 * 10 + other.0;
    }
}

macro_rules! product_case {
    ($name:ident, $matrix:ident, $count:literal, $expected:expr) => {
        #[test]
        fn $name() {
            let left = $matrix::from_data([ProductElement(2); $count]);
            let right = $matrix::from_data([ProductElement(3); $count]);
            let expected = [ProductElement($expected); $count];
            assert_eq!($matrix::product(&left, &right).data(), expected);
            assert_eq!(left.multiplied(&right).data(), expected);
            assert_eq!((left * right).data(), expected);
            let mut result = left;
            result.multiply(&right);
            assert_eq!(result.data(), expected);
            result = left;
            result *= right;
            assert_eq!(result.data(), expected);
        }
    };
}
product_case!(mat3_element_assignment_product, Mat3, 9, 666);
product_case!(mat4_element_assignment_product, Mat4, 16, 6666);

#[test]
fn integer_matrix_products_are_available() {
    let left = Mat3::from_data([2_i64; 9]);
    assert_eq!((left * Mat3::from_data([3_i64; 9])).data(), [18; 9]);
    let left = Mat4::from_data([2_i64; 16]);
    assert_eq!((left * Mat4::from_data([3_i64; 16])).data(), [24; 16]);
}
