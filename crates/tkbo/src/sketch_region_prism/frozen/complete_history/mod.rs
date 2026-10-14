// SPDX-License-Identifier: GPL-2.0-only
//! ADR0037 bounded history composition; observation alone never publishes a valid body.
mod bounds;
pub(in crate::sketch_region_prism) use bounds::{
    HistoryHierarchyCensus, preflight_history_census, preflight_history_root,
};
pub(super) mod context_names;
pub(super) mod contexts;
pub(super) mod copied_contexts;
mod types;
pub(crate) use types::*;
mod census;
mod limits;
pub(super) mod operands;
pub(super) mod parent_rows;
pub(super) mod partition_queries;
mod stage_names;
mod stages;
use super::*;
pub(in crate::sketch_region_prism) use limits::*;
#[cfg(test)]
mod bounds_tests;

#[cfg(test)]
mod context_tests;

#[cfg(test)]
mod copied_context_tests;

#[cfg(test)]
pub(in crate::sketch_region_prism) mod fixture;

mod observation_types;
pub(crate) use observation_types::*;
pub(super) mod candidate_copy;
pub(super) mod generated;
mod generated_directed;
mod intermediate;
pub(super) mod observed_values;
pub(super) mod relations;
pub(crate) use intermediate::PrismIntermediateHistory;
pub(super) mod partition_metadata;
pub(crate) use partition_metadata::{
    PrismPartitionEndpoint, PrismPartitionMetadata, PrismPartitionVertex,
};
pub(super) mod sweep;
pub(crate) use sweep::{PrismCompleteSweepQuery, PrismNativeSweepAnswer};
mod continuation;
mod sweep_lineage;
pub(crate) use continuation::PrismOriginalContinuation;
mod capture;
mod result;
pub(crate) use result::*;
pub(super) mod operation;
mod replay;
pub(crate) use operation::{
    inspect_bound_prism_history, inspect_complete_prism_history, rebind_complete_prism_history,
    reproject_complete_prism_history,
};
mod admission;
#[cfg(test)]
mod adversarial_tests;
pub(super) mod budget;
pub(super) mod diagnostics;
#[cfg(test)]
mod operation_tests;
mod sweep_coverage;

pub(in crate::sketch_region_prism) use copied_contexts::resolve_native_copy_context;
pub(in crate::sketch_region_prism) use generated_directed::resolve_generated_context;
pub(in crate::sketch_region_prism) use operands::{
    copied_contexts as retained_copy_contexts, current_contexts as capture_history_contexts,
    observed_contexts as retained_history_contexts, resolve_certified_candidate,
};
pub(in crate::sketch_region_prism) use partition_queries::resolve_native_partition_context;

pub(in crate::sketch_region_prism) use census::admit_stage as admit_history_stage_contexts;
pub(in crate::sketch_region_prism) use partition_queries::{
    admit_partition_history_availability, admit_unchanged_partition_query,
};
pub(in crate::sketch_region_prism) use stage_names::{
    append_member as append_historical_member,
    context_snapshot as snapshot_directed_history_context,
};

mod native_inputs;
pub(super) use native_inputs::NativeHistoryInputs;
