// SPDX-License-Identifier: GPL-2.0-only
//! ADR0055: four named conditional face areas and signed material sweep on one sealed domain.
mod access;
mod caps;
mod error;
mod failure_codec;
mod operands;
mod operation;
mod replay;
mod sides;
mod types;
mod work;
use super::{annular_join::*, annular_orientation::*, joined_measures::*, *};
use crate::sketch_region_prism::PrismImageRole;
pub(in crate::sketch_region_prism) use error::{
    AnnularMeasureCause, AnnularMeasureFailure, AnnularMeasurePhase,
};
use error::{count, operand};
pub(in crate::sketch_region_prism) use failure_codec::annular_measure_failure_bytes;
pub(in crate::sketch_region_prism) use operation::inspect_annular_measures;
pub(in crate::sketch_region_prism) use replay::options as encode_annular_measure_options;
use rustycad_tkmath::predicates::roots::metric_bounds::*;
pub(in crate::sketch_region_prism) use types::*;
use work::MeasureWork;
type MeasureCheck<T> = Result<T, AnnularMeasureCause>;

#[cfg(test)]
mod tests;
