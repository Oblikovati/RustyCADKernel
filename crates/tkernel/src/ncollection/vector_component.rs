//! Scalar policies keep native promotion and dispatch in one generic vector API.
use std::ops::{Add, Div, Mul};

/// Supplies native absolute-value semantics; `(-3_i32).vector_component_abs()`.
pub trait VectorAbsoluteComponent: Copy {
    /// Takes an absolute value without changing the element type; `(-3.0_f32).vector_component_abs()`.
    fn vector_component_abs(self) -> Self;
}
/// Supplies the element-returning squared sum; `i8::vector_component_square([100, 0]) == 16`.
pub trait VectorSquareComponent: Copy {
    /// Applies native promotions before the final cast; `i16::vector_component_square([200, 200]) == 14464`.
    fn vector_component_square<const N: usize>(components: [Self; N]) -> Self;
}
/// Supplies square-root evaluation and the division used by Normalize.
/// `f64::vector_component_modulus([3.0, 4.0]) == 5.0`.
pub trait VectorMagnitudeComponent: Copy + From<i8> + PartialEq + Div<Output = Self> {
    /// Squares and sums in component order using native promotions; `i16::vector_component_modulus([200, 200]) == 282`.
    fn vector_component_modulus<const N: usize>(components: [Self; N]) -> Self;
}
/// Promotes the supported floating DumpJson elements to double; `0.1_f32.promote_vector_dump()`.
/// Integers deliberately have no implementation: the upstream variadic dump reads them as doubles.
pub trait VectorDumpComponent: Copy {
    /// Converts the numeric argument before formatting; `1.0_f32.promote_vector_dump() == 1.0_f64`.
    fn promote_vector_dump(self) -> f64;
}

fn floating_component_square<T, const N: usize>(components: [T; N]) -> T
where
    T: Copy + From<i8> + Add<Output = T> + Mul<Output = T>,
{
    let mut squares = components.into_iter().map(|value| value * value);
    let first: T = squares.next().unwrap_or(T::from(0));
    squares.fold(first, |sum, square| sum + square)
}

macro_rules! floating_component_policies {
    ($scalar:ty) => {
        impl VectorAbsoluteComponent for $scalar {
            fn vector_component_abs(self) -> Self {
                self.abs()
            }
        }
        impl VectorSquareComponent for $scalar {
            fn vector_component_square<const N: usize>(components: [Self; N]) -> Self {
                floating_component_square(components)
            }
        }
        impl VectorMagnitudeComponent for $scalar {
            fn vector_component_modulus<const N: usize>(components: [Self; N]) -> Self {
                Self::vector_component_square(components).sqrt()
            }
        }
        impl VectorDumpComponent for $scalar {
            fn promote_vector_dump(self) -> f64 {
                f64::from(self)
            }
        }
    };
}
floating_component_policies!(f32);
floating_component_policies!(f64);

macro_rules! integral_component_square {
    ($scalar:ty, $promoted:ty, $module:ident) => {
        mod $module {
            pub(super) fn promoted_square<const N: usize>(components: [$scalar; N]) -> $promoted {
                let mut squares = components.into_iter().map(|value| {
                    let promoted: $promoted = value as $promoted;
                    promoted.wrapping_mul(promoted)
                });
                let first: $promoted = squares.next().unwrap_or(0);
                squares.fold(first, |sum, square| sum.wrapping_add(square))
            }
        }
        impl VectorSquareComponent for $scalar {
            fn vector_component_square<const N: usize>(components: [Self; N]) -> Self {
                $module::promoted_square(components) as Self
            }
        }
    };
}
// std::sqrt(int) computes in double and the int return value truncates it; `wrapping_abs`
// defines Rust behavior for i32::MIN, where native std::abs has undefined behavior.
macro_rules! signed_component_policies {
    ($scalar:ty, $promoted:ty, $module:ident) => {
        integral_component_square!($scalar, $promoted, $module);
        impl VectorAbsoluteComponent for $scalar {
            fn vector_component_abs(self) -> Self {
                // Retain narrow signed casts and wrapping signed minima where native abs is undefined.
                self.wrapping_abs()
            }
        }
        impl VectorMagnitudeComponent for $scalar {
            fn vector_component_modulus<const N: usize>(components: [Self; N]) -> Self {
                ($module::promoted_square(components) as f64).sqrt() as Self
            }
        }
    };
}
signed_component_policies!(i8, i32, signed_eight);
signed_component_policies!(i16, i32, signed_sixteen);
signed_component_policies!(i32, i32, signed_thirty_two);
signed_component_policies!(i64, i64, signed_sixty_four);
signed_component_policies!(isize, isize, signed_pointer);
integral_component_square!(i128, i128, signed_one_twenty_eight);
integral_component_square!(u8, i32, unsigned_eight);
integral_component_square!(u16, i32, unsigned_sixteen);
integral_component_square!(u32, u32, unsigned_thirty_two);
integral_component_square!(u64, u64, unsigned_sixty_four);
integral_component_square!(u128, u128, unsigned_one_twenty_eight);
integral_component_square!(usize, usize, unsigned_pointer);
