//! The TDataXtd package: the extended attributes of OCAF (ocaf-design.md section 11):
//! construction geometry over named shapes, constraints, patterns, positions, presentation
//! settings and triangulations.
mod constraint;
mod construction;
mod geometry;
mod package;
mod pattern;
mod position;
mod presentation;
pub mod presentation_colors;
mod triangulation;

pub use constraint::TDataXtdConstraint;
pub use construction::{
    TDataXtdAxis, TDataXtdPlacement, TDataXtdPlane, TDataXtdPoint, TDataXtdShape,
};
pub use geometry::TDataXtdGeometry;
pub use package::{TDataXtd, TDataXtdConstraintEnum, TDataXtdGeometryEnum};
pub use pattern::{TDataXtdPattern, TDataXtdPatternStd};
pub use position::TDataXtdPosition;
pub use presentation::TDataXtdPresentation;
pub use triangulation::TDataXtdTriangulation;
