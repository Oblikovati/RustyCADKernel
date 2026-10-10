//! AIS detection follows native callbacks, cursor independence and public camera picking.
#[path = "support/detection_scene.rs"]
mod scene;
use rustycad_tkv3d::{ais::*, select_mgr::*};
use scene::DetectionScene;
#[test]
fn detection_filters_preserve_cycling_candidates_and_cursor_independence() {
    let scene = DetectionScene::new(16, 6, -1, 0, 0);
    scene.seed(7);
    assert_eq!(
        scene.context.move_to_detected(&scene.view, false).unwrap(),
        AisStatusOfDetection::Nothing
    );
    assert!(!scene.context.has_detected());
    assert!(scene.context.has_next_detected());
    scene.context.init_detected();
    assert_eq!(
        scene.context.detected_current_owner().unwrap(),
        scene.owners[1]
    );
    assert_eq!(
        scene
            .context
            .hilight_next_detected(&scene.view, false)
            .unwrap(),
        1
    );
    assert_eq!(scene.owner_id(&scene.context.detected_owner()), 1);
    scene.context.next_detected();
    assert_eq!(
        scene.context.detected_current_owner().unwrap(),
        scene.owners[2]
    );
    assert_eq!(scene.owner_id(&scene.context.detected_owner()), 1);
}
#[test]
fn repeated_detection_skips_highlighting_unless_owner_forces_it() {
    let scene = DetectionScene::new(0, 7, -1, 0, 0);
    scene.seed(7);
    assert_eq!(
        scene.context.move_to_detected(&scene.view, false).unwrap(),
        AisStatusOfDetection::OnlyOneDetected
    );
    scene.trace.write().events.clear();
    scene.context.move_to_detected(&scene.view, false).unwrap();
    assert_eq!(scene.trace.read().events, vec![500, 501, 502]);
    scene.trace.write().forced = true;
    scene.context.move_to_detected(&scene.view, false).unwrap();
    assert!(scene.trace.read().events.contains(&300));
    assert!(scene.context.clear_detected(false));
    assert_eq!(scene.selector().nb_picked(), 0);
    assert!(!scene.context.more_detected());
}
#[test]
fn drawer_replacement_during_accept_mode_is_observed_without_a_lock() {
    for mutation in [1, 2] {
        let scene = DetectionScene::new(0, 7, 2, mutation, 0);
        scene.seed(1);
        scene.context.move_to_detected(&scene.view, false).unwrap();
        assert!(
            scene
                .trace
                .read()
                .events
                .windows(5)
                .any(|row| row == [200, 9, 9, 0, 1])
        );
    }
}
#[test]
fn highlight_failure_retains_last_owner_and_immediate_draw_state() {
    let scene = DetectionScene::new(0, 7, 2, 0, 1);
    scene.seed(7);
    assert!(scene.context.move_to_detected(&scene.view, true).is_err());
    assert_eq!(*scene.context.detected_owner(), scene.owners[0]);
    assert!(scene.context.main_prs_mgr().is_immediate_mode_on());
    assert_eq!(scene.context.fields.read().detected.len(), 3);
}
#[test]
fn public_pixel_and_axis_detection_use_real_geometry_and_validate_view_first() {
    let scene = DetectionScene::new(0, 7, -1, 0, 0);
    scene.operation(11, 0).unwrap();
    assert_eq!(scene.operation(9, 0).unwrap(), Some(4));
    assert_eq!(scene.operation(10, 0).unwrap(), Some(4));
    assert!(scene.operation(12, 0).is_err());
    assert!(scene.context.has_detected());
    assert_eq!(scene.operation(9, 200).unwrap(), Some(1));
}
#[test]
fn native_detection_enum_values_are_stable() {
    let values = [
        AisStatusOfDetection::Error,
        AisStatusOfDetection::Nothing,
        AisStatusOfDetection::AllBad,
        AisStatusOfDetection::Selected,
        AisStatusOfDetection::OnlyOneDetected,
        AisStatusOfDetection::OnlyOneGood,
        AisStatusOfDetection::SeveralGood,
    ];
    assert_eq!(values.map(|value| value as i32), [0, 1, 2, 3, 4, 5, 6]);
}
#[test]
fn highlight_mode_priority_and_original_style_slot_follow_native_rules() {
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkservice::graphic3d::Graphic3dPresentationAttributesSource;
    use rustycad_tkv3d::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dDrawer};
    let scene = DetectionScene::new(0, 7, -1, 0, 0);
    let object = scene.object.cast();
    let empty = Handle::default();
    assert_eq!(scene.context.get_hilight_mode(&object, &empty, -1), 3);
    scene.object.attributes().set_display_mode(6);
    assert_eq!(scene.context.get_hilight_mode(&object, &empty, -1), 6);
    assert_eq!(scene.context.get_hilight_mode(&object, &empty, 8), 8);
    let slot = scene.context.get_hi_style(&object, &scene.owners[0]);
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.set_display_mode(2);
    scene.context.set_dynamic_highlight_style(&drawer);
    assert_eq!(slot.read(), drawer);
    assert_eq!(scene.context.get_hilight_mode(&object, &drawer, 8), 2);
    scene.trace.write().accept = false;
    assert_eq!(scene.context.get_hilight_mode(&object, &drawer, 8), 8);
}
#[test]
fn slow_highlighting_invalidates_until_a_requested_update_redraws() {
    let scene = DetectionScene::new(128, 7, -1, 0, 0);
    scene.seed(1);
    scene.context.move_to_detected(&scene.view, false).unwrap();
    assert!(scene.view.is_invalidated());
    scene.context.move_to_detected(&scene.view, true).unwrap();
    assert!(scene.view.is_invalidated());
    scene.trace.write().forced = true;
    scene.context.move_to_detected(&scene.view, true).unwrap();
    assert!(!scene.view.is_invalidated());
}
