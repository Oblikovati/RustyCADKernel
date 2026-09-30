// isTreatAnalityc from IntTools_FaceFace.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Nearly parallel bounded plane/cylinder intersections use the parametric intersector.
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::geom_abs::SurfaceType;
pub(super) fn treat_analytically(
    first: &dyn Adaptor3dSurface,
    second: &dyn Adaptor3dSurface,
    tolerance: f64,
) -> Result<bool, Failure> {
    let kinds = [first.get_type()?, second.get_type()?];
    let Some(plane) = selected(first, second, kinds, SurfaceType::Plane) else {
        return Ok(true);
    };
    let plane = plane.plane()?;
    let Some(cylinder) = selected(first, second, kinds, SurfaceType::Cylinder) else {
        return Ok(true);
    };
    plane_cylinder(plane, cylinder, tolerance)
}
fn selected<'a>(
    first: &'a dyn Adaptor3dSurface,
    second: &'a dyn Adaptor3dSurface,
    kinds: [SurfaceType; 2],
    kind: SurfaceType,
) -> Option<&'a dyn Adaptor3dSurface> {
    if kinds[0] == kind {
        Some(first)
    } else if kinds[1] == kind {
        Some(second)
    } else {
        None
    }
}
fn plane_cylinder(
    plane: rustycad_tkmath::gp::Plane3,
    cylinder: &dyn Adaptor3dSurface,
    tolerance: f64,
) -> Result<bool, Failure> {
    let (first, last) = (cylinder.first_v_parameter()?, cylinder.last_v_parameter()?);
    if precision::is_negative_infinite(first) || precision::is_positive_infinite(last) {
        return Ok(true);
    }
    let mut intersection = QuadQuadGeo::new();
    intersection.perform_plane_cylinder(
        plane,
        cylinder.cylinder()?,
        1e-8,
        tolerance,
        last - first,
    )?;
    if intersection.type_inter()? != ResultType::Ellipse {
        return Ok(intersection.is_done());
    }
    let ellipse = intersection.ellipse(1)?;
    Ok(ellipse.major_radius() < 100000. * ellipse.minor_radius())
}
