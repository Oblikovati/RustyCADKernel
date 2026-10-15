// SPDX-License-Identifier: GPL-2.0-only
//! General planar regions and extrusion (ADR 0068): the public seam a feature evaluator uses
//! to turn a solved sketch into validated faces and an extruded solid with complete history.
//!
//! - [`build_planar_regions`] converts solved lines, arcs and circles into every bounded
//!   planar face (outer loop and holes), with a complete curve to edge mapping.
//! - [`extrude`] sweeps one region along a vector into an [`OperationResult`]: the solid,
//!   its complete source to result history, findings, validation and achieved tolerance.
//!
//! Every operation is non-destructive and either succeeds completely or returns a typed
//! [`OperationDecline`]; no partial result exists.
mod admission;
mod admitted;
mod bezier;
mod build_regions;

mod decline;
mod extrude;
mod loop_geometry;
mod loops;
mod operation_result;
mod plane_math;
mod profile;
mod proof;
mod proof_pieces;
mod region_area;
mod regions;
mod resolution;
mod solid_validation;
mod sweep_history;
mod sweep_images;
mod topology;
mod validation;

pub use crate::body_check::ValidationReport;
pub use crate::measured::{Evidence, IntegrationMethod, Measured, MeasuredPoint};
pub use build_regions::build_planar_regions;
pub use decline::{
    DeclineCause, DeclineClass, OperationDecline, OperationKind, OperationStage,
    ValidationStageKind,
};
pub use extrude::{ExtrudeOptions, extrude};
pub use loops::{LoopUse, ProfileLoop};
pub use operation_result::*;
pub use profile::*;
pub use regions::*;
pub use resolution::*;
pub use validation::{Finding, RegionValidation, SelfIntersectionEvidence, StageStatus};
