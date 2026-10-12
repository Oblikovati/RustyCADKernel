//! Actual camera interpolation rereads the retained view after valid callback retargeting.
use super::{AisAnimationCameraMembers, AisAnimationCameraSource};
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
pub(super) fn update_view_camera<A: AisAnimationCameraSource + ?Sized>(
    animation: &A,
    start: &Graphic3dCamera,
    end: &Graphic3dCamera,
    fraction: f64,
) -> Result<(), Failure> {
    let target = animation.target_view_ref();
    let camera = target.with_object(|view| view.unwrap().camera());
    Graphic3dCamera::interpolate(start, end, fraction, &camera)?;
    // Native myView is read again after SetCamera can invoke application presentation callbacks.
    let immediate = target.with_object(|view| view.unwrap().set_immediate_update(false));
    target.with_object(|view| view.unwrap().set_camera(&camera))?;
    target.with_object(|view| view.unwrap().set_immediate_update(immediate));
    target.with_object(|view| view.unwrap().invalidate());
    Ok(())
}
