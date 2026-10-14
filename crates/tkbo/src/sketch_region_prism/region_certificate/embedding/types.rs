// SPDX-License-Identifier: GPL-2.0-only
//! Owned theorem operands; no mutable body, validation status or publication authority.
use super::*;
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PrismEmbeddingOptions {
    pub version: u32,
    pub regularity: BezierRegularityOptions,
}
impl PrismEmbeddingOptions {
    /// Select theorem behavior 1; `PrismEmbeddingOptions::v1(regularity)` keeps explicit budgets.
    pub(crate) fn v1(regularity: BezierRegularityOptions) -> Self {
        Self {
            version: 1,
            regularity,
        }
    }
}
pub(crate) struct PrismEmbeddingEvidence {
    pub(super) requested: PrismEmbeddingOptions,
    pub(super) binding: CurrentPrismBinding,
    pub(super) regularity: BezierRegularityReport,
    pub(super) domains: [PrismFaceDomain; 3],
    pub(super) incidences: [PrismFaceIncidence; 6],
}
// A cap's planar region is the captured Jordan component, never a support-plane rectangle.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PrismDomainKind {
    Cap {
        region: PartitionOccurrenceIdentity,
        winding: i64,
        sweep: f64,
    },
    Side {
        boundary: PartitionOccurrenceIdentity,
        native: [f64; 2],
        stored_sweep: [f64; 2],
        normalized_sweep: [f64; 2],
        height: f64,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrismFaceDomain {
    pub face: PartitionOccurrenceIdentity,
    pub source: PartitionOccurrenceIdentity,
    pub role: PrismImageRole,
    pub parents: Vec<OriginalMemberIdentity>,
    pub kind: PrismDomainKind,
    pub trims: Vec<PrismAuthoredTrimUse>,
}
// Complete coefficient/expectation operands remain in binding; these keys link every use to them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrismAuthoredTrimUse {
    pub face: PartitionOccurrenceIdentity,
    pub output: PartitionOccurrenceIdentity,
    pub source: PartitionOccurrenceIdentity,
    pub parents: Vec<OriginalMemberIdentity>,
    pub role: PrismTrimRole,
    pub interval: [f64; 2],
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PrismIncidenceConclusion {
    InjectivePlanarDomain,
    InjectiveModuloSeam {
        start: PrismAuthoredTrimUse,
        end: PrismAuthoredTrimUse,
        native_endpoints: [f64; 2],
        normalized_sweep: [f64; 2],
    },
    Disjoint,
    ExactlyRim {
        cap: PrismAuthoredTrimUse,
        side: PrismAuthoredTrimUse,
        native: [f64; 2],
        normalized_sweep: f64,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrismFaceIncidence {
    pub first: PrismFaceDomain,
    pub second: PrismFaceDomain,
    pub conclusion: PrismIncidenceConclusion,
}
