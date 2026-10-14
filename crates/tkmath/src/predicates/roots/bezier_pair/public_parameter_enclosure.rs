// SPDX-License-Identifier: GPL-2.0-only
//! Borrow the actual computed rational enclosure; binary64 bounds remain a separate outward view.
use super::super::{
    certificate_arithmetic::{ExactRatio, RatioInterval},
    metric_bounds::ResidualError,
};
use std::cmp::Ordering;

/// Immutable rational bounds tied to the completed contact and its original source/root proof.
pub struct ExactBezierPairParameterEnclosure<'a> {
    interval: &'a RatioInterval,
}
impl<'a> ExactBezierPairParameterEnclosure<'a> {
    pub(super) fn new(interval: &'a RatioInterval) -> Self {
        let retained = interval;
        // WHY: issue #10 must compare actual graph evidence, never reinterpret its rounded display.
        Self { interval: retained }
    }
    /// Compare the actual lower rational bound exactly; example: `contact.exact_paired_parameter.compare_lower(0.5)?`.
    pub fn compare_lower(&self, value: f64) -> Result<Ordering, ResidualError> {
        let supplied = finite_parameter(value)?;
        Ok(self.interval.lower.cmp(&supplied))
    }
    /// Compare the actual upper rational bound exactly; example: `contact.exact_paired_parameter.compare_upper(0.5)?`.
    pub fn compare_upper(&self, value: f64) -> Result<Ordering, ResidualError> {
        let supplied = finite_parameter(value)?;
        Ok(self.interval.upper.cmp(&supplied))
    }
}
fn finite_parameter(value: f64) -> Result<ExactRatio, ResidualError> {
    if !value.is_finite() {
        return Err(ResidualError::Value {
            role: "paired parameter comparison",
            index: 0,
            value,
        });
    }
    Ok(ExactRatio::from_finite(value))
}
