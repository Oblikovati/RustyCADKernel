// Rust translation of ModelingData/TKGeomBase/AppDef (package overview)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! AppDef: the approximation of discrete point sets. The points are a
//! [`MultiLine`] of [`MultiPointConstraint`]s read through the AppDef_MyLineTool
//! [`crate::app_par_curves::LineTool`] implementation; the Bezier and B-spline
//! drivers ([`Compute`], [`BSplineCompute`]) and the least-squares, function,
//! gradient and BFGS classes are aliases of the AppParCurves and Approx generics
//! over it; [`Variational`] smooths the points by minimizing the quadratic error
//! and the tension, flexion and jerk criteria of a [`LinearCriteria`], an
//! [`SmoothCriterion`].
mod instantiations;
mod linear_criteria;
mod multi_line;
mod multi_point_constraint;
mod my_line_tool;
mod point_tabs;
mod smooth_criterion;
mod variational;

pub use instantiations::*;
pub use linear_criteria::LinearCriteria;
pub use multi_line::MultiLine;
pub use multi_point_constraint::MultiPointConstraint;
pub use smooth_criterion::{QualityValues, SmoothCriterion, SmoothErrors};
pub use variational::{Variational, VariationalOptions};
