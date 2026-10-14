// SPDX-License-Identifier: GPL-2.0-only
use super::{joined_orientation::*, topology::*, *};
use crate::sketch_region_prism::{
    exact_tolerance::*, region_certificate::embedding::PrismEmbeddingTableFailure,
};
use rustycad_tkernel::standard::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrderedPrismFailureClass {
    Failed,
    Incomplete,
    Cancelled,
}
pub(crate) enum OrderedPrismCause {
    Version { actual: u32, expected: u32 },
    Binding { proof: &'static str },
    Topology(Box<TopologyFailure>),
    Orientation(Box<JoinedPrismOrientationFailure>),
    Conformance(Box<JoinedConformanceFailure>),
    SelfIntersection(Box<PrismEmbeddingTableFailure>),
    Tolerance(Box<JoinedToleranceFailure>),
    Callback(Box<Failure>),
    Cancelled,
}
pub(crate) struct OrderedPrismValidationFailure {
    pub operation: super::super::AuthoredPrismId,
    pub requested: OrderedPrismValidationOptions,
    pub checkpoint: OrderedPrismCheckpoint,
    pub class: OrderedPrismFailureClass,
    pub trace: OrderedPrismStageTrace,
    pub cause: OrderedPrismCause,
    pub source: Box<[u8]>,
}
pub(super) struct OrderedStop {
    pub class: OrderedPrismFailureClass,
    pub cause: OrderedPrismCause,
}
pub(super) type OrderedCheck<T> = Result<T, OrderedStop>;
impl OrderedStop {
    pub(super) fn incomplete(cause: OrderedPrismCause) -> Self {
        let class = OrderedPrismFailureClass::Incomplete;
        Self { class, cause }
    }
    pub(super) fn cancelled() -> Self {
        let class = OrderedPrismFailureClass::Cancelled;
        Self {
            class,
            cause: OrderedPrismCause::Cancelled,
        }
    }
}
impl From<JoinedEvidenceClass> for OrderedPrismFailureClass {
    fn from(value: JoinedEvidenceClass) -> Self {
        match value {
            JoinedEvidenceClass::Failed => Self::Failed,
            JoinedEvidenceClass::Incomplete => Self::Incomplete,
            JoinedEvidenceClass::Cancelled => Self::Cancelled,
        }
    }
}
impl From<OrderedPrismFailureClass> for OrderedPrismStageStatus {
    fn from(value: OrderedPrismFailureClass) -> Self {
        match value {
            OrderedPrismFailureClass::Failed => Self::Failed,
            OrderedPrismFailureClass::Incomplete => Self::Incomplete,
            OrderedPrismFailureClass::Cancelled => Self::Cancelled,
        }
    }
}
