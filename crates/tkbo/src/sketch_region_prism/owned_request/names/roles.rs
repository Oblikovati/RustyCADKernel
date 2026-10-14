// SPDX-License-Identifier: GPL-2.0-only
//! Explicit versioned discriminants are lexical roles, never traversal positions or entity ordinals.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SemanticNameStage {
    Request = 1,
    Original = 2,
    InputCopy = 3,
    Partition = 4,
    CandidateCopy = 5,
    Selected = 6,
    Complement = 7,
    SweepBase = 8,
    Output = 9,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SemanticNameRole {
    Operation = 1,
    ClosureContact = 2,
    Boundary = 3,
    ClosureVertex = 4,
    StartUse = 5,
    EndUse = 6,
    BoundaryUse = 7,
    BoundedRegion = 8,
    ComplementRegion = 9,
    BoundedLoop = 10,
    ComplementLoop = 11,
    BoundedRegionUse = 12,
    ComplementRegionUse = 13,
    BoundedLoopUse = 14,
    ComplementLoopUse = 15,
    Solid = 16,
    Shell = 17,
    FirstCap = 18,
    LastCap = 19,
    Side = 20,
    FirstRim = 21,
    LastRim = 22,
    Seam = 23,
    FirstVertex = 24,
    LastVertex = 25,
    SolidUse = 26,
    ShellUse = 27,
    FaceUse = 28,
    LoopUse = 29,
    FirstRimUse = 30,
    LastRimUse = 31,
    SeamStartUse = 32,
    SeamEndUse = 33,
    Aggregate = 34,
    AggregateUse = 35,
}
/// Compact lookup token only; the full dictionary supplies its operation/source meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SemanticNameToken(pub(super) u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SemanticNameSlot {
    pub stage: SemanticNameStage,
    pub role: SemanticNameRole,
}
impl SemanticNameSlot {
    /// Injective fixed V1 encoding; example: `slot.token()` before any topology exists.
    pub(crate) fn token(self) -> SemanticNameToken {
        SemanticNameToken((u64::from(self.stage as u8) << 8) | u64::from(self.role as u8))
    }
}
impl SemanticNameToken {
    /// Temporary legacy binding value; example: `token.value()`; never a caller ID.
    pub(crate) fn value(self) -> u64 {
        self.0
    }
}
pub(super) const SOURCE_ROLES: &[SemanticNameRole] = &[
    SemanticNameRole::Boundary,
    SemanticNameRole::ClosureVertex,
    SemanticNameRole::StartUse,
    SemanticNameRole::EndUse,
    SemanticNameRole::BoundaryUse,
];
pub(super) const REGION_ROLES: &[SemanticNameRole] = &[
    SemanticNameRole::BoundedRegion,
    SemanticNameRole::ComplementRegion,
    SemanticNameRole::BoundedLoop,
    SemanticNameRole::ComplementLoop,
    SemanticNameRole::BoundedRegionUse,
    SemanticNameRole::ComplementRegionUse,
    SemanticNameRole::BoundedLoopUse,
    SemanticNameRole::ComplementLoopUse,
];
pub(super) const OUTPUT_ROLES: &[SemanticNameRole] = &[
    SemanticNameRole::Solid,
    SemanticNameRole::Shell,
    SemanticNameRole::FirstCap,
    SemanticNameRole::LastCap,
    SemanticNameRole::Side,
    SemanticNameRole::FirstRim,
    SemanticNameRole::LastRim,
    SemanticNameRole::Seam,
    SemanticNameRole::FirstVertex,
    SemanticNameRole::LastVertex,
    SemanticNameRole::SolidUse,
    SemanticNameRole::ShellUse,
    SemanticNameRole::FaceUse,
    SemanticNameRole::LoopUse,
    SemanticNameRole::FirstRimUse,
    SemanticNameRole::LastRimUse,
    SemanticNameRole::SeamStartUse,
    SemanticNameRole::SeamEndUse,
];

pub(super) const SOURCE_STAGES: &[SemanticNameStage] = &[
    SemanticNameStage::Original,
    SemanticNameStage::InputCopy,
    SemanticNameStage::Partition,
    SemanticNameStage::CandidateCopy,
    SemanticNameStage::Selected,
    SemanticNameStage::Complement,
    SemanticNameStage::SweepBase,
];
pub(super) const REGION_STAGES: &[SemanticNameStage] = &[
    SemanticNameStage::Partition,
    SemanticNameStage::CandidateCopy,
    SemanticNameStage::Selected,
    SemanticNameStage::Complement,
    SemanticNameStage::SweepBase,
];

pub(super) const REQUEST_ROLES: &[SemanticNameRole] = &[
    SemanticNameRole::Operation,
    SemanticNameRole::ClosureContact,
];
