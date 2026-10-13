// SPDX-License-Identifier: GPL-2.0-only
use super::super::{NonFiniteCoordinate, exact_coordinate};
use num_bigint::BigInt;
use std::{cmp::Ordering, fmt};

/// An exact number `significand * 2^exponent`; integer storage stays private.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DyadicBound {
    pub(super) significand: BigInt,
    exponent: i32,
}
impl DyadicBound {
    /// Preserve every finite binary64 bit, including subnormals. Example: `DyadicBound::from_f64(0.5)?`.
    pub fn from_f64(value: f64) -> Result<Self, NonFiniteCoordinate> {
        Ok(Self::new(exact_coordinate(value, 0, 0)?, -1074))
    }
    /// Compare a root bound to an exact binary64 value. Example: `bound.compare_f64(0.)?`.
    pub fn compare_f64(&self, value: f64) -> Result<Ordering, NonFiniteCoordinate> {
        Ok(self.cmp(&Self::from_f64(value)?))
    }
    pub(super) fn new(significand: BigInt, exponent: i32) -> Self {
        // Binary64 bounds plus at most u16::MAX subdivisions fit i32 exponents
        // and 32-bit shift counts, including a difference across opposite signs.
        let Some(zeros) = significand.trailing_zeros() else {
            return Self {
                significand,
                exponent: 0,
            };
        };
        Self {
            significand: significand >> zeros,
            exponent: exponent + zeros as i32,
        }
    }
    pub(super) fn midpoint(&self, other: &Self) -> Self {
        let (left, right, exponent) = self.aligned(other);
        Self::new(left + right, exponent - 1)
    }
    pub(super) fn difference(&self, other: &Self) -> Self {
        let (left, right, exponent) = self.aligned(other);
        Self::new(left - right, exponent)
    }
    fn aligned(&self, other: &Self) -> (BigInt, BigInt, i32) {
        let exponent = self.exponent.min(other.exponent);
        let left = &self.significand << (self.exponent - exponent) as usize;
        let right = &other.significand << (other.exponent - exponent) as usize;
        (left, right, exponent)
    }
    pub(super) fn ratio(&self) -> (BigInt, BigInt) {
        if self.exponent >= 0 {
            return (&self.significand << self.exponent as usize, BigInt::from(1));
        }
        (
            self.significand.clone(),
            BigInt::from(1) << (-self.exponent) as usize,
        )
    }
}
impl Ord for DyadicBound {
    fn cmp(&self, other: &Self) -> Ordering {
        let (left, right, _) = self.aligned(other);
        left.cmp(&right)
    }
}
impl PartialOrd for DyadicBound {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl fmt::Display for DyadicBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} * 2^{}", self.significand, self.exponent)
    }
}
