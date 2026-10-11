//! Actual native anchor presentations are owned from construction, not synthesized at dispatch.
use super::AisPoint;
use crate::prs_mgr::PrsMgrPresentableObject;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkservice::graphic3d::{GRAPHIC3D_ZLAYER_ID_TOP, GRAPHIC3D_ZLAYER_ID_TOPMOST};
pub(super) struct ViewControllerCameraActionState {
    pub(super) had_computed_hlr: bool,
    pub(super) rotate_anchor: Handle<AisPoint>,
    pub(super) pan_anchor: Handle<AisPoint>,
}
impl ViewControllerCameraActionState {
    pub(super) fn new() -> Result<Self, Failure> {
        Ok(Self {
            had_computed_hlr: false,
            rotate_anchor: camera_action_anchor(GRAPHIC3D_ZLAYER_ID_TOP)?,
            pan_anchor: camera_action_anchor(GRAPHIC3D_ZLAYER_ID_TOPMOST)?,
        })
    }
}
fn camera_action_anchor(layer: i32) -> Result<Handle<AisPoint>, Failure> {
    let geometry = Handle::new(GeomCartesianPoint::new(0.0, 0.0, 0.0));
    let anchor = AisPoint::new(&geometry.cast())?;
    anchor.set_z_layer(layer);
    anchor.set_mutable(true);
    Ok(anchor)
}
