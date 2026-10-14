// SPDX-License-Identifier: GPL-2.0-only
//! ADR0050: source-keyed correspondence for the actual annular sweep, not body validity.
mod boundaries;
mod capture;
mod charts;
mod correspondence;
pub(in crate::sketch_region_prism) use correspondence::snapshot as snapshot_annular_context;
pub(in crate::sketch_region_prism) use replay::context as encode_annular_context;
mod failure;
mod failure_class;
mod operation;
mod parents;
mod rebind;
mod replay;
mod settings;
pub(in crate::sketch_region_prism) use operation::bind_annular_prism_v2;
pub(in crate::sketch_region_prism) use rebind::rebind_annular_prism_v2;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod tests;
mod types;
mod views;
use super::super::*;
use super::two_loop::*;
use crate::sketch_region::{PartitionOccurrenceIdentity, RegionObservedContext, RegionSourceId};
pub(in crate::sketch_region_prism) use failure::*;
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(in crate::sketch_region_prism) use types::*;
