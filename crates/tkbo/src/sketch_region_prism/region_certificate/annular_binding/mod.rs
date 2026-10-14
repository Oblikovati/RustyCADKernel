// SPDX-License-Identifier: GPL-2.0-only
//! ADR0050: source-keyed correspondence for the actual annular sweep, not body validity.
mod boundaries;
mod capture;
mod charts;
mod correspondence;
mod failure;
#[cfg(test)]
mod tests;
mod types;
mod views;
use super::super::*;
use super::two_loop::*;
use crate::sketch_region::{PartitionOccurrenceIdentity, RegionObservedContext, RegionSourceId};
pub(in crate::sketch_region_prism) use failure::*;
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(in crate::sketch_region_prism) use types::*;
