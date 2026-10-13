// SPDX-License-Identifier: GPL-2.0-only
//! Owned arrangement prerequisite for ADR0014, not a completed region or solid operation.
mod arrangement;
mod copy_inputs;
mod occurrences;
mod outcome;
mod relations;

pub(crate) use arrangement::arrange_owned_sources;
pub(crate) use outcome::{
    ArrangementCause, ArrangementCompletion, ArrangementPhase, ArrangementRelation,
    ArrangementSource, OriginalMemberIdentity, OwnedArrangement, RegionSourceId,
};
#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
mod isolation_tests;

#[cfg(test)]
mod native_batch_tests;

#[cfg(test)]
mod native_prism_tests;

#[cfg(test)]
mod empty_history_tests;

#[cfg(test)]
mod occurrence_tests;
