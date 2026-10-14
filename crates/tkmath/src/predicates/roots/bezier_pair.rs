// SPDX-License-Identifier: GPL-2.0-only
//! Actual paired source equations; no complete provider claim before lifting and coverage gates.
mod equations;
mod polynomial;
mod projection;
mod source;

mod fiber_row;

mod algebraic;
mod operation;
#[cfg(test)]
mod operation_tests;

mod construction_refine;
mod contact_construction;
mod graph_enclosure;
mod membership;
#[cfg(test)]
mod native_replay;
mod operation_failure;
#[cfg(test)]
mod operation_limits_tests;

mod exceptional;

mod source_certificate;

#[cfg(test)]
mod algebraic_world_tests;
mod candidate_certificate;
mod membership_certificate;
#[cfg(test)]
mod retention_tests;

#[cfg(test)]
mod refinement_tests;
mod work_budget;
#[cfg(test)]
mod work_budget_tests;
