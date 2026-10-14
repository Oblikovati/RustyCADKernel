// SPDX-License-Identifier: GPL-2.0-only
//! Only an exact discrepancy witness is a failed geometric check; missing proof stays incomplete.
use super::{closure_scalar::ExactClosureFailure, joined_types::*, scalar::ExactEndpointFailure};
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedEvidenceClass {
    Failed,
    Incomplete,
    Cancelled,
}

impl JoinedConformanceFailure {
    /// Classify the retained typed cause; example: `failure.classification()`.
    pub(crate) fn classification(&self) -> JoinedEvidenceClass {
        match &self.cause {
            JoinedConformanceCause::Cancelled => JoinedEvidenceClass::Cancelled,
            JoinedConformanceCause::Endpoint { cause, .. } => endpoint_class(cause),
            JoinedConformanceCause::Closure { cause, .. } => closure_class(cause),
            _ => JoinedEvidenceClass::Incomplete,
        }
    }
}

fn endpoint_class(cause: &ExactEndpointFailure) -> JoinedEvidenceClass {
    if let ExactEndpointFailure::Residual(report) = cause
        && matches!(report.decision, Ok(ResidualDecision::Exceeds { .. }))
    {
        return JoinedEvidenceClass::Failed;
    }
    JoinedEvidenceClass::Incomplete
}

fn closure_class(cause: &ExactClosureFailure) -> JoinedEvidenceClass {
    match cause {
        ExactClosureFailure::Endpoint(cause) => endpoint_class(cause),
        ExactClosureFailure::SourceBounds { .. } => JoinedEvidenceClass::Incomplete,
    }
}
