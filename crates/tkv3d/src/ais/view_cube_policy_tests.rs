//! Boundary policies preserve native retained state and diagnose native null dereferences.
use super::{view_cube_interaction_fixture::*, *};
use crate::{prs_mgr::PrsMgrPresentableObject, select_mgr::*, select3d::*, v3d::*};
use rustycad_tkernel::standard::{Handle, Transient};
#[test]
fn cube_animation_policy_replacement_duration_and_null_errors() {
    let cube = AisViewCube::new().unwrap();
    let animation = Handle::new(AisAnimationCamera::new("replacement", &Handle::default()));
    cube.set_view_animation(&animation.cast());
    assert_eq!(
        cube.view_animation(),
        animation.cast::<dyn AisAnimationCameraSource>()
    );
    cube.set_duration(-0.75).unwrap();
    assert_eq!(cube.duration().unwrap(), -0.75);
    cube.set_duration(f64::NAN).unwrap();
    assert!(cube.duration().unwrap().is_nan());
    cube.set_view_animation(&Handle::default());
    assert!(cube.view_animation().is_null());
    assert!(cube.duration().is_err() && cube.set_duration(0.5).is_err());
    assert!(cube.has_animation().is_err() && cube.update_animation(false).is_err());
}
#[test]
fn cube_empty_click_preconditions_and_detached_scene_fit_are_explicit() {
    let scene = WidgetScene::new(false);
    scene.cube.start_animation(&Handle::default()).unwrap();
    assert!(!scene.cube.has_animation().unwrap());
    scene.context.fields.write().last_active_view = Default::default();
    let owner = AisViewCubeOwner::new(&scene.cube, V3dTypeOfOrientation::Xpos, 5);
    scene.cube.start_animation(&owner).unwrap();
    assert!(!scene.cube.has_animation().unwrap());
    assert_detached_cube_scene_fit(&scene, &owner);
}
fn assert_detached_cube_scene_fit(scene: &WidgetScene, owner: &Handle<AisViewCubeOwner>) {
    let detached = AisViewCube::new().unwrap();
    assert!(detached.start_animation(owner).is_err());
    detached.set_fit_selected(false);
    detached
        .view_fit_all(&scene.view, &scene.view.camera())
        .unwrap();
    detached.set_fit_selected(true);
    assert!(
        detached
            .view_fit_all(&scene.view, &scene.view.camera())
            .is_err()
    );
}
#[test]
fn cube_sensitive_requires_triangles_and_preserves_native_connected_base_type() {
    let scene = WidgetScene::new(false);
    let owner = AisViewCubeOwner::new(&scene.cube, V3dTypeOfOrientation::Xpos, 5);
    assert!(AisViewCubeSensitive::new(&owner.cast(), &Handle::default()).is_err());
    let selection = Handle::new(SelectMgrSelection::new(0));
    scene.cube.compute_selection(&selection, 0).unwrap();
    let entity = selection.entities()[0].base_sensitive();
    let connected = entity.get_connected().unwrap();
    assert!(connected.cast::<AisViewCubeSensitive>().is_null());
    assert!(
        !connected
            .cast::<Select3DSensitivePrimitiveArray>()
            .is_null()
    );
    assert_eq!(connected.owner_id(), entity.owner_id());
}
#[test]
fn cube_callback_absence_retains_real_camera_progress_without_observation() {
    let scene = WidgetScene::new(false);
    scene.cube.set_animation_events(None);
    scene.cube.set_fixed_animation_loop(false);
    let owner = AisViewCubeOwner::new(&scene.cube, V3dTypeOfOrientation::Xpos, 5);
    scene.cube.handle_click(&owner).unwrap();
    scene.clock.set(0.25);
    assert!(scene.cube.update_animation(false).unwrap());
    scene.clock.set(0.5);
    assert!(!scene.cube.update_animation(false).unwrap());
    assert!(!scene.cube.has_animation().unwrap());
    assert!(scene.cube.view_animation().target_view().is_null());
    assert_eq!(
        scene
            .events
            .frames
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}
#[test]
fn cube_owner_sensitive_runtime_descriptors_preserve_native_names_and_parents() {
    let scene = WidgetScene::new(false);
    let owner = AisViewCubeOwner::new(&scene.cube, V3dTypeOfOrientation::Xpos, 5);
    let selection = Handle::new(SelectMgrSelection::new(0));
    scene.cube.compute_selection(&selection, 0).unwrap();
    let sensitive = selection.entities()[0]
        .base_sensitive()
        .cast::<AisViewCubeSensitive>();
    assert_widget_cube_descriptor(&scene.cube);
    assert_widget_descriptor(
        owner.dynamic_type().unwrap(),
        "AIS_ViewCubeOwner",
        "SelectMgr_EntityOwner",
    );
    assert_widget_descriptor(
        sensitive.dynamic_type().unwrap(),
        "AIS_ViewCubeSensitive",
        "Select3D_SensitivePrimitiveArray",
    );
}
fn assert_widget_descriptor(
    descriptor: &rustycad_tkernel::standard::RuntimeType,
    name: &str,
    parent: &str,
) {
    assert_eq!(descriptor.name(), name);
    assert_eq!(descriptor.parent().unwrap().name(), parent);
}
#[test]
fn cube_context_display_activates_actual_part_selection_and_presentation() {
    let scene = WidgetScene::new(false);
    scene.context.display(&scene.cube.cast(), false).unwrap();
    assert_eq!(scene.cube.selection(0).entities().len(), 26);
    assert!(
        !scene
            .cube
            .presentable_base()
            .fields
            .read()
            .presentations
            .is_empty()
    );
    assert_eq!(scene.cube.get_context(), scene.context);
    scene.cube.set_fit_selected(false);
    scene.cube.set_reset_camera(true);
    assert!(!scene.cube.to_fit_selected() && scene.cube.to_reset_camera_up());
}

fn assert_widget_cube_descriptor(cube: &AisViewCube) {
    assert_widget_descriptor(cube.dynamic_type(), "AIS_ViewCube", "AIS_InteractiveObject");
}
