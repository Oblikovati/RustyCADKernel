// SPDX-License-Identifier: GPL-2.0-only
//! Wrapper policy never masquerades as a missing producer header (ADR0056).
use super::*;
use crate::sketch_region_prism::{frozen::joined_volume::*, publication::body_view::*};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProjectionRefusalClass {
    Failed,
    Cancelled,
    Incomplete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProjectionClassOrigin {
    OuterCallback,
    RetainedChild,
    BridgePolicyV1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RetainedProjectionClass {
    Unavailable,
    Body(PrismBodyProjectionClass),
    Volume(JoinedVolumeFailureClass),
}
impl OwnedProjectionRefusal {
    /// Wrapper classification with explicit provenance; `failure.classification()` is not a fabricated child header.
    pub(crate) fn classification(&self) -> (ProjectionRefusalClass, ProjectionClassOrigin) {
        if let Some(stop) = &self.callback {
            return (
                callback_class(&stop.cause),
                ProjectionClassOrigin::OuterCallback,
            );
        }
        child_classification(self.retained_child_class())
    }

    /// Read only class fields actually retained by the producer; `failure.retained_child_class()` may be unavailable.
    pub(crate) fn retained_child_class(&self) -> RetainedProjectionClass {
        match &self.original.cause {
            CompleteProjectionCause::Body(value) => RetainedProjectionClass::Body(value.class),
            CompleteProjectionCause::Volume(value) => RetainedProjectionClass::Volume(value.class),
            CompleteProjectionCause::Callback
            | CompleteProjectionCause::Names(_)
            | CompleteProjectionCause::Reservations(_) => RetainedProjectionClass::Unavailable,
        }
    }
}
fn callback_class(cause: &CompletePrismCallbackCause) -> ProjectionRefusalClass {
    match cause {
        CompletePrismCallbackCause::Cancelled => ProjectionRefusalClass::Cancelled,
        CompletePrismCallbackCause::Failed(_) => ProjectionRefusalClass::Failed,
        CompletePrismCallbackCause::Exhausted => ProjectionRefusalClass::Incomplete,
    }
}

fn child_classification(
    class: RetainedProjectionClass,
) -> (ProjectionRefusalClass, ProjectionClassOrigin) {
    match class {
        RetainedProjectionClass::Unavailable => (
            ProjectionRefusalClass::Incomplete,
            ProjectionClassOrigin::BridgePolicyV1,
        ),
        RetainedProjectionClass::Volume(JoinedVolumeFailureClass::Cancelled) => (
            ProjectionRefusalClass::Cancelled,
            ProjectionClassOrigin::RetainedChild,
        ),
        RetainedProjectionClass::Volume(JoinedVolumeFailureClass::Incomplete)
        | RetainedProjectionClass::Body(PrismBodyProjectionClass::Incomplete) => (
            ProjectionRefusalClass::Incomplete,
            ProjectionClassOrigin::RetainedChild,
        ),
    }
}
