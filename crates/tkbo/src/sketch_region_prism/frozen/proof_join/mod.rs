// SPDX-License-Identifier: GPL-2.0-only
//! ADR0035: bind existing proofs to the same authored immutable projection.
mod access;
mod operands;
mod operation;
mod probe;
mod rebind;
pub(crate) use operation::{compose_prism_proof, rebind_joined_prism_proof};
mod closure;
mod error;
mod replay_closure;
pub(crate) use error::*;
mod links;
mod parents;
mod raw_vertices;
mod replay;
mod replay_options;
mod replay_sources;
mod replay_theorem;
mod types;
use super::{bindings::*, encoding, identities::*, types::*};
use crate::sketch_region_prism::region_certificate::{current_binding::*, embedding::*};
use crate::sketch_region_prism::surface_charts::*;
pub(crate) use types::*;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(in crate::sketch_region_prism) mod operation_tests;

pub(in crate::sketch_region_prism) use replay::{
    block as encode_joined_block, text as encode_joined_text,
};
pub(in crate::sketch_region_prism) use replay_options::{
    embedding as encode_joined_embedding_options, roots as encode_joined_root_options,
};

#[allow(unused_imports)]
pub(super) use links::link_face as link_frozen_chart;
#[allow(unused_imports)]
pub(super) use parents::{
    append_remaining_history as append_frozen_remaining_history,
    current_dispositions as link_frozen_parent_dispositions,
};
#[allow(unused_imports)]
pub(super) use raw_vertices::append as encode_frozen_raw_vertices;
#[allow(unused_imports)]
pub(super) use raw_vertices::capture as capture_frozen_raw_vertices;
