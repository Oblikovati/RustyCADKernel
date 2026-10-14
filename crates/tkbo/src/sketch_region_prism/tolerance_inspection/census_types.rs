// SPDX-License-Identifier: GPL-2.0-only
//! Actual ancestry is the use key; storage order never creates an entity identity.
use super::super::{PrismCause, surface_charts::PrismTrimRole};
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ToleranceUseContext {
    pub members: Vec<PartitionOccurrenceIdentity>,
}

pub(crate) struct ToleranceEntityCensus {
    pub entity: PartitionOccurrenceIdentity,
    pub kind: ShapeEnum,
    pub contexts: Vec<ToleranceUseContext>,
}

pub(crate) struct ToleranceTrimCensus {
    pub face: PartitionOccurrenceIdentity,
    pub edge: PartitionOccurrenceIdentity,
    pub context: ToleranceUseContext,
    pub source: PartitionOccurrenceIdentity,
    pub parents: Vec<OriginalMemberIdentity>,
    pub role: PrismTrimRole,
}

pub(crate) struct PrismToleranceCensus {
    pub contexts: Vec<ToleranceUseContext>,
    pub entities: Vec<ToleranceEntityCensus>,
    pub trims: Vec<ToleranceTrimCensus>,
    pub history: Vec<ToleranceUseHistory>,
}

pub(crate) struct ToleranceUseHistory {
    pub context: ToleranceUseContext,
    pub preserved: Vec<OriginalMemberIdentity>,
    pub modified: Vec<OriginalMemberIdentity>,
    pub generated: Vec<OriginalMemberIdentity>,
}

pub(crate) struct ToleranceCensusFailure {
    pub context: Option<ToleranceUseContext>,
    pub cause: ToleranceCensusCause,
}

pub(crate) enum ToleranceCensusCause {
    Native(Failure),
    Body(PrismCause),
    Count {
        quantity: &'static str,
        actual: usize,
        expected: usize,
    },
    Kind {
        parent: Option<ShapeEnum>,
        actual: ShapeEnum,
        expected: &'static str,
    },
}

pub(super) type CensusCheck<T> = Result<T, ToleranceCensusFailure>;

impl From<Failure> for ToleranceCensusFailure {
    fn from(cause: Failure) -> Self {
        Self {
            context: None,
            cause: ToleranceCensusCause::Native(cause),
        }
    }
}

impl From<PrismCause> for ToleranceCensusFailure {
    fn from(cause: PrismCause) -> Self {
        Self {
            context: None,
            cause: ToleranceCensusCause::Body(cause),
        }
    }
}

pub(super) fn count(quantity: &'static str, actual: usize, expected: usize) -> CensusCheck<()> {
    if actual != expected {
        return Err(ToleranceCensusFailure {
            context: None,
            cause: ToleranceCensusCause::Count {
                quantity,
                actual,
                expected,
            },
        });
    }
    Ok(())
}
