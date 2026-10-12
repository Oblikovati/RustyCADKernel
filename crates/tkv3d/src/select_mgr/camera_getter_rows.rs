//! Retained builder/intersector/manager aliases preserve actual field selection and native counts.
use super::{camera_argument_fixture::CameraArgumentFrustum, camera_getter_fixture::*, *};
use rustycad_tkernel::standard::{Handle, HandleReference};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
pub(super) struct GetterVolumeScene {
    pub first: Handle<Graphic3dCamera>,
    pub second: Handle<Graphic3dCamera>,
    pub original: Handle<CameraArgumentFrustum>,
    pub next: Handle<CameraArgumentFrustum>,
    pub manager: SelectMgrSelectingVolumeManager,
}
pub(super) fn getter_volume_scene() -> GetterVolumeScene {
    let first = Handle::new(Graphic3dCamera::default());
    let second = Handle::new(Graphic3dCamera::default());
    let original = getter_test_volume();
    let next = getter_test_volume();
    next.set_camera(&second);
    let mut manager = SelectMgrSelectingVolumeManager::new();
    manager.init_selecting_volume(&original.cast());
    GetterVolumeScene {
        first,
        second,
        original,
        next,
        manager,
    }
}
pub(super) fn intersector_getter_rows(kind: i32) -> Vec<String> {
    let mut scene = getter_volume_scene();
    let alias = if kind == 1 {
        scene.original.camera_ref()
    } else {
        scene.manager.camera_ref()
    };
    (0..6)
        .map(|phase| {
            change_getter_volume(kind, phase, &mut scene);
            volume_getter_row([kind, phase, 0], kind, &alias, &scene)
        })
        .collect()
}
fn change_getter_volume(kind: i32, phase: i32, scene: &mut GetterVolumeScene) {
    change_original_getter_camera(kind, phase, scene);
    if kind != 2 {
        return;
    }
    match phase {
        2 => scene.manager.init_selecting_volume(&scene.next.cast()),
        4 => scene.manager.init_selecting_volume(&Handle::default()),
        5 => {
            scene.manager.init_selecting_volume(&scene.original.cast());
            scene.original.set_camera(&scene.second);
        }
        _ => {}
    }
}
fn change_original_getter_camera(kind: i32, phase: i32, scene: &GetterVolumeScene) {
    match phase {
        1 | 4 => scene.original.set_camera(&scene.first),
        3 => scene.original.set_camera(&Handle::default()),
        2 | 5 if kind == 1 => scene.original.set_camera(&scene.second),
        _ => {}
    }
}
pub(super) fn volume_getter_row(
    labels: [i32; 3],
    kind: i32,
    alias: &dyn HandleReference<Graphic3dCamera>,
    scene: &GetterVolumeScene,
) -> String {
    let fresh = selected_volume_camera_reference(kind, scene);
    let original = scene.original.camera_ref();
    let empty = SelectMgrSelectingVolumeManager::new().camera_ref();
    let fields = CameraGetterFields {
        alias,
        fresh: &fresh,
        original: &original,
        empty: &empty,
    };
    camera_getter_row(labels, fields, volume_getter_counts(scene))
}
fn volume_getter_counts(scene: &GetterVolumeScene) -> CameraGetterCounts<'_> {
    CameraGetterCounts {
        first: &scene.first,
        second: &scene.second,
        owner_counts: [scene.original.ref_count(), scene.next.ref_count()],
    }
}
struct BuilderGetterScene {
    first: Handle<Graphic3dCamera>,
    second: Handle<Graphic3dCamera>,
    original: Handle<SelectMgrFrustumBuilder>,
    next: Handle<SelectMgrFrustumBuilder>,
}
fn builder_getter_scene() -> BuilderGetterScene {
    let first = Handle::new(Graphic3dCamera::default());
    let second = Handle::new(Graphic3dCamera::default());
    let original = Handle::new(SelectMgrFrustumBuilder::new());
    let next = Handle::new(SelectMgrFrustumBuilder::new());
    next.set_camera(&second);
    BuilderGetterScene {
        first,
        second,
        original,
        next,
    }
}
pub(super) fn builder_getter_rows() -> Vec<String> {
    let scene = builder_getter_scene();
    let alias = scene.original.camera_ref();
    (0..6)
        .map(|phase| {
            change_builder_getter_camera(phase, &scene.original, &scene.first, &scene.second);
            builder_getter_row(phase, &alias, &scene)
        })
        .collect()
}
fn builder_getter_row(
    phase: i32,
    alias: &dyn HandleReference<Graphic3dCamera>,
    scene: &BuilderGetterScene,
) -> String {
    let fresh = scene.original.camera_ref();
    let empty = SelectMgrSelectingVolumeManager::new().camera_ref();
    let fields = CameraGetterFields {
        alias,
        fresh: &fresh,
        original: &fresh,
        empty: &empty,
    };
    let counts = CameraGetterCounts {
        first: &scene.first,
        second: &scene.second,
        owner_counts: [scene.original.ref_count(), scene.next.ref_count()],
    };
    camera_getter_row([0, phase, 0], fields, counts)
}
fn change_builder_getter_camera(
    phase: i32,
    builder: &SelectMgrFrustumBuilder,
    first: &Handle<Graphic3dCamera>,
    second: &Handle<Graphic3dCamera>,
) {
    match phase {
        1 | 4 => builder.set_camera(first),
        2 | 5 => builder.set_camera(second),
        3 => builder.set_camera(&Handle::default()),
        _ => {}
    }
}

fn selected_volume_camera_reference(
    kind: i32,
    scene: &GetterVolumeScene,
) -> SelectMgrCameraFieldReference {
    if kind == 1 {
        return scene.original.camera_ref();
    }
    scene.manager.camera_ref()
}
