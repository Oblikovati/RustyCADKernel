// SPDX-License-Identifier: GPL-2.0-only
//! ADR0030 keeps raw metadata caps separate from independently certified world residuals.
mod census;
mod census_contexts;
#[cfg(test)]
mod census_tests;
mod census_types;
#[cfg(test)]
mod compatibility_tests;
mod endpoint_census;
mod endpoint_contexts;
#[cfg(test)]
mod endpoint_tests;
mod history_census;
#[cfg(test)]
mod legacy_baseline_tests;
mod policy;
#[cfg(test)]
mod policy_tests;
mod trim_census;

pub(super) use policy::*;

mod inspect;
mod inspection_types;
mod metadata;
mod rebind;
mod residuals;
pub(crate) use inspect::inspect_prism_tolerances;
pub(crate) use inspection_types::*;
#[cfg(test)]
mod displacement_tests;
#[cfg(test)]
mod inspection_fixture;
#[cfg(test)]
mod inspection_tests;
#[cfg(test)]
mod metric_operand_tests;
#[cfg(test)]
mod progress_tests;
#[cfg(test)]
mod replay_tests;
