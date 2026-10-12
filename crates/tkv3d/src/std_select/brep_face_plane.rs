//! Native plane fallback clips only infinite parameter ends and suppresses interior for the complete clipped rectangle.
use super::brep_face_sensitive::FaceSelectionCriteria;
use crate::{select_mgr::SelectMgrEntityOwnerSource, select3d::*};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
type PlaneParameterRanges = ((f64, f64), (f64, f64));
pub(super) fn brep_face_plane_sensitive(
    surface: &BRepAdaptorSurface,
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    criteria: &FaceSelectionCriteria,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let max = criteria.max_parameter;
    let (u, v) = plane_selection_ranges(surface, max)?;
    let points = plane_selection_corners(surface, u, v)?;
    let infinite = u == (-max, max) && v == (-max, max);
    let kind = plane_selection_kind(criteria.interior && !infinite);
    Ok(Handle::new(Select3DSensitiveFace::new(&owner.observe(), &points, kind)?).cast())
}
fn plane_selection_ranges(
    surface: &BRepAdaptorSurface,
    max: f64,
) -> Result<PlaneParameterRanges, Failure> {
    let u = (
        clip_plane_first(surface.first_u_parameter()?, max),
        clip_plane_last(surface.last_u_parameter()?, max),
    );
    let v = (
        clip_plane_first(surface.first_v_parameter()?, max),
        clip_plane_last(surface.last_v_parameter()?, max),
    );
    Ok((u, v))
}
fn plane_selection_corners(
    surface: &BRepAdaptorSurface,
    u: (f64, f64),
    v: (f64, f64),
) -> Result<Vec<rustycad_tkmath::gp::Point3>, Failure> {
    let mut points = vec![
        surface.value(u.0, v.0)?,
        surface.value(u.1, v.0)?,
        surface.value(u.1, v.1)?,
        surface.value(u.0, v.1)?,
    ];
    points.push(points[0]);
    Ok(points)
}
fn plane_selection_kind(interior: bool) -> Select3DTypeOfSensitivity {
    if interior {
        return Select3DTypeOfSensitivity::Interior;
    }
    Select3DTypeOfSensitivity::Boundary
}

fn clip_plane_first(parameter: f64, max: f64) -> f64 {
    if parameter <= -precision::infinite() {
        return -max;
    }
    parameter
}
fn clip_plane_last(parameter: f64, max: f64) -> f64 {
    if parameter >= precision::infinite() {
        return max;
    }
    parameter
}
