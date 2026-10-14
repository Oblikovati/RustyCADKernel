// SPDX-License-Identifier: GPL-2.0-only
//! ADR0046: bounded two-loop planar-domain evidence; no annular body acceptance.
mod admission;
mod availability;
#[cfg(test)]
mod candidate_tests;
mod candidate_wires;
mod candidates;
mod census_settings;
mod current_history;
#[cfg(test)]
mod domain_refusal_tests;
mod endpoints;
mod failure;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod fixture;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod fixture_bindings;
mod geometry;
mod history_budget;
#[cfg(test)]
mod limit_tests;
mod mapped_geometry;
#[cfg(test)]
mod mapped_parent_tests;
mod mapped_parents;
pub(in crate::sketch_region_prism) use mapped_geometry::admit_two_loop_polynomial_counts;
mod member_capture;
mod member_parents;
mod member_types;
pub(in crate::sketch_region_prism) use mapped_parents::{
    TwoLoopMappedParents, map_two_loop_original_parents, map_two_loop_original_parents_after,
};
#[cfg(test)]
mod mutation_tests;
mod nesting;
mod operation;
#[cfg(test)]
mod operation_tests;
mod outcome;
mod pair_capture;
mod pair_graph;
#[cfg(test)]
mod pair_tests;
mod pair_types;
#[cfg(test)]
mod pipeline_tests;
mod plane;
mod replay;
#[cfg(test)]
mod replay_tests;
mod request;
mod source_stages;
#[cfg(test)]
mod source_tests;
mod sources;
mod winding;
#[cfg(test)]
mod winding_tests;
mod work;
#[cfg(test)]
mod work_tests;

mod views;
pub(crate) use failure::*;
pub(crate) use operation::{inspect_two_loop_region, rebind_two_loop_region};
pub(crate) use outcome::*;
pub(crate) use request::*;
pub(crate) use views::TwoLoopRegionLimitation;

mod census_budget;
mod operand_budget;
mod source_budget;

#[cfg(test)]
mod budget_tests;

mod failure_class;

#[cfg(test)]
mod callback_budget_tests;

#[cfg(test)]
mod refusal_tests;

#[cfg(test)]
mod membership_refusal_tests;

pub(crate) use failure_class::TwoLoopRefusalClass;
#[allow(unused_imports)]
pub(in crate::sketch_region_prism) use failure_class::{
    candidate as classify_region_candidate_cause, surface as classify_region_surface_cause,
};
#[allow(unused_imports)]
pub(in crate::sketch_region_prism) use member_types::{
    TwoLoopMemberKey, TwoLoopMemberParents, TwoLoopMemberRole, TwoLoopOriginalUse,
};
#[cfg(test)]
mod native_refusal_tests;
