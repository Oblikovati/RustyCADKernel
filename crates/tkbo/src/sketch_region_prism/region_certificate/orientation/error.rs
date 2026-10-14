// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismOrientationPhase {
    Admission,
    Embedding,
    Winding,
    Faces,
    FinalBinding,
}
// Failures preserve legacy causes and are not an immutable published body interface.
pub(crate) struct PrismOrientationFailure {
    pub requested: PrismOrientationOptions,
    pub phase: PrismOrientationPhase,
    pub original: Option<Box<crate::sketch_region::OriginalMemberIdentity>>,
    pub cause: PrismOrientationCause,
}
pub(crate) enum PrismOrientationCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Embedding(Box<PrismEmbeddingFailure>),
    Binding(Box<PrismBindingFailure>),
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
pub(super) fn failure(
    requested: PrismOrientationOptions,
    phase: PrismOrientationPhase,
    embedding: Option<&PrismEmbeddingEvidence>,
    cause: PrismOrientationCause,
) -> PrismOrientationFailure {
    PrismOrientationFailure {
        requested,
        phase,
        cause,
        original: embedding.map(|e| Box::new(e.binding().region.original.clone())),
    }
}
