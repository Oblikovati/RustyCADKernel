// SPDX-License-Identifier: GPL-2.0-only
//! Fixed semantic symbols precede geometry lookup; reserved symbols prove no correspondence.
mod identities;
mod plan;
mod replay;
mod roles;
pub(crate) use identities::*;
pub(crate) use plan::*;
pub(crate) use replay::*;
pub(crate) use roles::*;
#[cfg(test)]
mod tests;

mod body_keys;
pub(crate) mod body_map;
pub(crate) use body_keys::*;

#[cfg(test)]
mod v2_tests;
pub(crate) use body_map::{
    encode_frozen_member_key, encode_semantic_member_key, encode_semantic_use_key,
};

pub(in crate::sketch_region_prism::owned_request) mod reservations;
