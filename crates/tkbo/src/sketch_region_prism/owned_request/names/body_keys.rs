// SPDX-License-Identifier: GPL-2.0-only
//! Portable key expansion preserves source semantics, sweep role and complete directed ancestry.
use super::{CallerBoundaryId, CallerOperationId, SemanticNameSlot};
use crate::sketch_region_prism::frozen::{FrozenPrismKey, FrozenSweepRole, FrozenUseKey};

/// Definition identity; `key.source_member` names the source, not an output traversal rank.
/// Wire definitions deliberately have no invented V1 output member token (ADR0045).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SemanticMemberKey {
    pub operation: CallerOperationId,
    pub source: CallerBoundaryId,
    pub source_member: SemanticNameSlot,
    pub sweep_role: FrozenSweepRole,
    pub kind: u8,
}
/// One authored directed step; `step.member` retains its cap/side/container distinction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SemanticUseStep {
    pub member: SemanticMemberKey,
    pub directed_use: SemanticNameSlot,
}
/// Full ancestry is identity; `key.0` must never be reduced to its final generic LoopUse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SemanticUseKey(pub Vec<SemanticUseStep>);
/// Optional output association is metadata, not a replacement for definition identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SemanticMemberBinding {
    pub frozen: FrozenPrismKey,
    pub semantic: SemanticMemberKey,
    pub output_slot: Option<SemanticNameSlot>,
}
/// Explicit correspondence row; `binding.frozen` has no process address or returned index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SemanticUseBinding {
    pub frozen: FrozenUseKey,
    pub semantic: SemanticUseKey,
}
