// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Read access: local evaluation, knots, multiplicities, poles and weights, through
//! the accessors shared with the other B-spline curve class.
use super::Geom2dBSplineCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};

impl Geom2dBSplineCurve {
    crate::bspline_curve_accessors!(Point2, Vector2);
}
