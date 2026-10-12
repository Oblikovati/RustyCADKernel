//! Real driver-view and camera anchors reproduce the pinned native callback lifetime.
use super::view_camera_alias_fixture::{
    BorrowedCameraView, camera_alias_driver, fresh_camera_alias_control,
};
use super::view_clip_retarget_probe::{ViewClipRetargetProbe, new_owned_clip_probe_slot};
use super::{V3dView, V3dViewer};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use std::sync::Arc;
pub(super) struct ClipRetargetActors {
    pub owner: Handle<dyn super::V3dViewSource>,
    pub old: Handle<BorrowedCameraView>,
    pub next: Handle<BorrowedCameraView>,
    pub plane: Handle<Graphic3dClipPlane>,
    pub probe: Arc<ViewClipRetargetProbe>,
}
pub(super) fn clip_retarget_actors() -> Result<ClipRetargetActors, Failure> {
    let control = fresh_camera_alias_control();
    let probe = Arc::new(ViewClipRetargetProbe::default());
    *control.clip_probe.write() = Some(probe.clone());
    let driver = camera_alias_driver(&NullGraphicDriver::new_handle(), &control);
    let viewer = V3dViewer::new(&driver)?;
    let owner = V3dView::new(&viewer, super::V3dTypeOfView::Orthographic)?;
    let old = owner.view_ref().observe().cast::<BorrowedCameraView>();
    let next = create_next_clip_view(&viewer, &control)?;
    *probe.owner.write() = owner.downgrade();
    *probe.replacement.write() = next.cast();
    Ok(ClipRetargetActors {
        owner,
        old,
        next,
        plane: Handle::new(Graphic3dClipPlane::default()),
        probe,
    })
}
fn create_next_clip_view(
    viewer: &V3dViewer,
    control: &Arc<super::view_camera_alias_fixture::CameraAliasControl>,
) -> Result<Handle<BorrowedCameraView>, Failure> {
    let retained = Graphic3dRetainedView::new(&viewer.structure_manager(), &[])?;
    Ok(Handle::new(BorrowedCameraView {
        transient: Default::default(),
        identity: 2,
        retained,
        clip_probe_slot: Some(new_owned_clip_probe_slot()),
        control: control.clone(),
    }))
}
