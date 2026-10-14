// SPDX-License-Identifier: GPL-2.0-only
//! The metadata stage reuses completed exact geometry and retains cap decisions separately.
use super::{JoinedEvidenceClass, StoredToleranceCap};
use crate::sketch_region_prism::{
    frozen::{proof_join::JoinedPrismProof, *},
    tolerance_inspection::*,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedTolerancePhase {
    Admission,
    Metadata,
    Commit,
}
pub(crate) trait JoinedToleranceProgress {
    fn cancelled(&mut self, phase: JoinedTolerancePhase) -> bool;
}
pub(crate) struct JoinedToleranceMetadata {
    pub entity: FrozenPrismKey,
    pub contexts: Vec<FrozenUseKey>,
    pub observation: StoredToleranceObservation,
}
pub(crate) struct JoinedPrismTolerance {
    pub(super) binding: Vec<u8>,
    pub(super) operation_limits: FrozenPrismLimits,
    pub(super) cap: StoredToleranceCap,
    pub(super) metadata: Vec<JoinedToleranceMetadata>,
}
pub(crate) struct JoinedToleranceFailure {
    pub operation: &'static str,
    pub phase: JoinedTolerancePhase,
    pub entity: Option<FrozenPrismKey>,
    pub cause: JoinedToleranceCause,
    pub metadata: Vec<JoinedToleranceMetadata>,
    pub binding: Vec<u8>,
    pub operation_limits: FrozenPrismLimits,
    pub cap: StoredToleranceCap,
}
pub(crate) enum JoinedToleranceCause {
    ConformanceBinding,
    MissingMetadata {
        entity: FrozenPrismKey,
        quantity: &'static str,
    },
    MetadataScalar {
        entity: FrozenPrismKey,
        cause: ToleranceScalarError,
    },
    Census {
        actual: usize,
        expected: usize,
    },
    RefusedCap {
        entities: Vec<FrozenPrismKey>,
    },
    Cancelled,
}
impl JoinedToleranceFailure {
    /// Preserve refusal versus missing evidence; example: `failure.classification()`.
    pub(crate) fn classification(&self) -> JoinedEvidenceClass {
        match self.cause {
            JoinedToleranceCause::Cancelled => JoinedEvidenceClass::Cancelled,
            JoinedToleranceCause::RefusedCap { .. }
            | JoinedToleranceCause::MetadataScalar { .. } => JoinedEvidenceClass::Failed,
            _ => JoinedEvidenceClass::Incomplete,
        }
    }
}
impl JoinedPrismTolerance {
    /// Match every joined operand; example: `proof.binds(join)`.
    pub(crate) fn binds(&self, join: &JoinedPrismProof) -> bool {
        self.binding == join.replay_bytes()
    }
    /// Borrow all physical metadata observations; example: `proof.metadata().len()`.
    pub(crate) fn metadata(&self) -> &[JoinedToleranceMetadata] {
        &self.metadata
    }
    /// Retain the explicit cap; example: `proof.cap().maximum()`.
    pub(crate) fn cap(&self) -> StoredToleranceCap {
        self.cap
    }
    /// Keep the operation request independent; example: `proof.operation_limits().spatial_limit`.
    pub(crate) fn operation_limits(&self) -> &FrozenPrismLimits {
        &self.operation_limits
    }
    /// Retain behavior identity; example: `proof.version()`.
    pub(crate) fn version(&self) -> u32 {
        2
    }
}
