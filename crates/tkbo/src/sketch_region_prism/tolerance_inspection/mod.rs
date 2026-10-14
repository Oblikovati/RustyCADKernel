// SPDX-License-Identifier: GPL-2.0-only
//! ADR0030 keeps raw metadata caps separate from independently certified world residuals.
mod census;
mod census_contexts;
#[cfg(test)]
mod census_tests;
mod census_types;
#[cfg(test)]
mod compatibility_tests;
mod history_census;
#[cfg(test)]
mod legacy_baseline_tests;
mod policy;
#[cfg(test)]
mod policy_tests;
mod trim_census;

pub(super) use policy::*;
