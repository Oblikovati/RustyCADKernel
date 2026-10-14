// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularDirectionPhase {
    Admission,
    Winding,
    Directions,
    Commit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularDirectionCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Count {
        subject: &'static str,
        actual: usize,
        expected: usize,
    },
    Winding {
        source: RegionSourceId,
        actual: Option<i64>,
    },
    Unsupported {
        occurrence: Box<FrozenUseKey>,
        actual: Orientation,
    },
    Direction {
        occurrence: Box<FrozenUseKey>,
        actual: Orientation,
        expected: Orientation,
    },
    Context {
        occurrence: Box<FrozenUseKey>,
        required: &'static str,
    },
    ZeroJacobian,
    Cancelled,
}
pub(in crate::sketch_region_prism) struct AnnularDirectionFailure {
    pub operation: AuthoredPrismId,
    pub requested: AnnularOrientationOptions,
    pub phase: AnnularDirectionPhase,
    pub cause: AnnularDirectionCause,
    pub source_binding: Box<[u8]>,
}
impl AnnularDirectionFailure {
    /// Read the actual refusal class; `failure.classification()` never treats missing proof as valid.
    pub(in crate::sketch_region_prism) fn classification(&self) -> TwoLoopRefusalClass {
        match self.cause {
            AnnularDirectionCause::Cancelled => TwoLoopRefusalClass::Cancelled,
            AnnularDirectionCause::Direction { .. } | AnnularDirectionCause::ZeroJacobian => {
                TwoLoopRefusalClass::Failed
            }
            _ => TwoLoopRefusalClass::Incomplete,
        }
    }
}
pub(super) fn count(
    subject: &'static str,
    actual: usize,
    expected: usize,
) -> AnnularDirectionCheck<()> {
    if actual == expected {
        return Ok(());
    }
    Err(AnnularDirectionCause::Count {
        subject,
        actual,
        expected,
    })
}
