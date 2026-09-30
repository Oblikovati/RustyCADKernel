// Surface-order and approximation helpers from IntTools_FaceFace.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Native quadric ordering, approximation degree selection and cylinder/sphere policy.
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom_int::quadric_parameters;
use rustycad_tkmath::{
    elclib,
    geom_abs::SurfaceType,
    gp::{Cylinder3, Line3, Point3, Sphere3, Vector3},
};
pub(super) fn reverse_surfaces(first: SurfaceType, second: SurfaceType) -> bool {
    // The typed enum has the exact IndexType order for every representable native kind.
    (first as i32) < (second as i32)
}
pub(super) fn parameters(
    first: &GeomAdaptorSurface,
    second: &GeomAdaptorSurface,
    point: Point3,
) -> Result<[f64; 4], Failure> {
    let (u1, v1) = quadric_parameters(first, point, "GeomInt_IntSS::MakeCurve")?;
    let (u2, v2) = quadric_parameters(second, point, "GeomInt_IntSS::MakeCurve")?;
    Ok([u1, v1, u2, v2])
}
pub(super) fn approximation_parameters(
    first: &GeomAdaptorSurface,
    second: &GeomAdaptorSurface,
) -> Result<[i32; 3], Failure> {
    let maximum = if matching_radii(first, second)? { 6 } else { 8 };
    let cylinders =
        first.get_type()? == SurfaceType::Cylinder && second.get_type()? == SurfaceType::Cylinder;
    Ok([4, maximum, i32::from(cylinders)])
}
pub(super) fn tangential_tolerance(
    first: &GeomAdaptorSurface,
    second: &GeomAdaptorSurface,
    tolerance: f64,
) -> Result<f64, Failure> {
    Ok(if matching_radii(first, second)? {
        0.1 * tolerance
    } else {
        tolerance
    })
}
fn matching_radii(
    first: &GeomAdaptorSurface,
    second: &GeomAdaptorSurface,
) -> Result<bool, Failure> {
    let kinds = (first.get_type()?, second.get_type()?);
    let (cylinder, torus) = match kinds {
        (SurfaceType::Cylinder, SurfaceType::Torus) => (first.cylinder()?, second.torus()?),
        (SurfaceType::Torus, SurfaceType::Cylinder) => (second.cylinder()?, first.torus()?),
        _ => return Ok(false),
    };
    Ok((cylinder.radius() - torus.minor_radius()).abs() < precision::confusion())
}
pub(super) fn approximate_with_pcurves(
    cylinder: Cylinder3,
    sphere: Sphere3,
) -> Result<bool, Failure> {
    let axis = Line3::from_axis(cylinder.axis());
    if pole_on_cylinder(axis, cylinder.radius(), sphere) {
        return Ok(false);
    }
    if cylinder.radius() < 2. * sphere.radius() {
        return Ok(true);
    }
    let distance = axis.distance(sphere.location());
    if (distance - cylinder.radius()).abs() / sphere.radius() > 0.2 {
        return Ok(true);
    }
    let parameter = elclib::parameter(&axis, sphere.location())?;
    let point = elclib::value(parameter, &axis);
    let radial = Vector3::between(point, sphere.location());
    let dot = radial.dot(Vector3::from(sphere.position().x_direction()));
    Ok(!((distance < cylinder.radius() && dot > 0.) || (distance > cylinder.radius() && dot < 0.)))
}
fn pole_on_cylinder(axis: Line3, radius: f64, sphere: Sphere3) -> bool {
    let offset = sphere.radius() * sphere.position().direction().xyz();
    let center = sphere.location().xyz();
    let upper = Point3::from(center + offset);
    let lower = Point3::from(center - offset);
    (axis.square_distance(upper) - radius * radius).abs() < 1e-7
        || (axis.square_distance(lower) - radius * radius).abs() < 1e-7
}
