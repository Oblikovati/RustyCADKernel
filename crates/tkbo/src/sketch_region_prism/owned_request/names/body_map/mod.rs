// SPDX-License-Identifier: GPL-2.0-only
//! Portable semantic map; production issuance is sealed behind actual resolver/publication ownership.
mod checks;
mod expansion;
mod replay;
mod types;
pub(crate) use replay::{encode_semantic_member_key, encode_semantic_use_key};
pub(crate) use types::*;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use fixture::{fixture_bound_names, fixture_unbound_symbolic_names};

#[cfg(test)]
mod test_fixture;

#[cfg(test)]
mod adversarial_tests;

mod issued;
pub(in crate::sketch_region_prism::owned_request) use issued::issue_resolved_prism_names;

#[cfg(test)]
mod copied_base_tests;
