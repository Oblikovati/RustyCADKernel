use super::reference::{ReferenceCurve, curve_on_surface};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_adaptor;
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface},
    geom::{
        GeomCurve, GeomElementarySurface, GeomGeometry, GeomPlane, GeomRectangularTrimmedSurface,
        GeomSurface, GeomTrimmedCurve,
    },
    geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface},
};
use rustycad_tkgeombase::{geom_proj_lib, proj_lib::ProjectedCurve};
use rustycad_tkmath::toploc::Location;
pub(super) fn plane_projection(
    reference: &ReferenceCurve,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Option<Handle<Adaptor3dCurveOnSurface>>, Failure> {
    let mut plane: Handle<GeomPlane> = surface.cast();
    let trimmed: Handle<GeomRectangularTrimmedSurface> = surface.cast();
    if !trimmed.is_null() {
        plane = trimmed.basis_surface().cast();
    }
    if plane.is_null() {
        return Ok(None);
    }
    plane = plane.transformed(location.transformation())?.cast();
    let curve = projected_curve(reference, &plane)?;
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane.cast())?).cast();
    let projected = ProjectedCurve::with_curve(&surface, &curve)?;
    let pcurve = geom2d_adaptor::make_curve(&projected)?;
    Ok(Some(curve_on_surface(
        &pcurve,
        &plane.cast(),
        reference.range,
    )?))
}
fn projected_curve(
    reference: &ReferenceCurve,
    plane: &Handle<GeomPlane>,
) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let adaptor: Handle<GeomAdaptorCurve> = reference.adaptor.cast();
    let source=adaptor.get().ok_or_else(||Failure::with_type("Standard_NullObject","CorrectCurveOnSurface planar projection expected GeomAdaptor_Curve, got non-spatial reference",""))?.curve();
    let trimmed: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
        &source,
        reference.range.0,
        reference.range.1,
        true,
        true,
    )?)
    .cast();
    let curve =
        geom_proj_lib::project_on_plane(&trimmed, plane, plane.position().direction(), true)?;
    Ok(Handle::new(GeomAdaptorCurve::with_curve(&curve)?).cast())
}
