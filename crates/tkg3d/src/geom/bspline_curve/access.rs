// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve_1.cxx (read access) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Read access: local evaluation, knots, multiplicities, poles and weights, through
//! the accessors shared with the other B-spline curve class.
use super::GeomBSplineCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3};

impl GeomBSplineCurve {
    rustycad_tkg2d::bspline_curve_accessors!(Point3, Vector3);
}
