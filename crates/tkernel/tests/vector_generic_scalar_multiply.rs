//! Scalar/vector overloads must coexist for compatible user-defined element types.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use std::{
    fmt::Debug,
    ops::{Mul, MulAssign},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntegerElement(i64);

impl From<i32> for IntegerElement {
    fn from(value: i32) -> Self {
        Self(i64::from(value))
    }
}
impl Mul for IntegerElement {
    type Output = Self;
    fn mul(self, right: Self) -> Self {
        Self(self.0 * right.0)
    }
}

// Native custom elements may give assignment and value multiplication different behavior.
impl MulAssign for IntegerElement {
    fn mul_assign(&mut self, right: Self) {
        self.0 = self.0 * right.0 + 1;
    }
}

fn verify_scalar_multiply<
    T: rustycad_tkernel::ncollection::VectorArithmeticComponent
        + From<i32>
        + Mul<Output = T>
        + MulAssign
        + Eq
        + Debug,
>(
    increment: i32,
) {
    let mut planar = Vec2::new(T::from(-3), T::from(7)) * T::from(-2);
    assert_eq!(planar.data(), [T::from(6), T::from(-14)]);
    planar *= T::from(3);
    assert_eq!(
        planar.data(),
        [T::from(18 + increment), T::from(-42 + increment)]
    );
    verify_spatial_scalar::<T>(increment);
    verify_homogeneous_scalar::<T>(increment);
}

fn verify_spatial_scalar<
    T: rustycad_tkernel::ncollection::VectorArithmeticComponent
        + From<i32>
        + Mul<Output = T>
        + MulAssign
        + Eq
        + Debug,
>(
    increment: i32,
) {
    let mut vector = Vec3::new(T::from(1), T::from(-2), T::from(5)) * T::from(4);
    assert_eq!(
        vector.data(),
        [4 + increment, -8 + increment, 20 + increment].map(T::from)
    );
    vector *= T::from(-3);
    let expected = [-12 - 2 * increment, 24 - 2 * increment, -60 - 2 * increment];
    assert_eq!(vector.data(), expected.map(T::from));
}

fn verify_homogeneous_scalar<
    T: rustycad_tkernel::ncollection::VectorArithmeticComponent
        + From<i32>
        + Mul<Output = T>
        + MulAssign
        + Eq
        + Debug,
>(
    increment: i32,
) {
    let mut vector = Vec4::new(T::from(-2), T::from(3), T::from(0), T::from(9)) * T::from(-2);
    let expected = [4 + increment, -6 + increment, increment, -18 + increment];
    assert_eq!(vector.data(), expected.map(T::from));
    vector *= T::from(2);
    let expected = [
        8 + 3 * increment,
        -12 + 3 * increment,
        3 * increment,
        -36 + 3 * increment,
    ];
    assert_eq!(vector.data(), expected.map(T::from));
}

#[test]
fn signed_64_bit_elements_keep_native_scalar_operators() {
    verify_scalar_multiply::<i64>(0);
}

#[test]
fn custom_copy_elements_keep_native_scalar_operators() {
    verify_scalar_multiply::<IntegerElement>(1);
}

#[test]
fn scalar_and_componentwise_multiplication_remain_distinct() {
    let first = Vec3::new(2_i64, 3, 5);
    assert_eq!((first * 4).data(), [8, 12, 20]);
    assert_eq!((first * Vec3::new(7, 11, 13)).data(), [14, 33, 65]);
}

impl rustycad_tkernel::ncollection::VectorArithmeticComponent for IntegerElement {}
