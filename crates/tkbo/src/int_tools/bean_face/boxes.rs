//! Cached surface bounds from IntTools_BeanFaceIntersector.cxx at OCCT 3d097a0.
use crate::int_tools::SurfaceRangeLocalizeData;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::bnd::BoundingBox3;

pub(super) fn build_box(
    surface: &BRepAdaptorSurface,
    bounds: [f64; 4],
    cache: &mut SurfaceRangeLocalizeData,
    output: &mut BoundingBox3,
) -> Result<(), Failure> {
    let [first_u, last_u, first_v, last_v] = bounds;
    cache.set_frame(first_u, last_u, first_v, last_v);
    for (u, v) in [
        (first_u, first_v),
        (last_u, first_v),
        (first_u, last_v),
        (last_u, last_v),
    ] {
        output.add_point(surface.d0(u, v)?);
    }
    add_grid_rows(surface, bounds, cache, output)?;
    add_grid_sides(surface, bounds, cache, output)?;
    output.enlarge(cache.get_grid_deflection());
    Ok(())
}
fn add_grid_rows(
    surface: &BRepAdaptorSurface,
    bounds: [f64; 4],
    cache: &SurfaceRangeLocalizeData,
    output: &mut BoundingBox3,
) -> Result<(), Failure> {
    for i in 1..=cache.get_nb_u_points_in_frame() {
        let u = cache
            .get_u_param_in_frame(i)
            .map_err(|e| e.into_failure("BuildBox U parameter"))?;
        output.add_point(surface.d0(u, bounds[2])?);
        output.add_point(surface.d0(u, bounds[3])?);
        for j in 1..=cache.get_nb_v_points_in_frame() {
            output.add_point(
                cache
                    .get_point_in_frame(i, j)
                    .map_err(|e| e.into_failure("BuildBox grid point"))?,
            );
        }
    }
    Ok(())
}
fn add_grid_sides(
    surface: &BRepAdaptorSurface,
    bounds: [f64; 4],
    cache: &SurfaceRangeLocalizeData,
    output: &mut BoundingBox3,
) -> Result<(), Failure> {
    for j in 1..=cache.get_nb_v_points_in_frame() {
        let v = cache
            .get_v_param_in_frame(j)
            .map_err(|e| e.into_failure("BuildBox V parameter"))?;
        output.add_point(surface.d0(bounds[0], v)?);
        output.add_point(surface.d0(bounds[1], v)?);
    }
    Ok(())
}
pub(super) fn surface_box(
    surface: &BRepAdaptorSurface,
    bounds: [f64; 4],
    tolerance: f64,
    cache: &mut SurfaceRangeLocalizeData,
) -> Result<BoundingBox3, Failure> {
    let mut output = BoundingBox3::default();
    build_box(surface, bounds, cache, &mut output)?;
    output.enlarge(tolerance);
    Ok(output)
}
