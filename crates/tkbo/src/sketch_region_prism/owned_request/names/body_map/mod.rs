// SPDX-License-Identifier: GPL-2.0-only
//! Read seam only until the actual correspondence resolver owns production issuance.
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
