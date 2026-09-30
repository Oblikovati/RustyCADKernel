// Rust translation of FoundationClasses/TKMath/BVH/BVH_Types.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `T` template parameter of BVH classes: `float` or `double`.
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

/// Numeric data type of BVH templates, sealed to OCCT's `float`/`double`
/// instantiations; `BvhBox::<Vec3<f32>>` computes in single precision.
pub trait BvhScalar:
    sealed::Sealed
    + Copy
    + Debug
    + Default
    + PartialOrd
    + Send
    + Sync
    + 'static
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    /// `std::numeric_limits<T>::max()`; `f64::MAX`.
    const MAX: Self;
    /// `std::numeric_limits<T>::lowest()`; `-f64::MAX`.
    const LOWEST: Self;
    /// `std::numeric_limits<T>::epsilon()`; `f64::EPSILON`.
    const EPSILON: Self;
    /// `std::numeric_limits<T>::infinity()`; `f64::INFINITY`.
    const INFINITY: Self;
    /// `T(0)`; `0.0`.
    const ZERO: Self;
    /// `T(1)`; `1.0`.
    const ONE: Self;
    /// `static_cast<T>(double)` with native rounding; `f32::from_double(0.1)`.
    fn from_double(value: f64) -> Self;
    /// `static_cast<T>(int)` with native rounding; `f32::from_int(16777217)`.
    fn from_int(value: i32) -> Self;
    /// Exact promotion to `double`; `1.5_f32.to_double() == 1.5`.
    fn to_double(self) -> f64;
    /// `std::abs`; `(-2.0_f64).absolute() == 2.0`.
    fn absolute(self) -> Self;
    /// `std::isinf`; `f64::INFINITY.is_infinite_value()`.
    fn is_infinite_value(self) -> bool;
    /// `static_cast<int>` on the x86-64 oracle: truncation, or `INT_MIN` for NaN
    /// and out-of-range values (`cvttsd2si`/`cvttss2si`); Rust's `as` saturates instead.
    /// Example: `f64::NAN.truncate_to_int() == i32::MIN`.
    fn truncate_to_int(self) -> i32;
}

macro_rules! bvh_scalar {
    ($scalar:ty) => {
        impl BvhScalar for $scalar {
            const MAX: Self = <$scalar>::MAX;
            const LOWEST: Self = <$scalar>::MIN;
            const EPSILON: Self = <$scalar>::EPSILON;
            const INFINITY: Self = <$scalar>::INFINITY;
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            fn from_double(value: f64) -> Self {
                value as Self
            }
            fn from_int(value: i32) -> Self {
                value as Self
            }
            fn to_double(self) -> f64 {
                f64::from(self)
            }
            fn absolute(self) -> Self {
                self.abs()
            }
            fn is_infinite_value(self) -> bool {
                self.is_infinite()
            }
            fn truncate_to_int(self) -> i32 {
                let value = f64::from(self);
                // Valid when the truncated value lies in [-2^31, 2^31 - 1].
                if value > -2_147_483_649.0 && value < 2_147_483_648.0 {
                    return value as i32;
                }
                i32::MIN
            }
        }
    };
}
bvh_scalar!(f32);
bvh_scalar!(f64);

/// `std::min(a, b)`: returns `b` only when `b < a`; `std_min(-0.0, 0.0)` is `-0.0`.
pub fn std_min<T: PartialOrd>(first: T, second: T) -> T {
    if second < first { second } else { first }
}

/// `std::max(a, b)`: returns `b` only when `a < b`; `std_max(0.0, -0.0)` is `0.0`.
pub fn std_max<T: PartialOrd>(first: T, second: T) -> T {
    if first < second { second } else { first }
}

/// `BVH::IntFloor`: floor through truncation, as upstream computes it.
///
/// The correction compares the integer converted back to `T`, so float inputs
/// use single-precision rounding; `int_floor(-0.5_f64) == -1`.
pub fn int_floor<T: BvhScalar>(value: T) -> i32 {
    let truncated = value.truncate_to_int();
    // `aRes - static_cast<int>(aRes > theValue)`; INT_MIN - 1 wraps as the native
    // subtraction does for infinite inputs.
    truncated.wrapping_sub(i32::from(T::from_int(truncated) > value))
}
