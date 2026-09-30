// Rust translation of ModelingData/TKGeomBase/AppParCurves (package overview)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! AppParCurves: the parametric approximation of MultiLines by MultiCurves — the
//! Bernstein and B-spline basis matrices, the multipoint and multicurve containers,
//! the least-squares fit with extremity constraints ([`LeastSquare`]), the
//! constrained fit by Uzawa ([`ResolConstraint`]), the parameter functions
//! ([`Function`], [`BSpFunction`]) and the parameter optimizers ([`Gradient`],
//! [`BSpGradient`]).
//!
//! The native `.gxx` generics take a `MultiLine` and its `ToolLine`; the port takes
//! any type implementing [`LineTool`].
pub(crate) mod arrays;
mod bernstein;
mod bsp_function;
mod bsp_gradient;
mod constraint;
mod fit_targets;
mod function;
mod gradient;
mod gradient_bfgs;
mod least_square;
mod line_tool;
mod multi_bsp_curve;
mod multi_curve;
mod multi_point;
mod resol_constraint;
mod spline_function;

pub use bernstein::{bernstein, bernstein_matrix, second_derivative_bernstein};
pub use bsp_function::BSpFunction;
pub use bsp_gradient::{BSpGradient, BSpGradientInput};
pub use constraint::{Constraint, ConstraintCouple};
pub use function::{Function, point_constraint};
pub use gradient::Gradient;
pub use gradient_bfgs::{FitFunction, GradientBfgs};
pub use least_square::{FitErrors, LeastSquare, nb_b_columns, the_first_point, the_last_point};
pub use line_tool::LineTool;
pub use multi_bsp_curve::MultiBSpCurve;
pub use multi_curve::{MultiCurve, MultiCurveEval};
pub use multi_point::MultiPoint;
pub(crate) use multi_point::copy_array;
pub use resol_constraint::ResolConstraint;
pub use spline_function::spline_function;

use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;

/// `occ::handle<NCollection_HArray1<AppParCurves_ConstraintCouple>>`: the
/// constraints of a MultiLine, shared by the generics.
pub type Constraints = Handle<HArray1<ConstraintCouple>>;
