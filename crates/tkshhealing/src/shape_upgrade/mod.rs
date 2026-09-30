// Rust translation of ModelingAlgorithms/TKShHealing/ShapeUpgrade at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeUpgrade: the tools that split and convert shapes (division by
//! continuity, angle or area, UnifySameDomain).
//!
//! Ported so far (docs/tkshhealing-progress.md): the package functions [`ShapeUpgrade`]
//! (C0 B-splines split into C1 pieces), [`ShapeUpgradeTool`] (the context and
//! tolerances of the tools), [`ShapeUpgradeRemoveLocations`] and the base curve splitter
//! [`SplitCurve`] with [`ShapeUpgradeSplitCurve`]. docs/shhealing-design.md section 8
//! fixes how they map. The classes that use ShapeAnalysis or ShapeFix classes wait for
//! those packages.
mod package;
mod remove_locations;
mod split_curve;
mod tool;

pub use package::{BoundedCurve2dSequence, BoundedCurveSequence, ShapeUpgrade};
pub use remove_locations::{ShapeReplacements, ShapeUpgradeRemoveLocations};
pub use split_curve::{ShapeUpgradeSplitCurve, SplitCurve, SplitCurveData, SplitCurveFields};
pub use tool::ShapeUpgradeTool;

mod shell_sewing;
pub use shell_sewing::ShapeUpgradeShellSewing;

mod split_curve_3d;
pub use split_curve_3d::{ShapeUpgradeSplitCurve3d,SplitCurves3d};
