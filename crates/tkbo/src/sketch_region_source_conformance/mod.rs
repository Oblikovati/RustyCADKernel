// SPDX-License-Identifier: GPL-2.0-only
//! Original-source certificates do not authorize contact identity, candidate search or region success.
mod capture;
mod inspect;
mod membership;
mod query;
pub(crate) use query::{captured_map, source_query};
mod types;
pub(crate) use capture::capture_support;
pub(crate) use capture::{capture_construction, capture_originals, capture_space_representation};
pub(crate) use inspect::inspect_source_conformance;
pub(crate) use membership::native_parameter_membership;
pub(crate) use types::*;
#[cfg(test)]
mod tests;

mod binding;
mod binding_types;
mod descriptor_bits;
mod snapshot_binding;
mod stage_binding;
pub(crate) use binding::{
    bind_source_conformance, current_construction_correspondence, source_binding_faults,
};
pub(crate) use binding_types::{BoundSourceConformance, SourceBindingFault};
pub(crate) use descriptor_bits::{same_embedding, same_metric_curve, same_space_representation};

pub(crate) use snapshot_binding::binds_current_segment;

pub(crate) use snapshot_binding::same_construction_basis;
