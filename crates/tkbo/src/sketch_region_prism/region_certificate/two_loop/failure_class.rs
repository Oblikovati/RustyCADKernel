// SPDX-License-Identifier: GPL-2.0-only
//! Preserve native failure/cancellation through ownership wrappers; unsupported evidence stays incomplete.
use super::failure::*;
use crate::sketch_region::{
    ArrangementCause, ArrangementCompletion, PartitionObservation, PartitionReadCause,
};
use crate::sketch_region_candidates::CandidateCause;
use crate::sketch_region_geometry_capture::CapturedGeometryCause;
use crate::sketch_region_source_conformance::{SourceBindingFault, SourceConformanceCause};
use rustycad_tkmath::predicates::roots::{
    RootIsolationError, bezier_line::BezierWireError, bezier_pair::BezierPairCompletion,
    metric_bounds::ResidualError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RefusalClass {
    Incomplete,
    Failed,
    Cancelled,
}
pub(super) fn classify(failure: &TwoLoopRegionFailure<'_>) -> RefusalClass {
    let direct = owned(&failure.cause);
    if direct != RefusalClass::Incomplete {
        return direct;
    }
    failure.local.as_ref().map_or(direct, local)
}
fn owned(cause: &TwoLoopRegionCause) -> RefusalClass {
    use RefusalClass::*;
    match cause {
        TwoLoopRegionCause::Native(_) => Failed,
        TwoLoopRegionCause::Cancelled => Cancelled,
        TwoLoopRegionCause::Geometry(cause) => geometry(cause),
        TwoLoopRegionCause::Capture(cause) => surface(cause),
        TwoLoopRegionCause::Residual(cause) => residual(cause),
        TwoLoopRegionCause::Winding { cause, .. } | TwoLoopRegionCause::Numerical(cause) => {
            numerical(cause)
        }
        TwoLoopRegionCause::PairRefusal {
            completion: BezierPairCompletion::Cancelled,
            ..
        } => Cancelled,
        _ => Incomplete,
    }
}
fn local(cause: &TwoLoopLocalDiagnostic<'_>) -> RefusalClass {
    use RefusalClass::*;
    match cause {
        TwoLoopLocalDiagnostic::Arrangement(completion) => arrangement(completion),
        TwoLoopLocalDiagnostic::Candidate(cause) => candidate(cause),
        TwoLoopLocalDiagnostic::Source(cause) => source(cause),
        TwoLoopLocalDiagnostic::SourceBinding(faults) => source_faults(faults),
        TwoLoopLocalDiagnostic::Copy(copy) => {
            first_class(copy.observation_failures.iter().map(candidate))
        }
        TwoLoopLocalDiagnostic::PartitionObservation(PartitionObservation::Incomplete {
            failure,
            ..
        }) if matches!(failure.cause, PartitionReadCause::Native(_)) => Failed,
        _ => Incomplete,
    }
}
fn arrangement(completion: &ArrangementCompletion) -> RefusalClass {
    use RefusalClass::*;
    match completion {
        ArrangementCompletion::Cancelled(_) => Cancelled,
        ArrangementCompletion::Failed(_) => Failed,
        ArrangementCompletion::Declined(ArrangementCause::Failure { .. })
        | ArrangementCompletion::Incomplete(ArrangementCause::Failure { .. }) => Failed,
        _ => Incomplete,
    }
}
fn candidate(cause: &CandidateCause) -> RefusalClass {
    use RefusalClass::*;
    match cause {
        CandidateCause::Mapped(_) => Failed,
        CandidateCause::Cancelled(_) => Cancelled,
        CandidateCause::Numerical(cause) => numerical(cause),
        CandidateCause::CapturedNumerical(cause) => numerical(cause.cause()),
        CandidateCause::CapturedOperation(cause) => candidate(&cause.cause),
        _ => Incomplete,
    }
}
fn source(cause: &SourceConformanceCause) -> RefusalClass {
    match cause {
        SourceConformanceCause::Capture(cause) => geometry(cause),
        SourceConformanceCause::Exact(cause) => residual(cause),
        _ => RefusalClass::Incomplete,
    }
}
fn source_faults(faults: &[SourceBindingFault]) -> RefusalClass {
    first_class(faults.iter().map(|fault| match fault {
        SourceBindingFault::CurrentGeometry { cause, .. } => source(cause),
        _ => RefusalClass::Incomplete,
    }))
}
fn first_class(classes: impl Iterator<Item = RefusalClass>) -> RefusalClass {
    let mut ordered = classes;
    ordered
        .find(|class| *class != RefusalClass::Incomplete)
        .unwrap_or(RefusalClass::Incomplete)
}
fn geometry(cause: &CapturedGeometryCause) -> RefusalClass {
    match cause {
        CapturedGeometryCause::Native(_) => RefusalClass::Failed,
        CapturedGeometryCause::Exact(cause) => residual(cause),
        _ => RefusalClass::Incomplete,
    }
}
fn surface(cause: &super::super::super::surface_charts::PrismSurfaceCause) -> RefusalClass {
    use super::super::super::surface_charts::PrismSurfaceCause;
    match cause {
        PrismSurfaceCause::SourceSnapshot(cause) => surface(cause),
        PrismSurfaceCause::Capture(cause) => geometry(cause),
        PrismSurfaceCause::Exact(cause) => residual(cause),
        _ => RefusalClass::Incomplete,
    }
}
fn residual(cause: &ResidualError) -> RefusalClass {
    match cause {
        ResidualError::Roots(cause) => numerical(cause),
        ResidualError::Signs(cause) => numerical(cause),
        _ => RefusalClass::Incomplete,
    }
}
fn numerical(cause: &(dyn std::error::Error + 'static)) -> RefusalClass {
    if matches!(
        cause.downcast_ref::<RootIsolationError>(),
        Some(RootIsolationError::Cancelled)
    ) || matches!(
        cause.downcast_ref::<BezierWireError>(),
        Some(BezierWireError::Cancelled { .. })
    ) {
        return RefusalClass::Cancelled;
    }
    cause.source().map_or(RefusalClass::Incomplete, numerical)
}
