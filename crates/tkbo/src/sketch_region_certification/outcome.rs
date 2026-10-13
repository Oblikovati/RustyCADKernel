// SPDX-License-Identifier: GPL-2.0-only
//! Every admitted numerical domain remains visible even when no certificate exists.
use crate::sketch_region::{ArrangementCause, ArrangementRelation};
use crate::sketch_region_candidates::{CandidateCause, CandidateSegment};
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};

pub(crate) struct BoundaryInputs {
    pub segments: Vec<CandidateSegment>,
    pub relations: Vec<ArrangementRelation>,
}
pub(crate) struct BoundaryCensus {
    pub inputs: BoundaryInputs,
    pub domains: Vec<BoundaryDomain>,
    pub reports: Vec<BezierIncidenceReport>,
    pub failures: Vec<BezierIncidenceFailure>,
    pub requested: CensusPolicy,
    pub bisections: u64,
    pub missing: Vec<CertificationObligation>,
}
#[derive(Clone, Copy)]
pub(crate) struct CensusPolicy {
    pub roots: RootIsolationOptions,
    pub max_world_error: f64,
}
pub(crate) struct BoundaryDomain {
    pub occurrences: [usize; 2],
    pub evidence: DomainEvidence,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DomainEvidence {
    StoredIncidence {
        report: usize,
        curve: usize,
        line: usize,
    },
    NumericalFailure(usize),
    Unsupported(CensusGap),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CensusGap {
    SelfOffDiagonal,
    BezierPair,
    SamePhysicalCurve,
    EmptyBoundaryUniverse,
}
pub(crate) enum CensusCause {
    Input(ArrangementCause),
    Policy(CensusPolicyViolation),
    Capture(CandidateCause),
    Incomplete(CensusGap),
    Cancelled {
        next_domain: [usize; 2],
        during_domain: bool,
        completed: Box<BoundaryCensus>,
    },
}
/// Provisional matching evidence deliberately cannot promote the full region operation.
pub(crate) struct ContactCoverage {
    pub report: usize,
    pub contact: usize,
    pub vertices: Vec<EndpointEvidence>,
    pub complete: bool,
    pub missing: EndpointGap,
}
pub(crate) struct EndpointEvidence {
    pub vertex: rustycad_tkbrep::topods::Shape,
    pub parameter: f64,
    pub line_parameter: f64,
    pub parameter_in_bracket: bool,
    pub exact_parameter: bool,
    pub world_inside_enclosure: bool,
    pub generated_from_both: bool,
    pub certificate_world_error: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CertificationObligation {
    SourceConversionAndPlane,
    ExhaustivePairAndSelfDomains,
    FailedWorkAccounting,
    MappedSplitCorrespondence,
    CandidateSearchAndCoverage,
    SelfIntersection,
    Tolerance,
}
pub(crate) struct CandidateOccurrenceCoverage {
    pub input: usize,
    pub represented_occurrences: Vec<u64>,
    pub parameter_coverage_certified: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EndpointGap {
    SourcePlaneAndParameterConstructionUncertified,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CensusPolicyViolation {
    WorldError { received: f64 },
    RootVersion { received: u32 },
    ParameterWidth { received: f64 },
}
