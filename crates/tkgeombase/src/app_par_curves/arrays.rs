// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_*.gxx (shared scratch arrays)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The scratch arrays and the MultiLine point reads shared by the AppParCurves
//! generics.
use super::line_tool::LineTool;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::Vector;

/// An array `1..=max(count, 1)` of default values (the native `mynbP` sizes).
pub(crate) fn scratch<T: Default>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count.max(1)).map_err(|error| array_failure(error, "AppParCurves"))
}

/// The multipoint `index` of `ssp` through the Value overload the generics pick:
/// both arrays when there are 3D and 2D points, else the 2D or the 3D one.
pub(crate) fn line_points(
    ssp: &impl LineTool,
    index: i32,
    points: &mut Array1<'_, Point3>,
    points2d: &mut Array1<'_, Point2>,
) -> Result<(), Failure> {
    let (nb_p, nb_p2d) = (ssp.nb_p3d()?, ssp.nb_p2d()?);
    if nb_p != 0 && nb_p2d != 0 {
        return ssp.value(index, points, points2d);
    }
    if nb_p2d != 0 {
        return ssp.value_2d(index, points2d);
    }
    ssp.value_3d(index, points)
}

/// An owned copy of `v` with its bounds (the native math_Vector copy);
/// `owned_vector(&parameters)?`.
pub(crate) fn owned_vector(v: &Vector<'_>) -> Result<Vector<'static>, Failure> {
    let mut copy: Vector<'static> = Vector::with_bounds(v.lower(), v.upper())?;
    copy.initialized(v)?;
    Ok(copy)
}
