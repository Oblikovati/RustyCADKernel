//! Exact local interpolation, native inheritance, null guards and base update policy.
#[path = "support/context_object.rs"]
mod context_object;
use context_object::ContextObject;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::*};
fn translation(x: f64) -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(x, 0., 0.));
    transform
}
fn atom(object: &Handle<ContextObject>) -> Handle<AisAnimationObject> {
    let animation = Handle::new(AisAnimationObject::new(
        "move",
        &Handle::default(),
        &object.cast(),
        translation(2.),
        translation(6.),
    ));
    animation.set_own_duration(2.);
    animation
}
#[test]
fn exact_local_endpoints_replace_existing_placement() {
    let object = ContextObject::new();
    object.set_local_transformation(&translation(100.)).unwrap();
    let animation = atom(&object);
    animation.start(false).unwrap();
    for (pts, x) in [(0., 2.), (1., 4.), (2., 6.)] {
        animation.update(pts).unwrap();
        assert_eq!(object.local_transformation().translation_part().x(), x);
    }
    assert!(animation.is_stopped());
}
#[test]
fn zero_and_negative_own_duration_apply_start_placement() {
    let object = ContextObject::new();
    let animation = atom(&object);
    for duration in [0., -1.] {
        animation.set_own_duration(duration);
        animation.start(false).unwrap();
        animation.update(0.).unwrap();
        assert_eq!(object.local_transformation().translation_part().x(), 2.);
    }
}
#[test]
fn nested_progress_and_paused_atoms_preserve_native_timeline() {
    let object = ContextObject::new();
    let animation = atom(&object);
    let parent = AisAnimation::new("parent");
    parent.set_start_pts(0.5);
    animation.set_start_pts(0.5);
    parent.add(&animation.cast()).unwrap();
    parent.start(false).unwrap();
    parent.update(2.).unwrap();
    assert_eq!(object.local_transformation().translation_part().x(), 4.);
    animation.pause().unwrap();
    animation.update(2.5).unwrap();
    assert_eq!(object.local_transformation().translation_part().x(), 4.);
}
#[test]
fn native_base_and_child_rtti_allow_interface_round_trips() {
    let object = ContextObject::new();
    let animation = atom(&object);
    for name in [
        "AIS_AnimationObject",
        "AIS_BaseAnimationObject",
        "AIS_Animation",
    ] {
        assert!(animation.is_kind_name(name));
    }
    let base = animation.cast::<dyn AisBaseAnimationObjectSource>();
    assert_eq!(base.cast::<AisAnimationObject>(), animation);
    assert_eq!(
        base.object_animation_base().fields.read().object,
        object.cast::<dyn AisInteractiveObject>()
    );
}
#[test]
fn null_object_skips_interpolation_even_when_scale_crosses_zero() {
    let mut end = Transform3::default();
    end.set_scale_factor(-1.).unwrap();
    let animation = AisAnimationObject::new(
        "null",
        &Handle::default(),
        &Handle::default(),
        Transform3::default(),
        end,
    );
    animation.set_own_duration(2.);
    animation.start(false).unwrap();
    assert!(animation.update(1.).unwrap());
}
#[test]
fn interpolation_failure_leaves_placement_unchanged() {
    let object = ContextObject::new();
    let mut end = Transform3::default();
    end.set_scale_factor(-1.).unwrap();
    let animation = AisAnimationObject::new(
        "zero scale",
        &Handle::default(),
        &object.cast(),
        Transform3::default(),
        end,
    );
    object.set_local_transformation(&translation(9.)).unwrap();
    animation.set_own_duration(2.);
    animation.start(false).unwrap();
    assert!(animation.update(1.).is_err());
    assert_eq!(object.local_transformation().translation_part().x(), 9.);
}
#[test]
fn base_hook_is_noop_but_nonvirtual_transform_body_is_available() {
    let object = ContextObject::new();
    let base = AisBaseAnimationObject::new("base", &Handle::default(), &object.cast());
    base.start(true).unwrap();
    assert!(object.local_transformation_geom().is_null());
    ais_base_animation_object_update_transform(&base, &translation(3.)).unwrap();
    assert_eq!(object.local_transformation().translation_part().x(), 3.);
    base.object_animation_base().fields.write().object = Handle::default();
    assert!(ais_base_animation_object_update_transform(&base, &translation(3.)).is_err());
}
#[test]
fn context_animation_applies_active_selection_updates() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let object = ContextObject::new();
    context.display(&object.cast(), false).unwrap();
    let animation = atom(&object);
    animation.object_animation_base().fields.write().context = context.clone();
    animation.start(false).unwrap();
    animation.update(1.).unwrap();
    assert_eq!(
        context
            .location(&object.cast())
            .unwrap()
            .transformation()
            .translation_part()
            .x(),
        4.
    );
    assert_eq!(
        object.selection(0).update_status(),
        SelectMgrTypeOfUpdate::None
    );
}
