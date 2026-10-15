// SPDX-License-Identifier: GPL-2.0-only
//! General planar regions and extrusion (ADR 0068): the public seam a feature evaluator uses
//! to turn a solved sketch into validated faces and an extruded solid with complete history.
//!
//! - [`build_planar_regions`] converts solved lines, arcs and circles into every bounded
//!   planar face (outer loop and holes), with a complete curve to edge mapping.
//!
//! Every operation is non-destructive and either succeeds completely or returns a typed
//! [`OperationDecline`]; no partial result exists.
mod admission;
mod admitted;
mod bezier;
mod build_regions;
mod contacts;
mod decline;
mod loop_geometry;
mod loops;
mod nesting;
mod plane_math;
mod profile;
mod regions;
mod resolution;
mod seeds;
mod topology;
mod validation;

pub use build_regions::build_planar_regions;
pub use decline::{
    DeclineCause, DeclineClass, OperationDecline, OperationKind, OperationStage,
    ValidationStageKind,
};
pub use loops::{LoopUse, ProfileLoop};
pub use profile::*;
pub use regions::*;
pub use resolution::*;
pub use validation::{Finding, SelfIntersectionEvidence, StageStatus, ValidationReport};
