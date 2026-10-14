// SPDX-License-Identifier: GPL-2.0-only
//! ADR0052: named frozen parameter domains, conditional on the same actual annular proof.
mod access;
mod capture;
mod census;
mod links;
mod names;
mod operation;
mod rebind;
mod replay;
mod types;
use super::{bindings::*, identities::*, proof_join::*, types::*};
use crate::sketch_region::{PartitionOccurrenceIdentity, RegionSourceId};
use crate::sketch_region_prism::{
    PrismReceipt,
    region_certificate::{annular_binding::*, embedding::*, two_loop::*},
};
pub(in crate::sketch_region_prism) use operation::join_annular_frozen_domain;
pub(in crate::sketch_region_prism) use rebind::rebind_annular_frozen_domain;
pub(in crate::sketch_region_prism) use types::*;
#[cfg(test)]
mod mutation_tests;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod operation_tests;
#[cfg(test)]
mod refusal_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod topology_tests;
