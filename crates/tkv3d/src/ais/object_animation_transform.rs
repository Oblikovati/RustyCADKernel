//! Native nonvirtual updateTrsf body, keeping callbacks outside protected-state locks.
use super::{AisBaseAnimationObjectSource, AisInteractiveContext, AisInteractiveObject};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, v3d::V3dView};
use rustycad_tkernel::standard::{Failure, Handle, exceptions::null_object};
use rustycad_tkmath::{gp::Transform3, toploc::Location};
/// Apply exact local placement and native invalidation; `ais_base_animation_object_update_transform(&animation, &transform)?`.
pub fn ais_base_animation_object_update_transform<S: AisBaseAnimationObjectSource + ?Sized>(
    animation: &S,
    transform: &Transform3,
) -> Result<(), Failure> {
    let (context, object) = animation_targets(animation);
    if context.is_null() {
        let object = object
            .get()
            .ok_or_else(|| null_object("animation object is null; expected a live AIS object"))?;
        return object.set_local_transformation(transform);
    }
    context.set_location(&object, &Location::new(*transform))?;
    invalidate_object_animation(animation)
}
fn animation_targets<S: AisBaseAnimationObjectSource + ?Sized>(
    animation: &S,
) -> (
    Handle<AisInteractiveContext>,
    Handle<dyn AisInteractiveObject>,
) {
    let fields = animation.object_animation_base().fields.read();
    (fields.context.clone(), fields.object.clone())
}
fn invalidate_object_animation<S: AisBaseAnimationObjectSource + ?Sized>(
    animation: &S,
) -> Result<(), Failure> {
    // Native invalidateViewer rereads handles after SetLocation's virtual callbacks.
    let (context, object) = animation_targets(animation);
    if context.is_null() {
        return Ok(());
    }
    let object = object.get().ok_or_else(|| {
        null_object("animation invalidation object is null; expected a live AIS object")
    })?;
    let viewer = context.current_viewer().clone();
    if !viewer.z_layer_settings(object.z_layer())?.is_immediate() {
        viewer.invalidate();
        return Ok(());
    }
    for view in viewer.defined_views() {
        invalidate_escaped_z_range(&view)?;
    }
    Ok(())
}
fn invalidate_escaped_z_range(view: &V3dView) -> Result<(), Failure> {
    let graphic_view = view.view();
    let model = graphic_view.min_max_values(false)?;
    let graphic = graphic_view.min_max_values(true)?;
    let camera = view.camera();
    let (valid, near, far) =
        camera.z_fit_all_range(view.auto_z_fit_scale_factor(), &model, &graphic)?;
    if valid && (near < camera.z_near() || far > camera.z_far()) {
        view.invalidate();
    }
    Ok(())
}
