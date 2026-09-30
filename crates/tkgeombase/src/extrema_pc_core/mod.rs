//! The code shared by the ExtremaPC2d and ExtremaPC packages (not a native package).
//!
//! The two native packages declare the same result types, distance function and grid
//! evaluator, once for `gp_Pnt2d`/`Adaptor2d_Curve2d` and once for
//! `gp_Pnt`/`Adaptor3d_Curve`. The port writes each of them once, generic over a
//! [`PcSpace`], and each package names its instantiation with the native class name
//! (`extrema_pc2d::GridEvaluator` is `PcGridEvaluator<Space2d>`). Where the two
//! native copies differ, the difference is a member of [`PcSpace`] or an argument.
pub(crate) mod analytic;
pub(crate) mod conic_roots;
mod constant_distance;
mod curve_ref;
mod distance;
mod endpoints;
mod grid;
pub(crate) mod knot_params;
mod rational_bezier;
mod result;
mod space;
mod validity;

pub use curve_ref::PcCurveRef;
pub use distance::PcDistanceFunction;
pub use endpoints::CurveEvaluator;
pub(crate) use endpoints::{NeighborStep, ValueFn, add_endpoint_extrema_with};
pub use grid::{GridFailurePolicy, PcGridEvaluator};
pub use result::{PcExtremum, PcResult, SearchMode, Status};
pub use space::{PcD1, PcD2, PcPoint, PcSpace, PcVector, Space2d, Space3d};
pub use validity::{
    THE_BEZIER_DEGREE_MULTIPLIER, THE_BEZIER_MIN_SAMPLES, THE_BSPLINE_FALLBACK_SAMPLES,
    THE_BSPLINE_SPAN_MULTIPLIER, THE_DEFAULT_TOLERANCE, THE_OTHER_CURVE_NB_SAMPLES,
    THE_PARAM_TOLERANCE, is_closed_periodic_domain, is_finite_parameter, is_valid_tolerance,
};
