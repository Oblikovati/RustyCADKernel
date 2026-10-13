// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{RootIsolationError, signs::RootSignError};
use super::super::BezierLineError;
use std::fmt;
/// A directed contact query without a complete isolated-contact result.
#[derive(Clone, Debug, PartialEq)]
pub enum BezierRayError {
    /// Input, denominator, or normal-numerator failure with its original owner.
    Line(BezierLineError),
    /// The normal polynomial is zero; finite isolated contacts cannot describe overlap.
    SupportingLineCoincidence,
    /// Longitudinal conversion or exact product failed.
    Direction(RootIsolationError),
    /// Exact projected endpoint evaluation failed.
    Endpoint(RootIsolationError),
    /// Exact sign certification failed.
    Signs(RootSignError),
    /// A certified isolated root did not have a nonzero local normal sign.
    LocalNormalSign(super::super::super::RootBracket),
}
impl fmt::Display for BezierRayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line(cause) => write!(formatter, "ray supporting line: {cause}"),
            Self::SupportingLineCoincidence => write!(
                formatter,
                "identically zero ray normal; expected isolated contacts, not supporting-line coincidence"
            ),
            Self::Direction(cause) => write!(formatter, "ray longitudinal numerator: {cause}"),
            Self::Endpoint(cause) => write!(formatter, "curve endpoint: {cause}"),
            Self::Signs(cause) => write!(formatter, "ray side: {cause}"),
            Self::LocalNormalSign(root) => write!(
                formatter,
                "zero local normal at {root:?}; expected nonzero leading derivative sign or nonroot upper bound"
            ),
        }
    }
}
impl std::error::Error for BezierRayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(cause) => Some(cause),
            Self::Direction(cause) | Self::Endpoint(cause) => Some(cause),
            Self::Signs(cause) => Some(cause),
            Self::SupportingLineCoincidence | Self::LocalNormalSign(_) => None,
        }
    }
}
