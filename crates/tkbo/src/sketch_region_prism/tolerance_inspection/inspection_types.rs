// SPDX-License-Identifier: GPL-2.0-only
//! Private owned evidence is not an ordered tolerance-validation certificate or a published body.
use super::super::region_certificate::current_binding::{CurrentPrismBinding, PrismBindingFailure};
use super::{census_types::*, policy::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::{DyadicBound, RootIsolationOptions, metric_bounds::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToleranceInspectionPhase {
    Capture,
    Metadata,
    Endpoints,
    Commit,
}
pub(crate) trait ToleranceInspectionProgress {
    fn cancelled(&mut self, phase: ToleranceInspectionPhase) -> bool;
}

pub(crate) struct PrismToleranceInspection {
    pub behavior_version: u32,
    pub requested: PrismToleranceRequest,
    pub binding: CurrentPrismBinding,
    pub census: PrismToleranceCensus,
    pub metadata: Vec<ToleranceMetadataEvidence>,
    pub endpoints: Vec<ToleranceEndpointEvidence>,
    pub trims: Vec<ToleranceTrimEvidence>,
    pub unresolved: Vec<ToleranceRelationGap>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToleranceRelationGap {
    StoredTolerancePlacementSemantics,
}
pub(crate) struct ToleranceMetadataEvidence {
    pub entity: PartitionOccurrenceIdentity,
    pub observation: StoredToleranceObservation,
}
pub(crate) struct ToleranceEndpointEvidence {
    pub edge: PartitionOccurrenceIdentity,
    pub vertex: PartitionOccurrenceIdentity,
    pub parameter: f64,
    pub contexts: Vec<ToleranceUseContext>,
    pub requested_spatial: f64,
    pub effective_spatial: f64,
    pub effective_roots: RootIsolationOptions,
    pub report: TransportedResidualReport,
}
pub(crate) struct ToleranceTrimEvidence {
    pub context: ToleranceUseContext,
    pub squared_upper: DyadicBound,
    pub oracle: ToleranceTrimOracle,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToleranceTrimOracle {
    ExactSourceAndChartIdentities,
}

pub(crate) struct ToleranceInspectionFailure {
    pub operation: &'static str,
    pub phase: ToleranceInspectionPhase,
    pub cause: ToleranceInspectionCause,
}
pub(crate) enum ToleranceInspectionCause {
    Binding(Box<PrismBindingFailure>),
    Census(Box<ToleranceCensusFailure>),
    Native {
        entity: PartitionOccurrenceIdentity,
        cause: Box<Failure>,
    },
    Scalar {
        entity: PartitionOccurrenceIdentity,
        cause: ToleranceScalarError,
    },
    Stale {
        entity: Option<PartitionOccurrenceIdentity>,
        quantity: &'static str,
    },
    Cancelled,
}
pub(super) type InspectionCheck<T> = Result<T, ToleranceInspectionFailure>;
pub(super) fn failure(
    phase: ToleranceInspectionPhase,
    cause: ToleranceInspectionCause,
) -> ToleranceInspectionFailure {
    ToleranceInspectionFailure {
        operation: "private prism tolerance inspection V1",
        phase,
        cause,
    }
}
pub(super) fn census_failure(
    phase: ToleranceInspectionPhase,
    cause: ToleranceCensusFailure,
) -> ToleranceInspectionFailure {
    failure(phase, ToleranceInspectionCause::Census(Box::new(cause)))
}
pub(super) fn binding_failure(
    phase: ToleranceInspectionPhase,
    cause: PrismBindingFailure,
) -> ToleranceInspectionFailure {
    failure(phase, ToleranceInspectionCause::Binding(Box::new(cause)))
}
pub(super) fn poll(
    progress: &mut impl ToleranceInspectionProgress,
    phase: ToleranceInspectionPhase,
) -> InspectionCheck<()> {
    if progress.cancelled(phase) {
        return Err(failure(phase, ToleranceInspectionCause::Cancelled));
    }
    Ok(())
}
