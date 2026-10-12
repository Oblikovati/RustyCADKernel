//! ViewCube Compute preserves infinite-state mutation even for unsupported modes.
use super::AisViewCubePresentationSource;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::v3d::V3dTypeOfOrientation as Orientation;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;
/// Append the complete native ViewCube presentation without replacing application selection hooks.
/// Example: `ais_view_cube_compute(&cube,&presentation,0)?`.
/// Axes and outlines require their own aspects or a context default-drawer link;
/// missing dependencies return a typed null-object error instead of native null dereferences.
pub fn ais_view_cube_compute<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    prs.set_infinite_state(true);
    if mode != 0 {
        return Ok(());
    }
    super::view_cube_axes::submit_cube_axes(object, prs)?;
    super::view_cube_submission::submit_cube_parts(
        object,
        prs,
        &Orientation::ALL[..6],
        &object.attributes().shading_aspect().aspect(),
        object.attributes().face_boundary_draw(),
    )?;
    super::view_cube_labels::submit_cube_side_labels(object, prs)?;
    submit_cube_secondary_parts(object, prs)
}
fn submit_cube_secondary_parts<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let base = object.view_cube_presentation_base();
    super::view_cube_submission::submit_cube_parts(
        object,
        prs,
        &Orientation::ALL[6..18],
        &base.edge_aspect.read().aspect(),
        false,
    )?;
    super::view_cube_submission::submit_cube_parts(
        object,
        prs,
        &Orientation::ALL[18..],
        &base.corner_aspect.read().aspect(),
        false,
    )
}
