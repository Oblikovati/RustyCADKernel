// SPDX-License-Identifier: GPL-2.0-only
//! ADR0014 source-bound census certificates do not certify the mapped candidate search.
mod candidate_coverage;
mod capture;
mod census;
mod coverage;
mod outcome;
pub(crate) use candidate_coverage::inspect_candidate_coverage;
pub(crate) use capture::capture_boundary_inputs;
pub(crate) use census::{enumerate_boundary_census, enumerate_source_bound_census};
pub(crate) use coverage::inspect_contact_coverage;
pub(crate) use outcome::*;
#[cfg(test)]
mod tests;

mod coverage_status;
mod incidence_binding;
mod parameter_binding;
mod source_status;

#[cfg(test)]
mod binding_mutation_tests;
#[cfg(test)]
mod source_binding_tests;

#[cfg(test)]
mod coverage_binding_tests;
