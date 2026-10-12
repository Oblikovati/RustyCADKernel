//! Native inherited clip-plane operations retain exactly the list's owning reference.
use super::V3dView;
use rustycad_tkernel::standard::{Handle, HandleReference};
use rustycad_tkservice::graphic3d::{Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane};

/// Execute the real inherited plane insertion; `v3d_view_add_clip_plane(base,&plane)`.
pub fn v3d_view_add_clip_plane(view: &V3dView, plane: &dyn HandleReference<Graphic3dClipPlane>) {
    let mut planes = view.clip_planes();
    if planes.is_null() {
        planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    } else if planes
        .get()
        .is_some_and(|p| p.items().contains(&plane.observe()))
    {
        return;
    }
    if let Some(sequence) = planes.get() {
        sequence.append(plane.clone_handle());
    }
    view.set_clip_planes(&planes);
}
/// Execute the real inherited plane removal; `v3d_view_remove_clip_plane(base,&plane)`.
pub fn v3d_view_remove_clip_plane(view: &V3dView, plane: &dyn HandleReference<Graphic3dClipPlane>) {
    let planes = view.clip_planes();
    if planes.get().is_some_and(|p| p.remove(&plane.observe())) {
        view.set_clip_planes(&planes);
    }
}
