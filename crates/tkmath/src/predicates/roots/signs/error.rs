// SPDX-License-Identifier: GPL-2.0-only
use super::super::RootIsolationError;
use std::fmt;
/// Input or calculation owning a failed sign request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootSignStage {
    /// Invalid version, interval, or width.
    Policy,
    /// Source coefficients or source root isolation.
    Source,
    /// Query coefficients.
    Query,
    /// Signed remainder construction or sign classification.
    Evaluation,
}
impl RootSignStage {
    pub(super) fn failure(self, cause: RootIsolationError) -> RootSignError {
        RootSignError::Root { stage: self, cause }
    }
}
/// Failure to produce a complete certified sign list.
#[derive(Clone, Debug, PartialEq)]
pub enum RootSignError {
    /// Typed root failure and its owner.
    Root {
        /// Failing stage.
        stage: RootSignStage,
        /// Original failure, including the offending value.
        cause: RootIsolationError,
    },
    /// A one-root bracket must have Tarski sum -1, 0, or 1.
    Variation {
        /// Variation at its lower endpoint.
        lower: usize,
        /// Variation at its upper endpoint.
        upper: usize,
    },
}
impl fmt::Display for RootSignError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root { stage, cause } => write!(formatter, "root sign {stage:?}: {cause}"),
            Self::Variation { lower, upper } => write!(
                formatter,
                "root sign variations {lower}, {upper}; expected difference -1, 0, or 1 in a one-root bracket"
            ),
        }
    }
}
impl std::error::Error for RootSignError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Root { cause, .. } => Some(cause),
            Self::Variation { .. } => None,
        }
    }
}
