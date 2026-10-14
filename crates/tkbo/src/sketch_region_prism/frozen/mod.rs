// SPDX-License-Identifier: GPL-2.0-only
//! ADR0023 private diagnostic projection. Captured never means a valid published body.
mod admission;
mod alerts;
mod bindings;
mod capture;
pub(crate) mod complete_history;
mod coverage;
mod domain;
mod encoding;
mod endpoints;
mod geometry;
mod graph;
mod history;
mod identities;
mod source_bindings;
mod types;
mod views;
pub(crate) use capture::{freeze_prism, freeze_prism_with_behavior};
pub(crate) use types::*;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod mutation_tests;

#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod isolation_tests;

#[cfg(test)]
mod expected_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod source_mutation_tests;

#[cfg(test)]
mod diagnostic_tests;

#[cfg(test)]
mod tolerance_tests;
mod tolerances;

#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod endpoint_tests;

#[cfg(test)]
mod admission_tests;

mod error_context;
#[cfg(test)]
mod error_context_tests;

pub(crate) mod proof_join;
#[cfg(test)]
mod topology_baseline_tests;

pub(crate) mod topology;

pub(crate) mod lineage;
mod traversal;
#[cfg(test)]
mod traversal_tests;

#[cfg(test)]
mod ordered_validation_baseline_tests;

pub(crate) mod joined_measures;
pub(crate) mod joined_orientation;
pub(crate) mod joined_volume;
pub(crate) mod ordered_validation;
pub(crate) mod owned_refusal;
#[cfg(test)]
mod owned_refusal_baseline_tests;
pub(super) use encoding::{numbers as encode_frozen_numbers, placement as encode_frozen_placement};
#[cfg(test)]
pub(in crate::sketch_region_prism) mod publication_read_snapshot;
