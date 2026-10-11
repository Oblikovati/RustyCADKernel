//! Object animation fixtures shared by capture, retained-stream and performance checks.
use crate::ais_scene::AisViewerScene;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOP;
use rustycad_tkv3d::{ais::*, prs_mgr::PrsMgrPresentableObject};
pub const MODES: [&str; 10] = [
    "animobj-start",
    "animobj-mid",
    "animobj-end",
    "animobj-instant",
    "animobj-nested",
    "animobj-paused",
    "animobj-detached",
    "animobj-erased",
    "animobj-immediate",
    "animobj-escaped",
];
/// Build exact local endpoints around the input translation; `animation(&scene, 0.)`.
pub fn animation(scene: &AisViewerScene, offset: f64) -> Handle<AisAnimationObject> {
    let start = Transform3::translation_vector(Vector3::new(offset, 0., 0.));
    let mut end = Transform3::translation_vector(Vector3::new(offset + 4., 0., 0.));
    end.set_scale_factor(0.5).unwrap();
    let animation = Handle::new(AisAnimationObject::new(
        "object",
        &scene.context,
        &scene.object.cast(),
        start,
        end,
    ));
    animation.set_own_duration(2.);
    animation
}
/// Apply a native capture mode without redrawing; `apply(&scene, "animobj-mid", 0.)`.
pub fn apply(scene: &AisViewerScene, mode: &str, offset: f64) {
    let animation = animation(scene, offset);
    configure(scene, &animation, mode);
    animation.start(false).unwrap();
    if mode == "animobj-paused" {
        animation.pause().unwrap();
    }
    if mode == "animobj-nested" {
        return nested(&animation);
    }
    let pts = match mode {
        "animobj-start" | "animobj-instant" => 0.,
        "animobj-end" => 2.,
        _ => 1.,
    };
    animation.update(pts).unwrap();
}
fn configure(scene: &AisViewerScene, animation: &AisAnimationObject, mode: &str) {
    if mode == "animobj-instant" {
        animation.set_own_duration(0.);
    }
    if mode == "animobj-detached" {
        animation.object_animation_base().fields.write().context = Handle::default();
    }
    if mode == "animobj-erased" {
        scene.context.erase(&scene.object.cast(), false).unwrap();
    }
    if mode == "animobj-immediate" || mode == "animobj-escaped" {
        scene.object.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP);
    }
    if mode == "animobj-escaped" {
        scene.view.camera().set_z_range(1., 5.).unwrap();
    }
}
fn nested(animation: &Handle<AisAnimationObject>) {
    let parent = AisAnimation::new("parent");
    parent.set_start_pts(0.5);
    animation.set_start_pts(0.5);
    parent.add(&animation.cast()).unwrap();
    parent.start(false).unwrap();
    parent.update(2.).unwrap();
}
