// SPDX-License-Identifier: GPL-2.0-only
//! ADR0033: exact finite incidence certificate, independent of geometric body validity.
mod admission;
mod containment;
mod domain;
mod execution;
mod incidence;
mod links;
mod model;
mod pairing;
mod types;
mod views;
mod wires;
pub(crate) use execution::certify_frozen_topology;
pub(crate) use types::*;
#[cfg(test)]
mod combinatorial_tests;
#[cfg(test)]
mod containment_tests;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod tests;

mod traversal_binding;
#[cfg(test)]
mod traversal_tests;
