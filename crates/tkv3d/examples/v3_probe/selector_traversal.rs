//! Two-level traversal over the shared real-point test scene.
#[path = "../../tests/support/traversal_scene.rs"]
pub(super) mod traversal_scene;
use super::{
    common::{Reader, attempt, dir, end, flag, int, pnt, real},
    selection_ranking::criterion_state,
};
use rustycad_tkernel::{ncollection::Mat4, standard::Failure};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::select_mgr::*;
use std::sync::atomic::Ordering;
use traversal_scene::{TraversalScene, volume::TraversalConfig};
pub(super) fn traversal_scene_state(scene: &TraversalScene) {
    int(scene.selector.sorts.load(Ordering::Acquire) as i32);
    let fields = scene.selector.viewer_selector_base().fields.read();
    flag(fields.results.is_sorted);
    pnt(fields.settings.camera_eye);
    dir(fields.settings.camera_direction);
    real(fields.settings.camera_scale);
    int(fields.results.stored.len() as i32);
    for (owner, criterion) in fields.results.stored.items() {
        int(scene
            .owners
            .iter()
            .position(|value| value == owner)
            .unwrap() as i32);
        int(scene
            .entities
            .iter()
            .find(|entity| entity.object_address() == criterion.entity.object_address())
            .unwrap()
            .tag);
        criterion_state(criterion);
    }
    int(fields.results.indexes.len() as i32);
    for index in fields.results.indexes.as_slice() {
        int(*index);
    }
    drop(fields);
    let mut events = scene.events.write();
    int(events.len() as i32);
    for event in events.iter() {
        int(*event);
    }
    events.clear();
}
fn run_scene_traversal(scene: &TraversalScene, entry: i32, view_id: i32) -> Result<(), Failure> {
    if entry != 0 {
        return scene.selector.traverse_sensitives(view_id);
    }
    let base = scene.selector.viewer_selector_base();
    let volume = base.selecting_volume.read().clone();
    let camera = volume.camera();
    let (width, height) = volume.window_size().unwrap_or((0, 0));
    let projection = camera
        .get()
        .map_or_else(Mat4::identity, Graphic3dCamera::projection_matrix);
    let world_view = camera
        .get()
        .map_or_else(Mat4::identity, Graphic3dCamera::orientation_matrix);
    let view = SelectMgrTraversalView::new(camera, projection, world_view, [width, height]);
    for object in &scene.objects {
        select_mgr_traverse_object(base, &object.cast(), &volume, &view)?;
    }
    scene.selector.sort_result()
}
pub fn selector_traversal_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let config = TraversalConfig {
        kind: r.integer(),
        camera: r.integer(),
        persistence: r.integer(),
        entity_persistence: r.integer(),
        transform: r.integer(),
        clipping: r.integer(),
        overlap: r.integer() != 0,
        flags: r.integer(),
        failing: r.integer(),
        count: r.integer(),
    };
    let entry = r.integer();
    let view_id = r.integer();
    let scene = TraversalScene::new(config);
    attempt(|| run_scene_traversal(&scene, entry, view_id));
    traversal_scene_state(&scene);
    if entry == 2 {
        for entity in &scene.entities {
            entity.fail.store(0, Ordering::Release);
        }
        attempt(|| run_scene_traversal(&scene, entry, view_id));
        traversal_scene_state(&scene);
    }
    end();
}
