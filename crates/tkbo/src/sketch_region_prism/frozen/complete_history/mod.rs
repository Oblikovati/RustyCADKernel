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
pub(in crate::sketch_region_prism) mod fixture;

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
    inspect_bound_prism_history, inspect_complete_prism_history, rebind_complete_prism_history,
    reproject_complete_prism_history,
};
mod admission;
#[cfg(test)]
mod adversarial_tests;
mod budget;
mod diagnostics;
#[cfg(test)]
mod operation_tests;
mod sweep_coverage;

pub(in crate::sketch_region_prism) use copied_contexts::resolve_native_copy_context;
pub(in crate::sketch_region_prism) use generated_directed::resolve_generated_context;
pub(in crate::sketch_region_prism) use operands::{
    copied_contexts as retained_copy_contexts, current_contexts as capture_history_contexts,
    observed_contexts as retained_history_contexts,
};
pub(in crate::sketch_region_prism) use partition_queries::resolve_native_partition_context;
