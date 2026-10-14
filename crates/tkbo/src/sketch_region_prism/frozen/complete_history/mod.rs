// SPDX-License-Identifier: GPL-2.0-only
//! ADR0037 bounded history composition; observation alone never publishes a valid body.
mod bounds;
mod context_names;
mod contexts;
mod copied_contexts;
mod types;
pub(crate) use types::*;
mod census;
mod limits;
mod operands;
mod parent_rows;
mod partition_queries;
mod stage_names;
mod stages;
use super::*;
use limits::*;
#[cfg(test)]
mod bounds_tests;

#[cfg(test)]
mod context_tests;

#[cfg(test)]
mod copied_context_tests;

#[cfg(test)]
mod fixture;

mod observation_types;
pub(crate) use observation_types::*;
mod candidate_copy;
mod generated;
mod generated_directed;
mod intermediate;
mod observed_values;
mod relations;
pub(crate) use intermediate::PrismIntermediateHistory;
mod partition_metadata;
pub(crate) use partition_metadata::{
    PrismPartitionEndpoint, PrismPartitionMetadata, PrismPartitionVertex,
};
mod sweep;
pub(crate) use sweep::{PrismCompleteSweepQuery, PrismNativeSweepAnswer};
mod continuation;
mod sweep_lineage;
pub(crate) use continuation::PrismOriginalContinuation;
mod capture;
mod result;
pub(crate) use result::*;
mod operation;
mod replay;
pub(crate) use operation::{
    inspect_complete_prism_history, rebind_complete_prism_history, reproject_complete_prism_history,
};
mod admission;
mod diagnostics;
#[cfg(test)]
mod adversarial_tests;
mod budget;
#[cfg(test)]
mod operation_tests;
