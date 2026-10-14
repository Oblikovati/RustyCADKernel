// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(in crate::sketch_region_prism) enum AnnularValidationCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Binding {
        proof: &'static str,
    },
    Topology(Box<topology::TopologyFailure>),
    Orientation(Box<AnnularDirectionFailure>),
    Conformance(Box<AnnularConformanceFailure>),
    SelfIntersection(Box<AnnularTableFailure>),
    Tolerance(Box<AnnularToleranceFailure>),
    Callback {
        stop: Box<AnnularValidationCallbackStop>,
        downstream: Option<Box<ValidationStop>>,
    },
    Cancelled,
}
pub(in crate::sketch_region_prism) struct AnnularValidationFailure {
    pub operation: AuthoredPrismId,
    pub requested: AnnularValidationOptions,
    pub checkpoint: AnnularValidationCheckpoint,
    pub class: OrderedPrismFailureClass,
    pub trace: OrderedPrismStageTrace,
    pub cause: AnnularValidationCause,
    pub source: Box<[u8]>,
    pub partial: AnnularValidationPartial,
}
pub(in crate::sketch_region_prism) struct ValidationStop {
    pub class: OrderedPrismFailureClass,
    pub cause: AnnularValidationCause,
}
impl ValidationStop {
    pub(super) fn incomplete(cause: AnnularValidationCause) -> Self {
        Self {
            class: OrderedPrismFailureClass::Incomplete,
            cause,
        }
    }
    pub(super) fn cancelled() -> Self {
        Self {
            class: OrderedPrismFailureClass::Cancelled,
            cause: AnnularValidationCause::Cancelled,
        }
    }
}
pub(super) fn binding(matches: bool, proof: &'static str) -> ValidationCheck<()> {
    if !matches {
        return Err(ValidationStop::incomplete(
            AnnularValidationCause::Binding { proof },
        ));
    }
    Ok(())
}
