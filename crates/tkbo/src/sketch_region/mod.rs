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

pub(crate) use occurrences::equal_occurrences;

pub(crate) use copy_inputs::{admit_sources, copy_sources, original_identity};

mod partition_observation;
mod partition_types;
pub(crate) use partition_observation::{observe_owned_partition, observe_partition_space};
pub(crate) use partition_types::*;
#[cfg(test)]
mod partition_tests;

mod partition_capture;

mod partition_blocks;
mod partition_diagnostics;

mod partition_vertices;

#[cfg(test)]
mod partition_policy_tests;
mod partition_source_vertices;
