// SPDX-License-Identifier: GPL-2.0-only
//! Scalar metric reports retain actual point coordinates and all observed directed uses.
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};
use crate::sketch_region_source_conformance::SourceConformanceCause;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::ResidualDecision};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClosurePointStage {
    Original,
    Census,
    Partition,
    CandidateFirst,
    CandidateSecond,
    PrismCopy,
}
#[derive(Debug)]
pub(crate) struct ClosurePointContext {
    pub stage: ClosurePointStage,
    pub occurrence: PartitionOccurrenceIdentity,
}
#[derive(Debug)]
pub(crate) struct ClosurePointEvidence {
    pub contexts: Vec<ClosurePointContext>,
    pub point: [f64; 3],
    pub original: OriginalMemberIdentity,
    pub requested_world_limit: f64,
    pub exact_certificate_limit: f64,
    pub roots: RootIsolationOptions,
    pub decision: Result<ResidualDecision, SourceConformanceCause>,
    pub bisections: u64,
}
