// SPDX-License-Identifier: GPL-2.0-only
//! Actual DS incidence is independent of canonical algebraic contact identity.
mod inspect;
mod outcome;
pub(crate) use inspect::inspect_partition_bindings;
pub(crate) use outcome::*;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod native_tests;
#[cfg(test)]
mod span_tests;
mod spans;

#[cfg(test)]
mod copy_tests;
#[cfg(test)]
mod image_link_tests;
mod image_links;
mod integrity;
#[cfg(test)]
mod integrity_tests;
#[cfg(test)]
mod source_domain_tests;
mod source_domains;

mod candidate_history;
mod copy_disposition;
mod history_parents;
mod history_types;
mod occurrence_keys;
mod transaction;
pub(crate) use transaction::{
    PartitionBoundOutputs, PartitionCaptureState, PartitionTransactionInspection,
    RequiredPartitionProof, bind_partition_outputs,
};
#[cfg(test)]
mod candidate_history_tests;
mod copied_tree_bindings;
#[cfg(test)]
mod corpus;
#[cfg(test)]
mod corpus_tests;
#[cfg(test)]
mod image_partition_tests;
mod image_partitions;
#[cfg(test)]
mod isolation_tests;
#[cfg(test)]
mod transaction_tests;

#[cfg(test)]
mod generated_native_tests;

mod generated_bindings;

#[cfg(test)]
mod generated_binding_tests;

#[cfg(test)]
mod generated_native_copy_tests;
