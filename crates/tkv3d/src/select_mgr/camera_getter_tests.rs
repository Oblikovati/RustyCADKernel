//! The native selected camera reference remains bound to the original source after retargeting.
use super::{camera_getter_fixture::*, camera_getter_rows::*, *};
use rustycad_tkernel::standard::{Failure, Handle, HandleReference, catch_native_failure};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[test]
fn native_three_camera_getters_alias_counts_null_retarget_failure_and_retry() {
    let mut rows = builder_getter_rows();
    rows.extend(intersector_getter_rows(1));
    rows.extend(intersector_getter_rows(2));
    rows.extend(
        (0..2)
            .flat_map(|empty| (0..3).flat_map(move |fault| camera_getter_fault_rows(empty, fault))),
    );
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/selectmgr-camera-getters-native.txt")
    );
}
fn camera_getter_fault_rows(empty: i32, fault: i32) -> Vec<String> {
    let mut scene = getter_volume_scene();
    scene.original.set_camera(&scene.first);
    if empty != 0 {
        scene.manager.init_selecting_volume(&Handle::default());
    }
    let alias = scene.manager.camera_ref();
    scene.manager.init_selecting_volume(&scene.next.cast());
    *scene.original.getter_probe.write() = Some(CameraGetterSetterProbe::new(&scene.second, fault));
    (0..2)
        .map(|frame| {
            let failure = apply_camera_getter_fault(&scene, &alias);
            volume_getter_row([3 + empty, 2 * fault + frame, failure], 2, &alias, &scene)
        })
        .collect()
}
fn apply_camera_getter_fault(
    scene: &GetterVolumeScene,
    alias: &dyn HandleReference<Graphic3dCamera>,
) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        catch_native_failure(|| {
            scene.original.set_camera(alias);
            Ok::<(), Failure>(())
        })
    }));
    classify_camera_getter_fault(result)
}
fn classify_camera_getter_fault(
    result: Result<Result<(), Failure>, Box<dyn std::any::Any + Send>>,
) -> i32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(failure)) => {
            assert!(failure.message().contains("fault=1"));
            1
        }
        Err(payload) => {
            assert_eq!(payload.downcast_ref::<u32>(), Some(&107));
            2
        }
    }
}

#[test]
fn builder_alias_retains_no_owner_and_expiration_is_safe_outside_native_lifetime() {
    let camera = Handle::new(Graphic3dCamera::default());
    let builder = Handle::new(SelectMgrFrustumBuilder::new());
    builder.set_camera(&camera);
    let alias = builder.camera_ref();
    assert_eq!((builder.ref_count(), camera.ref_count()), (1, 2));
    let snapshot = alias.clone_handle();
    assert_eq!(camera.ref_count(), 3);
    drop(snapshot);
    drop(builder);
    assert_eq!(camera.ref_count(), 1);
    assert!(alias.is_null());
    assert!(alias.reference_address().is_null());
}
#[test]
fn intersector_alias_retains_no_owner_or_lock_through_camera_reset_and_expiration() {
    let camera = Handle::new(Graphic3dCamera::default());
    let volume = getter_test_volume();
    volume.set_camera(&camera);
    let alias = volume.camera_ref();
    assert_eq!((volume.ref_count(), camera.ref_count()), (1, 3));
    volume.set_camera(&Handle::default());
    assert!(alias.is_null());
    volume.set_camera(&camera);
    drop(volume);
    assert_eq!(camera.ref_count(), 1);
    assert!(alias.is_null());
    assert!(alias.reference_address().is_null());
}
#[test]
fn manager_static_null_alias_stays_original_after_manager_replacement_and_drop() {
    let mut manager = SelectMgrSelectingVolumeManager::new();
    let empty = manager.camera_ref();
    let other = SelectMgrSelectingVolumeManager::new().camera_ref();
    assert_eq!(empty.reference_address(), other.reference_address());
    let volume = getter_test_volume();
    manager.init_selecting_volume(&volume.cast());
    assert_ne!(
        empty.reference_address(),
        manager.camera_ref().reference_address()
    );
    drop(manager);
    assert!(empty.is_null());
    assert!(!empty.reference_address().is_null());
    assert_eq!(volume.ref_count(), 1);
}

#[test]
fn cloned_camera_field_alias_keeps_original_receiver_without_target_ownership() {
    let mut scene = getter_volume_scene();
    scene.original.set_camera(&scene.first);
    let first = scene.manager.camera_ref();
    let retained = first.clone();
    assert_eq!(first.reference_address(), retained.reference_address());
    assert_eq!(
        (scene.original.ref_count(), scene.first.ref_count()),
        (2, 3)
    );
    scene.manager.init_selecting_volume(&scene.next.cast());
    assert_eq!(retained.object_address(), scene.first.object_address());
    scene.original.set_camera(&Handle::default());
    assert!(retained.is_null());
    assert_eq!(scene.original.ref_count(), 1);
}
