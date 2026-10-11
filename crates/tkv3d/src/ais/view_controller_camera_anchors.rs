//! Real AIS_Point anchors retain native context identity, placement and layer redraw policy.
use super::{AisInteractiveContext, AisInteractiveObjectMembers, AisPoint, AisViewController};
use crate::{
    prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers},
    v3d::V3dView,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
impl AisViewController {
    pub(super) fn remove_camera_action_anchor(
        &self,
        context: &AisInteractiveContext,
        view: &V3dView,
        anchor: &Handle<AisPoint>,
    ) -> Result<(), Failure> {
        if !anchor.has_interactive_context() {
            return Ok(());
        }
        context.remove(&anchor.cast(), false)?;
        invalidate_camera_anchor_layer(view, anchor.z_layer())
    }
    pub(super) fn display_camera_action_anchor(
        &self,
        context: &AisInteractiveContext,
        anchor: &Handle<AisPoint>,
    ) -> Result<(), Failure> {
        context.display_with_modes(&anchor.cast(), 0, -1, false, PrsMgrDisplayStatus::Displayed)
    }
    pub(super) fn locate_camera_action_anchor(
        &self,
        context: &AisInteractiveContext,
        anchor: &Handle<AisPoint>,
        point: Point3,
    ) -> Result<(), Failure> {
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(point.x(), point.y(), point.z()));
        context.set_location(&anchor.cast(), &Location::new(transform))
    }
}

fn invalidate_camera_anchor_layer(view: &V3dView, layer: i32) -> Result<(), Failure> {
    if view.viewer().z_layer_settings(layer)?.is_immediate() {
        view.invalidate_immediate();
    } else {
        view.invalidate();
    }
    Ok(())
}
