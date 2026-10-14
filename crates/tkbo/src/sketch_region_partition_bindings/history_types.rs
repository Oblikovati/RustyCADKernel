// SPDX-License-Identifier: GPL-2.0-only
//! Directed occurrence diagnostics never equate contacts by their parent sets.
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};

pub(crate) struct CandidateHistoryInspection {
    pub mapped_candidates_available: bool,
    pub occurrences: Vec<CandidateHistoryOccurrence>,
    pub copy_dispositions: Vec<CopyRowDisposition>,
    pub faults: Vec<CandidateHistoryFault>,
}
pub(crate) struct CandidateHistoryOccurrence {
    pub candidate: usize,
    pub key: PartitionOccurrenceIdentity,
    pub copy_rows: Vec<usize>,
    pub relation_rows: Vec<usize>,
    pub parents: KindedBindingParents,
}
#[derive(Default)]
pub(crate) struct KindedBindingParents {
    pub preserved: Vec<OriginalMemberIdentity>,
    pub modified: Vec<OriginalMemberIdentity>,
    pub generated: Vec<OriginalMemberIdentity>,
}
pub(crate) struct CandidateHistoryFault {
    pub candidate: Option<usize>,
    pub key: PartitionOccurrenceIdentity,
    pub cause: CandidateHistoryCause,
}
pub(crate) enum CandidateHistoryCause {
    Access(std::sync::Arc<rustycad_tkernel::standard::Failure>),
    MissingCopiedLeaf,
    CopiedTreeMultiplicity { row: usize, count: usize },
    CopiedKind { row: usize },
    RelationMultiplicity { count: usize },
    ExtraRelation { row: usize },
    KindedParents { kind: &'static str },
    DuplicateParents { kind: &'static str },
    MissingDisposition { copy_row: usize },
    ConflictingDisposition { copy_row: usize },
    DuplicateDirectedCopy { first: usize, repeated: usize },
    MissingUnrepresentedCopy { row: usize },
}
pub(crate) struct CopyRowDisposition {
    pub copy_row: usize,
    pub represented: Vec<(usize, usize)>,
    pub unrepresented: Vec<usize>,
    pub skipped_unsupported_kind: bool,
}
