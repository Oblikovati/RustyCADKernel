//! Native inherited clipping retains the intentional local sequence owner, never a view snapshot.
use super::V3dView;
use rustycad_tkernel::standard::{Handle, HandleReference};
use rustycad_tkservice::graphic3d::{Graphic3dClipPlaneSource, Graphic3dSequenceOfHClipPlane};
/// Execute native insertion with a live caller plane field; `v3d_view_add_clip_plane(base,&plane)`.
pub fn v3d_view_add_clip_plane(
    view: &V3dView,
    plane: &dyn HandleReference<dyn Graphic3dClipPlaneSource>,
) {
    let mut planes = view.clip_planes_ref().clone_handle();
    if planes.is_null() {
        planes = Handle::new(Graphic3dSequenceOfHClipPlane::default()).into_cast();
    } else if planes.contains_reference(plane) {
        return;
    }
    planes.append_reference(plane);
    // Allocate/destructor callbacks may retarget protected myView: reread it for SetClipPlanes.
    view.set_clip_planes(&planes);
}
/// Execute native removal with its intentional local sequence owner; `v3d_view_remove_clip_plane(base,&plane)`.
pub fn v3d_view_remove_clip_plane(
    view: &V3dView,
    plane: &dyn HandleReference<dyn Graphic3dClipPlaneSource>,
) {
    let planes = view.clip_planes_ref().clone_handle();
    if planes
        .get()
        .is_some_and(|sequence| sequence.remove_reference(plane))
    {
        // Native setter receives the still-owning local sequence after node callbacks complete.
        view.set_clip_planes(&planes);
    }
}
