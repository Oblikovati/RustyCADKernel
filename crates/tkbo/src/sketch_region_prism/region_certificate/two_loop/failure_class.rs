// SPDX-License-Identifier: GPL-2.0-only
//! Preserve native failure/cancellation through ownership wrappers; unsupported evidence stays incomplete.
use super::failure::*;
use crate::sketch_region::{
    ArrangementCause, ArrangementCompletion, PartitionObservation, PartitionReadCause,
};
use crate::sketch_region_candidates::{
    CandidateCause, CandidateCopyInspection, CandidateCopyTreeObservation,
};
use crate::sketch_region_geometry_capture::CapturedGeometryCause;
use crate::sketch_region_source_conformance::{SourceBindingFault, SourceConformanceCause};
use rustycad_tkmath::predicates::roots::{
    RootIsolationError, bezier_line::BezierWireError, bezier_pair::BezierPairCompletion,
    metric_bounds::ResidualError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TwoLoopRefusalClass {
    Incomplete,
    Failed,
    Cancelled,
}
pub(super) fn classify(failure: &TwoLoopRegionFailure<'_>) -> TwoLoopRefusalClass {
    let direct = owned(&failure.cause);
    if direct != TwoLoopRefusalClass::Incomplete {
        return direct;
    }
    failure.local.as_ref().map_or(direct, local)
}
fn owned(cause: &TwoLoopRegionCause) -> TwoLoopRefusalClass {
    use TwoLoopRefusalClass::*;
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
fn local(cause: &TwoLoopLocalDiagnostic<'_>) -> TwoLoopRefusalClass {
    use TwoLoopRefusalClass::*;
    match cause {
        TwoLoopLocalDiagnostic::Arrangement(completion) => arrangement(completion),
        TwoLoopLocalDiagnostic::Candidate(cause) => candidate(cause),
        TwoLoopLocalDiagnostic::Source(cause) => source(cause),
        TwoLoopLocalDiagnostic::SourceBinding(faults) => source_faults(faults),
        TwoLoopLocalDiagnostic::Copy(copy) => copied(copy),
        TwoLoopLocalDiagnostic::Region(cause) | TwoLoopLocalDiagnostic::Partition { cause, .. } => {
            region(cause)
        }
        TwoLoopLocalDiagnostic::PartitionObservation(PartitionObservation::Incomplete {
            failure,
            ..
        }) if matches!(failure.cause, PartitionReadCause::Native(_)) => Failed,
        _ => Incomplete,
    }
}
fn copied(copy: &CandidateCopyInspection) -> TwoLoopRefusalClass {
    let class = first_class(copy.observation_failures.iter().map(candidate));
    if class != TwoLoopRefusalClass::Incomplete {
        return class;
    }
    match copy.copied_tree {
        CandidateCopyTreeObservation::Incomplete { .. } => TwoLoopRefusalClass::Failed,
        _ => TwoLoopRefusalClass::Incomplete,
    }
}
fn region(cause: &super::super::RegionCertificateCause) -> TwoLoopRefusalClass {
    use super::super::RegionCertificateCause;
    match cause {
        RegionCertificateCause::Mapped(_) => TwoLoopRefusalClass::Failed,
        RegionCertificateCause::Capture(cause) => surface(cause),
        _ => TwoLoopRefusalClass::Incomplete,
    }
}
fn arrangement(completion: &ArrangementCompletion) -> TwoLoopRefusalClass {
    use TwoLoopRefusalClass::*;
    match completion {
        ArrangementCompletion::Cancelled(_) => Cancelled,
        ArrangementCompletion::Failed(_) => Failed,
        ArrangementCompletion::Declined(ArrangementCause::Failure { .. })
        | ArrangementCompletion::Incomplete(ArrangementCause::Failure { .. }) => Failed,
        _ => Incomplete,
    }
}
fn candidate(cause: &CandidateCause) -> TwoLoopRefusalClass {
    use TwoLoopRefusalClass::*;
    match cause {
        CandidateCause::Mapped(_) => Failed,
        CandidateCause::Cancelled(_) => Cancelled,
        CandidateCause::Numerical(cause) => numerical(cause),
        CandidateCause::CapturedNumerical(cause) => numerical(cause.cause()),
        CandidateCause::CapturedOperation(cause) => candidate(&cause.cause),
        _ => Incomplete,
    }
}
fn source(cause: &SourceConformanceCause) -> TwoLoopRefusalClass {
    match cause {
        SourceConformanceCause::Capture(cause) => geometry(cause),
        SourceConformanceCause::Exact(cause) => residual(cause),
        _ => TwoLoopRefusalClass::Incomplete,
    }
}
fn source_faults(faults: &[SourceBindingFault]) -> TwoLoopRefusalClass {
    first_class(faults.iter().map(|fault| match fault {
        SourceBindingFault::CurrentGeometry { cause, .. } => source(cause),
        _ => TwoLoopRefusalClass::Incomplete,
    }))
}
fn first_class(classes: impl Iterator<Item = TwoLoopRefusalClass>) -> TwoLoopRefusalClass {
    let mut ordered = classes;
    ordered
        .find(|class| *class != TwoLoopRefusalClass::Incomplete)
        .unwrap_or(TwoLoopRefusalClass::Incomplete)
}
fn geometry(cause: &CapturedGeometryCause) -> TwoLoopRefusalClass {
    match cause {
        CapturedGeometryCause::Native(_) => TwoLoopRefusalClass::Failed,
        CapturedGeometryCause::Exact(cause) => residual(cause),
        _ => TwoLoopRefusalClass::Incomplete,
    }
}
fn surface(cause: &super::super::super::surface_charts::PrismSurfaceCause) -> TwoLoopRefusalClass {
    use super::super::super::surface_charts::PrismSurfaceCause;
    match cause {
        PrismSurfaceCause::SourceSnapshot(cause) => surface(cause),
        PrismSurfaceCause::Capture(cause) => geometry(cause),
        PrismSurfaceCause::Exact(cause) => residual(cause),
        _ => TwoLoopRefusalClass::Incomplete,
    }
}
fn residual(cause: &ResidualError) -> TwoLoopRefusalClass {
    match cause {
        ResidualError::Roots(cause) => numerical(cause),
        ResidualError::Signs(cause) => numerical(cause),
        _ => TwoLoopRefusalClass::Incomplete,
    }
}
fn numerical(cause: &(dyn std::error::Error + 'static)) -> TwoLoopRefusalClass {
    if matches!(
        cause.downcast_ref::<RootIsolationError>(),
        Some(RootIsolationError::Cancelled)
    ) || matches!(
        cause.downcast_ref::<BezierWireError>(),
        Some(BezierWireError::Cancelled { .. })
    ) {
        return TwoLoopRefusalClass::Cancelled;
    }
    cause
        .source()
        .map_or(TwoLoopRefusalClass::Incomplete, numerical)
}
