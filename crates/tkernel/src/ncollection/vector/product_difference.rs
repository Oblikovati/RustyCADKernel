//! Shared scalar dispatch prevents bool specialization from making Vec3::cross ambiguous.
use super::{ElementCast, VectorArithmeticComponent};
use std::ops::{Mul, Sub};

/// Computes the scalar expression used by Cross; existing arithmetic policies opt in automatically.
/// Example: `i32::vector_product_difference(2, 3, 1, 4) == 2`.
pub trait VectorProductDifference: Copy {
    /// Evaluates `a*b-c*d` with the native element dispatch; `Self::vector_product_difference(a,b,c,d)`.
    fn vector_product_difference(a: Self, b: Self, c: Self, d: Self) -> Self;
}
impl<T: VectorArithmeticComponent + Mul<Output = T> + Sub<Output = T>> VectorProductDifference
    for T
{
    fn vector_product_difference(a: Self, b: Self, c: Self, d: Self) -> Self {
        a.vector_multiply(b).vector_subtract(c.vector_multiply(d))
    }
}
impl VectorProductDifference for bool {
    fn vector_product_difference(a: Self, b: Self, c: Self, d: Self) -> Self {
        (a & b) ^ (c & d)
    }
}
/// Supplies native `Element_t(1)-parameter` for interpolation, including boolean promotion.
/// Example: `true.vector_interpolation_complement() == false`.
pub trait VectorInterpolationComponent: Copy {
    /// Subtracts the parameter from native one; `0.25_f64.vector_interpolation_complement() == 0.75`.
    fn vector_interpolation_complement(self) -> Self;
}
impl<T: VectorArithmeticComponent + Sub<Output = T>> VectorInterpolationComponent for T
where
    i32: ElementCast<T>,
{
    fn vector_interpolation_complement(self) -> Self {
        let one: T = 1_i32.cast();
        one.vector_subtract(self)
    }
}
impl VectorInterpolationComponent for bool {
    fn vector_interpolation_complement(self) -> Self {
        !self
    }
}
