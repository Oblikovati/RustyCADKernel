// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedPrismOrientationPhase {
    Admission,
    Directions,
    Names,
    Commit,
}
pub(crate) struct JoinedPrismOrientationFailure {
    pub operation: AuthoredPrismId,
    pub requested: JoinedPrismOrientationOptions,
    pub phase: JoinedPrismOrientationPhase,
    pub cause: JoinedPrismOrientationCause,
}
pub(crate) enum JoinedPrismOrientationCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Cancelled,
    Orientation(Box<PrismOrientationFailure>),
    Count {
        subject: &'static str,
        actual: usize,
        expected: usize,
    },
    Named {
        occurrence: Box<FrozenUseKey>,
        cause: Box<JoinedPrismOrientationCause>,
    },
    Direction {
        actual: Orientation,
        expected: Orientation,
    },
    Interval {
        actual: [u64; 2],
        expected: [u64; 2],
    },
    ParentMultiplicity {
        actual: usize,
        expected: usize,
    },
}
pub(super) fn failure(
    join: &JoinedPrismProof,
    requested: JoinedPrismOrientationOptions,
    phase: JoinedPrismOrientationPhase,
    cause: JoinedPrismOrientationCause,
) -> JoinedPrismOrientationFailure {
    JoinedPrismOrientationFailure {
        operation: join.projection().operation,
        requested,
        phase,
        cause,
    }
}
pub(super) fn named<T>(
    occurrence: &FrozenUseKey,
    result: JoinedDirectionCheck<T>,
) -> JoinedDirectionCheck<T> {
    result.map_err(|cause| JoinedPrismOrientationCause::Named {
        occurrence: Box::new(occurrence.clone()),
        cause: Box::new(cause),
    })
}
pub(super) fn count(
    subject: &'static str,
    actual: usize,
    expected: usize,
) -> JoinedDirectionCheck<()> {
    if actual == expected {
        return Ok(());
    }
    Err(JoinedPrismOrientationCause::Count {
        subject,
        actual,
        expected,
    })
}
