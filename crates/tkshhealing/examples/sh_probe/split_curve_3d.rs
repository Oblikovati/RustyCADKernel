//! Native 3D curve-split geometry, shared arrays and status histories.
use super::{
    brep_lib_common::geom_curve as read_curve, sh_common::curve as curve_digest,
    shape_construct::construct_spline_3d as spline_digest,
};
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgradeSplitCurve3d as ProbeSplitter, SplitCurves3d as ProbeCurves,
};
include!("split_curve_geometry.inc.rs");
pub use curve_geometry_case as split_curve_3d_case;
