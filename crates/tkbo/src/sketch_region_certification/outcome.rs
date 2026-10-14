// SPDX-License-Identifier: GPL-2.0-only
//! Every admitted numerical domain remains visible even when no certificate exists.
use crate::sketch_region::{ArrangementCause, ArrangementRelation};
use crate::sketch_region_candidates::{CandidateCause, CandidateSegment};
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};

pub(crate) struct BoundaryInputs {
    pub construction_support: rustycad_tkbrep::topods::Face,
    pub segments: Vec<CandidateSegment>,
    pub relations: Vec<ArrangementRelation>,
    pub originals: Vec<crate::sketch_region_source_conformance::OriginalCurveCapture>,
    pub constructions: Vec<crate::sketch_region_source_conformance::CapturedBoundaryConstruction>,
}
pub(crate) struct BoundaryCensus {
    pub inputs: BoundaryInputs,
    pub source_policy: Option<crate::sketch_region_source_conformance::SourceConformancePolicy>,
    pub source_binding: Option<crate::sketch_region_source_conformance::BoundSourceConformance>,
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
    SourceCancelled {
        completed: Box<BoundaryCensus>,
    },
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
    pub missing: Vec<EndpointGap>,
}
pub(crate) struct EndpointEvidence {
    pub vertex: rustycad_tkbrep::topods::Shape,
    pub parameter: f64,
    pub line_parameter: f64,
    pub native_membership: Result<
        crate::sketch_region_source_conformance::NativeParameterMembership,
        crate::sketch_region_source_conformance::SourceConformanceCause,
    >,
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
    RepresentationUncertified,
    IncidenceSourceUnbound,
    NativeCurveParameterUncertified,
    DualAxisParameterUncertified,
    CanonicalContactIdentityUncertified,
    World3dErrorUncertified,
    PartitionUncertified,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CensusPolicyViolation {
    WorldError { received: f64 },
    RootVersion { received: u32 },
    ParameterWidth { received: f64 },
}

impl BoundaryCensus {
    /// Recheck current inputs and the separately declared policy, not a stale gap vector.
    /// Example: `census.source_representation_certified()` never certifies topology.
    pub(crate) fn source_representation_certified(&self) -> bool {
        super::source_status::representation_matches(self)
    }
    /// Return authoritative current obligations after possible private stage input edits.
    /// Example: `census.current_obligations().contains(&CertificationObligation::SourceConversionAndPlane)`.
    pub(crate) fn current_obligations(&self) -> Vec<CertificationObligation> {
        let mut current = self.missing.clone();
        if !self.source_representation_certified()
            && !current.contains(&CertificationObligation::SourceConversionAndPlane)
        {
            current.push(CertificationObligation::SourceConversionAndPlane);
        }
        current
    }
}
