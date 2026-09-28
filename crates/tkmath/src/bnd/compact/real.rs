//! Scalar rounding boundary for the native float/double bounding-box templates.
use std::ops::{Add, Div, Mul, Neg, Sub};
mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}
/// Restricts storage to OCCT's float/double instantiations; `CompactBox2::<f32>`.
pub trait BoxReal:
    sealed::Sealed
    + Copy
    + std::fmt::Debug
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    /// Rounds from OCCT's double coordinates; `f32::from_double(1.0)`.
    fn from_double(value: f64) -> Self;
    /// Promotes stored values for geometry operations; `value.to_double()`.
    fn to_double(self) -> f64;
    /// Computes the scalar absolute value; `value.absolute()`.
    fn absolute(self) -> Self;
}
impl BoxReal for f32 {
    fn from_double(value: f64) -> Self {
        value as Self
    }
    fn to_double(self) -> f64 {
        f64::from(self)
    }
    fn absolute(self) -> Self {
        self.abs()
    }
}
impl BoxReal for f64 {
    fn from_double(value: f64) -> Self {
        value
    }
    fn to_double(self) -> f64 {
        self
    }
    fn absolute(self) -> Self {
        self.abs()
    }
}
