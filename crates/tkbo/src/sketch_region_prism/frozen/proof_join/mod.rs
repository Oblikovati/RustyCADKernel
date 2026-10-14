// SPDX-License-Identifier: GPL-2.0-only
//! ADR0035: bind existing proofs to the same authored immutable projection.
mod access;
mod operation;
mod probe;
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

pub(in crate::sketch_region_prism) use replay::{block as encode_joined_block, text as encode_joined_text};
pub(in crate::sketch_region_prism) use replay_options::roots as encode_joined_root_options;
