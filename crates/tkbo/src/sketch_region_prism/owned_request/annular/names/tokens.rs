// SPDX-License-Identifier: GPL-2.0-only
//! Injective annular V1 tokens from lexical roles: category, scope, authored boundary role, kind
//! and variant. No token is derived from traversal rank, coordinates or process addresses.
use super::*;

/// Fixed namespace so annular tokens never coincide with single-boundary slot tokens (< 2^16).
const NAMESPACE: u64 = 0x4d41 << 48;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::sketch_region_prism) enum AnnularNameCategory {
    BoundaryMember = 3,
    OriginalUse = 4,
    OutputUse = 5,
    SelectedMember = 6,
    HistoryUse = 7,
    HistoryMember = 8,
}
/// Authored boundary role; `None` is a region-level (face/compound) member.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::sketch_region_prism) enum AnnularBoundaryRole {
    None = 0,
    Outer = 1,
    Inner = 2,
}
/// History scope code; rejected candidates are named by their authored region role.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::sketch_region_prism) enum AnnularScopeCode {
    None = 0,
    InputCopy = 1,
    Partition = 2,
    CandidateCopy = 3,
    RejectedInnerDomain = 4,
    RejectedOuterComplement = 5,
}
/// Explicit kind codes; a later kind needs a new code, never a reused one.
pub(in crate::sketch_region_prism) fn kind_code(kind: ShapeEnum) -> Option<u8> {
    Some(match kind {
        ShapeEnum::Compound => 1,
        ShapeEnum::Face => 2,
        ShapeEnum::Wire => 3,
        ShapeEnum::Edge => 4,
        ShapeEnum::Vertex => 5,
        ShapeEnum::Solid => 6,
        ShapeEnum::Shell => 7,
        _ => return None,
    })
}
/// One lexical slot; `slot.token()` is injective over all five fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::sketch_region_prism) struct AnnularNameSlot {
    pub category: AnnularNameCategory,
    pub scope: AnnularScopeCode,
    pub boundary: AnnularBoundaryRole,
    pub kind: u8,
    pub variant: u8,
}
impl AnnularNameSlot {
    /// Category-only slot; `AnnularNameSlot::of(AnnularNameCategory::Operation)`.
    pub(in crate::sketch_region_prism) fn of(category: AnnularNameCategory) -> Self {
        Self {
            category,
            scope: AnnularScopeCode::None,
            boundary: AnnularBoundaryRole::None,
            kind: 0,
            variant: 0,
        }
    }
    /// Injective fixed V1 encoding; `slot.token().0 != 0`.
    pub(in crate::sketch_region_prism) fn token(self) -> AuthoredPrismId {
        let fields = [
            self.category as u8,
            self.scope as u8,
            self.boundary as u8,
            self.kind,
            self.variant,
        ];
        let packed = fields
            .iter()
            .fold(0u64, |acc, field| (acc << 8) | u64::from(*field));
        AuthoredPrismId(NAMESPACE | packed)
    }
}
