// SPDX-License-Identifier: GPL-2.0-only
//! ADR0030 keeps raw metadata caps separate from independently certified world residuals.
#[cfg(test)]
mod compatibility_tests;
mod policy;
#[cfg(test)]
mod policy_tests;

pub(super) use policy::*;
