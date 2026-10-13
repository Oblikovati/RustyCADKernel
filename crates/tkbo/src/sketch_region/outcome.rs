// SPDX-License-Identifier: GPL-2.0-only
//! Stage-specific ownership and provenance never imply validated geometry or certified incidences.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;

/// Caller identity for an authored input, not an ordinal-based persistent name; `RegionSourceId(42)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegionSourceId(pub u64);
/// Read-only borrowed source occurrence; underlying caller state may be mutable; `ArrangementSource { identity: RegionSourceId(42), shape: &edge }`.
pub struct ArrangementSource<'a> {
    pub identity: RegionSourceId,
    pub shape: &'a Shape,
}
/// Cancellation and failure checkpoints; `ArrangementPhase::Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrangementPhase {
    Admission,
    Copy,
    Interference,
    History,
}
/// Explicit non-success cause, with original typed failure retained; `ArrangementCause::Empty`.
pub enum ArrangementCause {
    Empty,
    MappedReport {
        phase: ArrangementPhase,
        report: std::sync::Arc<rustycad_tkernel::message::Report>,
        affected_source: Option<RegionSourceId>,
    },
    UnsupportedHistoryCoverage {
        member: OriginalMemberIdentity,
    },
    DuplicateIdentity(RegionSourceId),
    NullSource(RegionSourceId),
    Failure {
        phase: ArrangementPhase,
        cause: Failure,
    },
    MissingHistory {
        source: Option<RegionSourceId>,
    },
    UnattributedOutput,
}
/// Operation-local original occurrence identity, not a persistent name or recovery heuristic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OriginalMemberIdentity {
    pub source: RegionSourceId,
    pub topology_address: usize,
    pub location: rustycad_tkmath::toploc::Location,
    pub orientation: rustycad_tkg3d::top_abs::Orientation,
}
/// Complete original-member relations for one actual result entity; `relation.preserved`.
pub struct ArrangementRelation {
    pub output: Shape,
    pub preserved: Vec<OriginalMemberIdentity>,
    pub modified: Vec<OriginalMemberIdentity>,
    pub generated: Vec<OriginalMemberIdentity>,
}
/// Owned stage output, with validation and exact-incidence acceptance still outstanding.
pub struct OwnedArrangement {
    pub arrangement: Shape,
    pub relations: Vec<ArrangementRelation>,
    pub deleted: Vec<OriginalMemberIdentity>,
    pub achieved_error: Option<f64>,
    pub incidences_certified: bool,
}
/// Successful stage completion differs from final region acceptance; `ArrangementCompletion::Cancelled(ArrangementPhase::Copy)`.
pub enum ArrangementCompletion {
    Arranged(OwnedArrangement),
    CompletedEmpty {
        deleted: Vec<OriginalMemberIdentity>,
    },
    Declined(ArrangementCause),
    Failed(ArrangementCause),
    Cancelled(ArrangementPhase),
    Incomplete(ArrangementCause),
}
