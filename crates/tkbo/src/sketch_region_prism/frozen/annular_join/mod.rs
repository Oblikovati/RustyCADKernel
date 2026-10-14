// SPDX-License-Identifier: GPL-2.0-only
//! ADR0052: named frozen parameter domains, conditional on the same actual annular proof.
mod census;
mod links;
mod names;
mod types;
use super::{bindings::*, identities::*, proof_join::*, types::*};
use crate::sketch_region::{PartitionOccurrenceIdentity, RegionSourceId};
use crate::sketch_region_prism::{
    PrismReceipt,
    region_certificate::{annular_binding::*, embedding::*, two_loop::*},
};
pub(in crate::sketch_region_prism) use types::*;
#[cfg(test)]
mod tests;
