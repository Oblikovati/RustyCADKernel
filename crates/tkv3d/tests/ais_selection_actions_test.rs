//! Martensite AIS selection preserves native membership, callback and cursor behavior.
#[path = "support/detection_scene.rs"]
mod detection_scene;
#[path = "support/selection_actions_scene.rs"]
mod scene;
use rustycad_tkernel::{
    ncollection::{Array1, Vec2},
    standard::Handle,
};
use rustycad_tkv3d::ais::*;
use scene::SelectionActionsScene;

#[test]
fn selection_schemes_preserve_order_and_public_cursor() {
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 0);
    scene.operation(0, 3, 0).unwrap();
    scene.operation(11, 0, 0).unwrap();
    scene.operation(12, 0, 0).unwrap();
    let ctx = &scene.detection.context;
    assert_eq!(ctx.selected_owner().unwrap(), scene.owner(1));
    assert_eq!(ctx.first_selected_object(), scene.detection.object);
    scene.operation(0, 6, 3).unwrap();
    assert_eq!(ctx.nb_selected(), 2);
    assert_eq!(scene.detection.owner_id(&scene.owner(2)), 2);
    assert!(ctx.is_selected_owner(&scene.owner(0)));
    assert!(!ctx.is_selected_owner(&scene.owner(1)));
    assert!(ctx.is_selected_owner(&scene.owner(2)));
    scene.operation(10, 0, 1).unwrap();
    assert_eq!(ctx.nb_selected(), 0);
    assert!(!ctx.more_selected());
    assert!(ctx.selected_owner().unwrap().is_null());
}
#[test]
fn selected_owner_rejected_by_filter_can_still_be_removed() {
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 0);
    scene.operation(5, 1, 0).unwrap();
    scene.operation(13, 0, 0).unwrap();
    scene.operation(5, 1, 0).unwrap();
    assert_eq!(scene.detection.context.nb_selected(), 0);
    scene.operation(5, 2, 0).unwrap();
    assert_eq!(scene.detection.context.nb_selected(), 0);
}
#[test]
fn handled_click_leaves_existing_selection_and_remove_skips_click() {
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 0);
    scene.operation(1, 0, 0).unwrap();
    scene.operation(15, 1, 0).unwrap();
    scene.operation(17, 1, 0).unwrap();
    assert_eq!(scene.operation(16, 0, 0).unwrap(), vec![1]);
    assert_eq!(scene.detection.context.nb_selected(), 1);
    assert!(scene.detection.context.is_selected_owner(&scene.owner(0)));
    scene.detection.trace.write().events.clear();
    scene.operation(16, 0, 2).unwrap();
    assert!(!scene.detection.trace.read().events.contains(&901));
}
#[test]
fn invalid_owner_style_keeps_output_but_missing_object_style_clears_it() {
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 0);
    let ctx = &scene.detection.context;
    let mut style = scene.style.clone();
    assert!(!ctx.owner_highlight_style(&Handle::default(), &mut style));
    assert_eq!(style, scene.style);
    assert!(!ctx.object_highlight_style(&scene.detection.object.cast(), &mut style));
    assert!(style.is_null());
}
#[test]
fn nonautomatic_owners_are_grouped_in_selection_order() {
    let scene = SelectionActionsScene::new(1024, 7, 2, 0, 0);
    scene.operation(0, 7, 0).unwrap();
    assert!(
        scene
            .detection
            .trace
            .read()
            .events
            .windows(5)
            .any(|row| row == [700, 3, 0, 1, 2])
    );
    scene.operation(24, 0, 0).unwrap();
    assert!(scene.detection.trace.read().events.contains(&800));
}
#[test]
fn hilight_selected_visits_an_owner_appended_by_a_callback() {
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 0);
    scene.operation(1, 0, 0).unwrap();
    scene.operation(38, 2, 0).unwrap();
    scene.operation(8, 0, 0).unwrap();
    assert_eq!(scene.detection.context.nb_selected(), 2);
    assert!(scene.detection.trace.read().events.contains(&202));
}
#[test]
fn camera_selection_uses_real_sensitive_geometry_and_empty_replacement_clears() {
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 0);
    let ctx = &scene.detection.context;
    assert_eq!(
        ctx.select_point(
            Vec2::new(40, 30),
            &scene.detection.view,
            AisSelectionScheme::Replace
        )
        .unwrap(),
        AisStatusOfPick::SeveralSelected
    );
    assert_eq!(ctx.nb_selected(), 3);
    assert_eq!(
        ctx.select_owners(&Array1::default(), AisSelectionScheme::Replace)
            .unwrap(),
        AisStatusOfPick::NothingSelected
    );
}
#[test]
fn pick_status_discriminants_match_native_values() {
    assert_eq!(
        [
            AisStatusOfPick::Error,
            AisStatusOfPick::NothingSelected,
            AisStatusOfPick::Removed,
            AisStatusOfPick::OneSelected,
            AisStatusOfPick::SeveralSelected
        ]
        .map(|status| status as i32),
        [0, 1, 2, 3, 4]
    );
}
#[test]
fn selection_failure_preserves_mutation_for_recovery() {
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 1);
    assert!(scene.operation(0, 7, 0).is_err());
    assert_eq!(scene.detection.context.nb_selected(), 3);
    assert!(scene.status.is_hilighted());
    scene.operation(14, 0, 0).unwrap();
    scene.operation(8, 0, 0).unwrap();
    assert!(
        scene
            .detection
            .context
            .is_hilighted_object(&scene.detection.object.cast())
    );
}
#[test]
fn context_assignment_reaches_children_and_rejects_foreign_context() {
    use detection_scene::object::DetectionObject;
    use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObject;
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 0);
    let child = DetectionObject::new(scene.detection.trace.clone());
    scene.detection.object.add_child(&child.cast()).unwrap();
    scene
        .detection
        .context
        .set_context_to_object(&scene.detection.object.cast())
        .unwrap();
    assert_eq!(child.get_context(), scene.detection.context);
    let foreign = SelectionActionsScene::new(0, 7, 2, 0, 0);
    assert!(
        foreign
            .detection
            .context
            .set_context_to_object(&child.cast())
            .is_err()
    );
    assert_eq!(child.get_context(), scene.detection.context);
}
#[test]
fn selection_drawer_replacement_is_observed_after_accept_mode_callback() {
    for mutation in [3, 4] {
        let scene = SelectionActionsScene::new(0, 7, 2, mutation, 0);
        scene.operation(0, 1, 0).unwrap();
        assert!(
            scene
                .detection
                .trace
                .read()
                .events
                .windows(5)
                .any(|row| row == [200, 9, 9, 0, 0])
        );
    }
}
#[test]
fn selection_validates_viewer_before_remembering_a_foreign_view() {
    let scene = SelectionActionsScene::new(0, 7, 2, 0, 0);
    assert!(scene.detection.context.last_active_view().is_null());
    assert!(scene.operation(39, 0, 0).is_err());
    assert!(scene.detection.context.last_active_view().is_null());
    scene.operation(18, 0, 0).unwrap();
    assert_eq!(
        scene.detection.context.last_active_view(),
        scene.detection.view
    );
}
