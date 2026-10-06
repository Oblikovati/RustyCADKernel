//! Native promoted unary minus also accepts unsigned and boolean components.
use super::{Vec2, Vec3, Vec4};
use std::ops::Neg;

/// Supplies unary minus without Rust's signed-only primitive bound.
/// Custom scalars opt in with `fn vector_negated_component(self) -> Self { -self }`.
pub trait VectorNegationComponent: Copy {
    /// Returns the component after native promotion and narrowing; `1_u8.vector_negated_component() == 255`.
    fn vector_negated_component(self) -> Self;
}
macro_rules! integral_negation {
    ($($scalar:ty),+) => {$(
        impl VectorNegationComponent for $scalar {
            fn vector_negated_component(self) -> Self { self.wrapping_neg() }
        }
    )+};
}
integral_negation!(
    i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize
);
macro_rules! floating_negation {
    ($($scalar:ty),+) => {$(
        impl VectorNegationComponent for $scalar {
            fn vector_negated_component(self) -> Self { -self }
        }
    )+};
}
floating_negation!(f32, f64);
impl VectorNegationComponent for bool {
    fn vector_negated_component(self) -> Self {
        self
    }
}
macro_rules! vector_negation {
    ($vector:ident, [$($field:ident),+]) => {
        impl<T: VectorNegationComponent> Neg for $vector<T> {
            type Output = Self;
            fn neg(self) -> Self { Self::new($(self.$field.vector_negated_component()),+) }
        }
    };
}
vector_negation!(Vec2, [x, y]);
vector_negation!(Vec3, [x, y, z]);
vector_negation!(Vec4, [x, y, z, w]);
