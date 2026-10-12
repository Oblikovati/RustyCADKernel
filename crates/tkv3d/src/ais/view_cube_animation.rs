//! Real timer-backed click animation with native detach, redraw and callback ordering.
use super::{
    AisAnimationCameraMembers, AisAnimationCameraSource, AisAnimationMembers, AisViewCubeMembers,
    AisViewCubeOwner, AisViewCubeSource,
};
use crate::{ais::AisInteractiveObjectMembers, v3d::V3dView};
use rustycad_tkernel::standard::{Failure, Handle, catch_native_failure, exceptions::null_object};
/// Start the picked orientation on the context's last active view; `ais_view_cube_start_animation(&cube,&owner)?`.
pub fn ais_view_cube_start_animation<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    owner: &Handle<AisViewCubeOwner>,
) -> Result<(), Failure> {
    catch_native_failure(|| cube_start_animation(cube, owner))
}
fn cube_start_animation<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    owner: &Handle<AisViewCubeOwner>,
) -> Result<(), Failure> {
    let context = cube.get_context();
    let context = context.get().ok_or_else(|| {
        null_object("AIS_ViewCube context=null; expected an attached interactive context")
    })?;
    let view = context.last_active_view();
    let (Some(owner), Some(view_ref)) = (owner.get(), view.get()) else {
        return Ok(());
    };
    super::view_cube_camera_fit::cube_oriented_end_camera(
        cube,
        view_ref,
        owner.main_orientation(),
    )?;
    prepare_cube_animation(cube, &view)?;
    Ok(())
}
fn prepare_cube_animation<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    view: &Handle<V3dView>,
) -> Result<(), Failure> {
    with_cube_animation(cube, |animation| {
        animation.set_view(view);
        let fields = cube.interaction_base().fields.read();
        animation.set_camera_start(&fields.start);
        animation.set_camera_end(&fields.end);
        Ok(())
    })?;
    with_cube_animation(cube, |animation| {
        animation.start_timer(0.0, 1.0, true, false)
    })
}
/// Advance a native timer frame and optionally redraw; `ais_view_cube_update_animation(&cube,false)?`.
pub fn ais_view_cube_update_animation<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    redraw: bool,
) -> Result<bool, Failure> {
    catch_native_failure(|| cube_update_animation(cube, redraw))
}
fn cube_update_animation<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    redraw: bool,
) -> Result<bool, Failure> {
    let view = with_cube_animation(cube, |animation| Ok(animation.target_view()))?;
    if !cube.has_animation()? || !ais_view_cube_update_animation_step(cube)? {
        return Ok(false);
    }
    if redraw && let Some(view) = view.get() {
        if view.is_invalidated() {
            view.redraw();
        } else {
            view.redraw_immediate();
        }
    }
    cube.on_after_animation();
    Ok(true)
}
/// Perform the native completion transition; `ais_view_cube_update_animation_step(&cube)?`.
pub fn ais_view_cube_update_animation_step<C: AisViewCubeSource + ?Sized>(
    cube: &C,
) -> Result<bool, Failure> {
    catch_native_failure(|| cube_update_animation_step(cube))
}
fn cube_update_animation_step<C: AisViewCubeSource + ?Sized>(cube: &C) -> Result<bool, Failure> {
    let pts = with_cube_animation(cube, |animation| animation.update_timer())?;
    let duration = with_cube_animation(cube, |animation| Ok(animation.own_duration()))?;
    if pts >= duration {
        with_cube_animation(cube, |animation| animation.stop())?;
        cube.on_animation_finished();
        with_cube_animation(cube, |animation| {
            animation.set_view(&Handle::default());
            Ok(())
        })?;
        return Ok(false);
    }
    Ok(true)
}
/// Honor disabled auto-start and non-fixed click policies; `ais_view_cube_handle_click(&cube,&owner)?`.
pub fn ais_view_cube_handle_click<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    owner: &Handle<AisViewCubeOwner>,
) -> Result<(), Failure> {
    catch_native_failure(|| cube_handle_click(cube, owner))
}
fn cube_handle_click<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    owner: &Handle<AisViewCubeOwner>,
) -> Result<(), Failure> {
    if !cube.interaction_base().fields.read().auto_start {
        return Ok(());
    }
    cube.start_animation(owner)?;
    if !cube.interaction_base().fields.read().fixed_loop {
        return Ok(());
    }
    while cube.has_animation()? {
        cube.update_animation(true)?;
    }
    Ok(())
}
pub(super) fn with_cube_animation<C: AisViewCubeSource + ?Sized, R>(
    cube: &C,
    action: impl FnOnce(&dyn AisAnimationCameraSource) -> Result<R, Failure>,
) -> Result<R, Failure> {
    // Native rereads myViewAnimation between virtual calls, whose overrides can replace it.
    cube.view_animation_ref().with_animation(|animation| {
        let animation = animation.ok_or_else(|| {
            null_object("AIS_ViewCube animation=null; expected a non-null camera animation")
        })?;
        action(animation)
    })
}
