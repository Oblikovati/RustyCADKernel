//! Public queries retain native volume reuse and partial update order.
#[path = "support/pick_scene.rs"]
mod pick_scene;
#[path = "support/traversal_scene.rs"]
mod traversal_scene;
use pick_scene::{pick_view, run_public_pick};
use rustycad_tkv3d::{select_basics::SelectBasicsSelectingVolumeManager, select_mgr::*};
use std::sync::atomic::Ordering;
use traversal_scene::{TraversalScene, volume::TraversalConfig};
#[test]
fn public_pick_switches_volumes_and_dispatches_derived_sorter() {
    let scene = TraversalScene::new(TraversalConfig::default());
    let (_viewer, view) = pick_view(TraversalConfig::default());
    for (index, kind) in [1, 2, 4, 3, 3, 1].into_iter().enumerate() {
        run_public_pick(scene.selector.get().unwrap(), &view, kind).unwrap();
        assert_eq!(scene.selector.sorts.load(Ordering::Acquire), index + 1);
        let volume = scene
            .selector
            .viewer_selector_base()
            .selecting_volume
            .read();
        assert_eq!(volume.camera().is_null(), kind == 3);
        assert!(
            scene
                .selector
                .viewer_selector_base()
                .fields
                .read()
                .results
                .is_sorted
        );
    }
}
#[test]
fn point_then_box_reuses_volume_and_axis_retains_camera_frame() {
    let scene = TraversalScene::new(TraversalConfig::default());
    let (_viewer, view) = pick_view(TraversalConfig::default());
    scene.selector.pick_point(40, 30, &view).unwrap();
    let base = scene.selector.viewer_selector_base();
    let point = base.selecting_volume.read().active_volume().clone();
    scene.selector.pick_box(20, 15, 60, 45, &view).unwrap();
    assert_eq!(*base.selecting_volume.read().active_volume(), point);
    let frame = base.fields.read().settings.camera_eye;
    run_public_pick(scene.selector.get().unwrap(), &view, 3).unwrap();
    assert_eq!(base.fields.read().settings.camera_eye, frame);
    assert_eq!(
        base.selecting_volume.read().get_active_selection_type(),
        SelectMgrSelectionType::Point
    );
}
#[test]
fn public_pick_failure_keeps_completed_results_and_recovers() {
    let config = TraversalConfig {
        failing: 4,
        flags: 8,
        ..Default::default()
    };
    let scene = TraversalScene::new(config);
    let (_viewer, view) = pick_view(config);
    assert!(scene.selector.pick_box(20, 15, 60, 45, &view).is_err());
    assert_eq!(scene.selector.sorts.load(Ordering::Acquire), 0);
    scene.entities[4].fail.store(0, Ordering::Release);
    scene.selector.pick_box(20, 15, 60, 45, &view).unwrap();
    assert_eq!(scene.selector.sorts.load(Ordering::Acquire), 1);
}
#[test]
fn missing_window_fails_after_point_initialization() {
    let scene = TraversalScene::new(TraversalConfig::default());
    let viewer = rustycad_tkv3d::v3d::V3dViewer::new(
        &rustycad_tkservice::null_driver::NullGraphicDriver::new_handle(),
    )
    .unwrap();
    let view = viewer.create_view().unwrap();
    let error = scene.selector.pick_point(40, 30, &view).unwrap_err();
    assert!(error.to_string().contains("window=null"));
    assert_eq!(scene.selector.sorts.load(Ordering::Acquire), 0);
    assert_eq!(
        scene
            .selector
            .viewer_selector_base()
            .selecting_volume
            .read()
            .camera(),
        view.camera()
    );
}
