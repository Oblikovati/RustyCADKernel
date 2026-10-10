//! Public pick overloads on real views with repeated volume switches.
use super::{
    common::{Reader, attempt, end, flag, int},
    selector_traversal::{traversal_scene, traversal_scene_state},
};
#[path = "../../tests/support/pick_scene.rs"]
pub(super) mod pick_scene;
use pick_scene::{pick_view, run_public_pick};
use rustycad_tkv3d::{select_basics::SelectBasicsSelectingVolumeManager, select_mgr::*};
use traversal_scene::{TraversalScene, volume::TraversalConfig};
pub fn selector_pick_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let config = TraversalConfig {
        camera: r.integer(),
        persistence: r.integer(),
        entity_persistence: r.integer(),
        transform: r.integer(),
        clipping: r.integer(),
        overlap: r.integer() != 0,
        flags: r.integer(),
        failing: r.integer(),
        count: r.integer(),
        ..Default::default()
    };
    let scene = TraversalScene::new(config);
    let (_viewer, view) = pick_view(config);
    scene.selector.set_pixel_tolerance(r.integer());
    let count = r.integer();
    for _ in 0..count {
        let kind = r.integer();
        attempt(|| run_public_pick(scene.selector.get().unwrap(), &view, kind));
        traversal_scene_state(&scene);
        let volume = scene
            .selector
            .viewer_selector_base()
            .selecting_volume
            .read()
            .clone();
        int(volume.get_active_selection_type() as i32);
        flag(volume.camera().is_null());
        let (width, height) = volume.window_size().unwrap_or((0, 0));
        int(width);
        int(height);
        flag(volume.view_clipping().is_null());
        flag(volume.object_clipping().is_null());
        int(scene
            .selector
            .viewer_selector_base()
            .fields
            .read()
            .z_layer_order
            .extent() as i32);
    }
    end();
}
