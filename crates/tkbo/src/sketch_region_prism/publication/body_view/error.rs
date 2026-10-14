// SPDX-License-Identifier: GPL-2.0-only
use crate::sketch_region_prism::owned_request::names::{
    CallerBoundaryId, CallerOperationId, SemanticMemberKey, SemanticUseKey,
    body_map::{PrismSemanticMapFailure, ResolvedPrismNames},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismBodyProjectionClass {
    Incomplete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PrismBodyProjectionEntity {
    Member(SemanticMemberKey),
    Occurrence(SemanticUseKey),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PrismBodyProjectionCause {
    PublicationBinding,
    UnsupportedVersion {
        actual: u32,
        expected: u32,
    },
    SemanticMap(Box<PrismSemanticMapFailure>),
    Coverage {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
    OperandMismatch {
        required: &'static str,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrismBodyProjectionFailure {
    pub operation: CallerOperationId,
    pub source: CallerBoundaryId,
    pub version: u32,
    pub entity: Option<PrismBodyProjectionEntity>,
    pub class: PrismBodyProjectionClass,
    pub cause: PrismBodyProjectionCause,
}
pub(super) type BodyCheck<T> = Result<T, Box<PrismBodyProjectionFailure>>;
pub(super) fn failure(
    names: &ResolvedPrismNames,
    cause: PrismBodyProjectionCause,
) -> Box<PrismBodyProjectionFailure> {
    let ids = names.plan().identities();
    Box::new(PrismBodyProjectionFailure {
        operation: ids.operation,
        source: ids.boundary,
        version: 1,
        entity: None,
        class: PrismBodyProjectionClass::Incomplete,
        cause,
    })
}
pub(super) fn count(
    names: &ResolvedPrismNames,
    required: &'static str,
    actual: usize,
    expected: usize,
) -> BodyCheck<()> {
    if actual == expected {
        return Ok(());
    }
    Err(failure(
        names,
        PrismBodyProjectionCause::Coverage {
            required,
            actual,
            expected,
        },
    ))
}

pub(super) fn missing_traversal(names: &ResolvedPrismNames) -> Box<PrismBodyProjectionFailure> {
    failure(
        names,
        PrismBodyProjectionCause::Coverage {
            required: "captured OrderedTraversalV2 wire rows",
            actual: 0,
            expected: 1,
        },
    )
}
