// SPDX-License-Identifier: GPL-2.0-only
//! ADR0054: exact conditional directions on one immutable, fully named annular domain.
mod access;
mod error;
mod faces;
mod lookup;
mod operation;
mod replay;
mod trims;
mod types;
mod winding;
use super::{annular_join::*, proof_join::*, *};
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_prism::region_certificate::{
    orientation::*, two_loop::TwoLoopRefusalClass,
};
use crate::sketch_region_prism::{PrismImageRole, surface_charts::PrismTrimRole};
pub(in crate::sketch_region_prism) use error::*;
pub(in crate::sketch_region_prism) use operation::inspect_annular_orientation;
use rustycad_tkg3d::top_abs::Orientation;
pub(in crate::sketch_region_prism) use trims::source_for_trim as annular_trim_source;
pub(in crate::sketch_region_prism) use types::*;
pub(in crate::sketch_region_prism) use winding::annular_boundary_directions;
type AnnularDirectionCheck<T> = Result<T, AnnularDirectionCause>;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod fixture;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod variant_tests;
