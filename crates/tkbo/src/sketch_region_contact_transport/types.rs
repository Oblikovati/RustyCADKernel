// SPDX-License-Identifier: GPL-2.0-only
//! A contact bridge retains native construction observations without declaring a healthy region.
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_certification::BoundaryCensus;
use crate::sketch_region_partition_bindings::{PartitionBoundOutputs, RequiredPartitionProof};
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::ResidualError};

#[derive(Clone, Copy)]
pub(crate) struct ContactTransportPolicy {
    /// Separate native-parameter units for the retained report's canonical t and u axes.
    pub native_parameter_limits: [f64; 2],
    pub world_endpoint_limit: f64,
    pub roots: RootIsolationOptions,
}

impl ContactTransportPolicy {
    pub(super) fn remaining_roots(&self, used: u64) -> RootIsolationOptions {
        let mut roots = self.roots;
        // WHY: failed/cancelled completed work remains charged when the next certificate starts.
        roots.max_bisections = roots
            .max_bisections
            .saturating_sub(used.min(u16::MAX.into()) as u16);
        roots
    }
}

pub(crate) struct PairedContactTransaction {
    pub census: BoundaryCensus,
    pub mapped: PartitionBoundOutputs,
    pub requested: ContactTransportPolicy,
    pub inspection: ContactTransportInspection,
    pub current_partition: Option<
        Result<
            crate::sketch_region_partition_bindings::PartitionTransactionInspection,
            rustycad_tkernel::standard::Failure,
        >,
    >,
    pub exact_sources: Vec<super::exact_sources::ExactContactSource>,
}
pub(crate) struct ContactTransportInspection {
    pub completion: ContactTransportCompletion,
    pub contacts: Vec<ObservedPairedContact>,
    pub missing: Vec<ContactTransportGap>,
    pub remaining_operation_proofs: Vec<RequiredPartitionProof>,
    pub completed_bisections: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ReportContactIdentity {
    pub report: usize,
    pub candidate_root: usize,
}
pub(crate) struct ObservedPairedContact {
    pub identity: ReportContactIdentity,
    pub bounded_construction: Option<super::admission::BoundedContactConstruction>,
    pub occurrence_domains: Vec<usize>,
    pub observed_vertices: Vec<ObservedContactVertex>,
    pub world_measurements: Vec<super::world::ContactWorldMeasurement>,
    pub missing: Vec<ContactTransportGap>,
}
pub(crate) struct ObservedContactVertex {
    pub vertex_index: i32,
    pub occurrence: PartitionOccurrenceIdentity,
    pub native_parameters: [f64; 2],
    pub original_parents: Vec<crate::sketch_region::OriginalMemberIdentity>,
    pub native_point: [f64; 3],
    pub generated_observations: Vec<super::generated::GeneratedContactObservation>,
    pub parameter_measurement: Option<NativeParameterMeasurement>,
}
pub(crate) type NativeParameterMeasurement = Result<
    Option<[rustycad_tkmath::predicates::roots::bezier_pair::MeasuredPairParameter; 2]>,
    rustycad_tkmath::predicates::roots::bezier_pair::PairParameterConstructionFailure,
>;

pub(crate) enum ContactTransportGap {
    MissingSourceBoundDomain {
        domain: usize,
    },
    ContactSetCardinality {
        report: usize,
        expected: usize,
        observed: usize,
    },
    RepeatedRootCorrespondence {
        identity: ReportContactIdentity,
        multiplicity: usize,
    },
    ReportContactCoverageUnproved {
        report: usize,
    },
    ConstructionConformanceUnproved {
        identity: ReportContactIdentity,
    },
    AdditionalIncidentSources {
        identity: ReportContactIdentity,
        vertex_index: i32,
        sources: Vec<i32>,
    },
    MissingRetainedContactRoot {
        identity: ReportContactIdentity,
        contact_count: usize,
    },
    MissingPartitionCapture,
    IncompletePartitionCapture,
    MissingCurrentSourceProof {
        source_index: i32,
    },
    MissingExactSourceCorrespondence {
        occurrence: u64,
    },
    MissingActualPairIncidence {
        identity: ReportContactIdentity,
    },
    MissingCanonicalTopologyCorrespondence {
        identity: ReportContactIdentity,
    },
    AmbiguousPairIncidence {
        identity: ReportContactIdentity,
        count: usize,
    },
    MissingGeneratedVertexHistory {
        vertex_index: i32,
    },
    MissingCandidateOccurrence {
        vertex_index: i32,
    },
    CrossReportEqualityUnproved {
        vertex_index: i32,
    },
    UnsupportedParameterMap {
        occurrence: u64,
    },
    ConstructionBound {
        axis: Option<usize>,
        cause: Box<ResidualError>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContactTransportPhase {
    SourceBinding,
    CanonicalContacts,
    TopologyBinding,
    ParameterConstruction,
    WorldConstruction,
    GeneratedBinding,
}
pub(crate) enum ContactTransportStop {
    Policy {
        field: &'static str,
        value: f64,
        completed: Box<PairedContactTransaction>,
    },
    Cancelled {
        phase: ContactTransportPhase,
        completed: Box<PairedContactTransaction>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContactTransportCompletion {
    Completed,
    CompletedEmpty,
    Incomplete,
    Failed,
}
