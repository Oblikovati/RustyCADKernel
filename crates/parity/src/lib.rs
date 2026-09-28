//! Shared assertions for future OCCT differential tests.
//!
//! A tolerance must come from an upstream assertion or a reviewed contract.
//! This crate deliberately supplies no universal geometry tolerance.

/// Absolute and relative error bounds for a finite scalar comparison.
#[derive(Clone, Copy, Debug)]
pub struct Tolerance {
    absolute: f64,
    relative: f64,
}

impl Tolerance {
    /// Constructs finite, nonnegative error bounds, or returns `None`.
    #[must_use]
    pub fn new(absolute: f64, relative: f64) -> Option<Self> {
        (absolute.is_finite() && relative.is_finite() && absolute >= 0.0 && relative >= 0.0)
            .then_some(Self { absolute, relative })
    }

    /// Compares scalars using `abs(a-b) <= max(absolute, relative*max(abs(a),abs(b)))`.
    ///
    /// Equal infinities and signed zeros compare equal. NaNs never compare equal.
    /// Tests requiring signed-zero or NaN payload parity must compare `to_bits()`.
    #[must_use]
    pub fn matches(self, expected: f64, actual: f64) -> bool {
        if expected == actual {
            return true;
        }
        if !expected.is_finite() || !actual.is_finite() {
            return false;
        }
        let difference = (expected - actual).abs();
        if difference <= self.absolute {
            return true;
        }
        let scale = expected.abs().max(actual.abs());
        // Scale before subtracting if subtraction overflowed at opposite extremes.
        let relative_error = if difference.is_finite() {
            difference / scale
        } else {
            (expected / scale - actual / scale).abs()
        };
        relative_error <= self.relative
    }
}
