// SPDX-License-Identifier: GPL-2.0-only
//! ADR0023 private diagnostic projection. Captured never means a valid published body.
mod admission;
mod bindings;
mod capture;
mod coverage;
mod encoding;
mod endpoints;
mod geometry;
mod graph;
mod history;
mod identities;
mod types;
mod views;
pub(crate) use capture::freeze_prism;
pub(crate) use types::*;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod mutation_tests;
