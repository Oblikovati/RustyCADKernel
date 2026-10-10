//! Native object and leaf clipping, including persistent anchor-point rejection.
use super::*;
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers, select_basics::SelectBasicsSelectingVolumeManager,
};
use rustycad_tkernel::{
    ncollection::Vec4,
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::*;
pub(super) fn configure_object_clipping(
    object: &Handle<dyn SelectMgrSelectableObject>,
    global: &SelectMgrSelectingVolumeManager,
    local: &mut SelectMgrSelectingVolumeManager,
) -> Result<bool, Failure> {
    let planes = object.clip_planes();
    let persistence = object.transform_persistence();
    if planes
        .get()
        .is_some_and(Graphic3dSequenceOfHClipPlane::to_override_global)
    {
        local.set_view_clipping(&Handle::default(), &planes, Some(global))?;
    } else if !persistence.is_null() {
        if persistent_anchor_clipped(&persistence, global)? {
            return Ok(false);
        }
        local.set_view_clipping(&Handle::default(), &planes, Some(global))?;
    } else if planes.get().is_some_and(|planes| !planes.is_empty()) {
        local.set_view_clipping(global.view_clipping(), &planes, Some(global))?;
    }
    Ok(true)
}
fn persistent_anchor_clipped(
    persistence: &Handle<dyn Graphic3dTransformPersSource>,
    global: &SelectMgrSelectingVolumeManager,
) -> Result<bool, Failure> {
    if !persistence.is_zoom_or_rotate() {
        return Ok(false);
    }
    let Some(planes) = global.view_clipping().get() else {
        return Ok(false);
    };
    let anchor = persistence.anchor_point()?;
    let point = Vec4::new(anchor.x(), anchor.y(), anchor.z(), 1.);
    Ok(planes
        .items()
        .iter()
        .any(|plane| plane.is_on() && plane.probe_point(&point) == Graphic3dClipState::Out))
}
pub(super) fn selector_box_clipped(
    global: &SelectMgrSelectingVolumeManager,
    bounds: &Graphic3dBndBox3d,
    reject_partial: bool,
) -> bool {
    if global.get_active_selection_type() != SelectMgrSelectionType::Box {
        return false;
    }
    let Some(planes) = global.view_clipping().get() else {
        return false;
    };
    planes
        .items()
        .iter()
        .any(|plane| selector_plane_clips(plane, bounds, reject_partial))
}
fn selector_plane_clips(
    plane: &Graphic3dClipPlane,
    bounds: &Graphic3dBndBox3d,
    reject_partial: bool,
) -> bool {
    if !plane.is_on() {
        return false;
    }
    match plane.probe_box(bounds) {
        Graphic3dClipState::Out => true,
        Graphic3dClipState::On if reject_partial => !plane.probe_box_touch(bounds),
        _ => false,
    }
}
