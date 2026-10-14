// SPDX-License-Identifier: GPL-2.0-only
//! Pure failures cannot retain live embedding/binding diagnostics, even as unreachable variants.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismOrientationEvaluationPhase {
    Winding,
    Faces,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrismOrientationEvaluationFailure {
    pub phase: PrismOrientationEvaluationPhase,
    pub cause: PrismOrientationEvaluationCause,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PrismOrientationEvaluationCause {
    Unsupported {
        entity: Box<PartitionOccurrenceIdentity>,
        actual: Orientation,
    },
    Winding {
        actual: i64,
        expected: i64,
    },
    ZeroJacobian,
    Face {
        domain: Box<PrismFaceDomain>,
        actual: Orientation,
        expected: Orientation,
    },
    Trim {
        authored: Box<PrismAuthoredTrimUse>,
        actual: Orientation,
        expected: Orientation,
    },
}
pub(super) fn legacy_failure(
    requested: PrismOrientationOptions,
    embedding: &PrismEmbeddingEvidence,
    error: PrismOrientationEvaluationFailure,
) -> PrismOrientationFailure {
    failure(
        requested,
        error.phase.into(),
        Some(embedding),
        error.cause.into(),
    )
}
impl From<PrismOrientationEvaluationPhase> for PrismOrientationPhase {
    fn from(phase: PrismOrientationEvaluationPhase) -> Self {
        match phase {
            PrismOrientationEvaluationPhase::Winding => Self::Winding,
            PrismOrientationEvaluationPhase::Faces => Self::Faces,
        }
    }
}
impl From<PrismOrientationEvaluationCause> for PrismOrientationCause {
    fn from(cause: PrismOrientationEvaluationCause) -> Self {
        use PrismOrientationEvaluationCause as Pure;
        match cause {
            Pure::Unsupported { entity, actual } => Self::Unsupported { entity, actual },
            Pure::Winding { actual, expected } => Self::Winding { actual, expected },
            Pure::ZeroJacobian => Self::ZeroJacobian,
            Pure::Face {
                domain,
                actual,
                expected,
            } => legacy_face(domain, actual, expected),
            Pure::Trim {
                authored,
                actual,
                expected,
            } => legacy_trim(authored, actual, expected),
        }
    }
}

fn legacy_face(
    domain: Box<PrismFaceDomain>,
    actual: Orientation,
    expected: Orientation,
) -> PrismOrientationCause {
    PrismOrientationCause::Face {
        domain,
        actual,
        expected,
    }
}
fn legacy_trim(
    authored: Box<PrismAuthoredTrimUse>,
    actual: Orientation,
    expected: Orientation,
) -> PrismOrientationCause {
    PrismOrientationCause::Trim {
        authored,
        actual,
        expected,
    }
}
