//! Owner-slot timing and per-owner result updates through actual sensitive callbacks.
#[allow(dead_code)]
#[path = "support/boxed_selectable.rs"]
mod boxed_selectable;
#[path = "support/overlap_sensitive.rs"]
mod overlap_sensitive;
use boxed_selectable::BoxedSelectable;
use overlap_sensitive::OverlapSensitivePoint;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{GeneralTransform3, Point2, Point3};
use rustycad_tkv3d::{
    prs_mgr::PrsMgrPresentableObject, select_basics::*, select_mgr::*, select3d::*,
};
use std::sync::atomic::Ordering;
fn pick_entity(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    depth: f64,
) -> Handle<OverlapSensitivePoint> {
    let entity = OverlapSensitivePoint::new(owner);
    *entity.pick.write() = SelectBasicsPickResult::new(depth, 1., Point3::new(1., 2., 3.));
    entity
}
fn overlap(
    selector: &SelectMgrViewerSelector,
    entity: &Handle<OverlapSensitivePoint>,
    volume: &SelectMgrSelectingVolumeManager,
) {
    select_mgr_check_overlap(
        selector.viewer_selector_base(),
        &entity.cast(),
        &GeneralTransform3::default(),
        volume,
    )
    .unwrap();
}
#[test]
fn null_owner_still_matches_and_replacement_uses_pre_match_selectable() {
    let selector = Handle::new(SelectMgrViewerSelector::new());
    let object = BoxedSelectable::new(0., false);
    object.set_z_layer(20);
    let owner = SelectMgrEntityOwner::with_selectable(&object.cast(), 9);
    selector
        .viewer_selector_base()
        .fields
        .write()
        .z_layer_order
        .bind(20, 7);
    let entity = pick_entity(&Handle::default(), 2.);
    *entity.selector.write() = selector
        .cast::<dyn SelectMgrViewerSelectorSource>()
        .downgrade();
    let volume = SelectMgrSelectingVolumeManager::new();
    overlap(&selector, &entity, &volume);
    assert_eq!(*entity.events.read(), [1]);
    assert_eq!(selector.nb_picked(), 0);
    *entity.match_owner.write() = Some(owner.cast());
    overlap(&selector, &entity, &volume);
    let fields = selector.viewer_selector_base().fields.read();
    let criterion = fields.results.stored.find_from_key(&owner.cast()).unwrap();
    assert_eq!(criterion.selection_priority, 9);
    assert_eq!(criterion.z_layer_position, 0);
}
#[test]
fn repeated_owner_count_survives_nonwinning_matches_and_box_skips_point_update() {
    let selector = SelectMgrViewerSelector::new();
    let owner = SelectMgrEntityOwner::new(3);
    let entity = pick_entity(&owner.cast(), 5.);
    let mut volume = SelectMgrSelectingVolumeManager::new();
    volume.init_box_selecting_volume(Point2::new(0., 0.), Point2::new(1., 1.));
    overlap(&selector, &entity, &volume);
    entity.pick.write().set_depth(1.);
    overlap(&selector, &entity, &volume);
    let fields = selector.viewer_selector_base().fields.read();
    let criterion = fields.results.stored.find_from_key(&owner.cast()).unwrap();
    assert_eq!(criterion.nb_owner_matches, 2);
    assert_eq!(criterion.depth, 5.);
    assert_eq!(*entity.events.read(), [1, 1]);
}
#[test]
fn closer_match_replaces_entity_without_changing_insertion_index_or_sorted_flag() {
    let selector = SelectMgrViewerSelector::new();
    selector
        .viewer_selector_base()
        .fields
        .write()
        .results
        .is_sorted = true;
    let owner = SelectMgrEntityOwner::new(3);
    let first = pick_entity(&owner.cast(), 5.);
    let closer = pick_entity(&owner.cast(), 1.);
    let volume = SelectMgrSelectingVolumeManager::new();
    overlap(&selector, &first, &volume);
    overlap(&selector, &closer, &volume);
    overlap(&selector, &first, &volume);
    let fields = selector.viewer_selector_base().fields.read();
    let criterion = fields.results.stored.find_from_index(1).unwrap();
    assert_eq!(
        criterion.entity,
        closer.cast::<dyn Select3DSensitiveEntity>()
    );
    assert_eq!(criterion.nb_owner_matches, 3);
    assert_eq!(criterion.depth, 1.);
    assert!(fields.results.is_sorted);
}
#[test]
fn owner_slot_changed_by_point_update_is_used_only_when_inserting_a_new_owner() {
    let selector = Handle::new(SelectMgrViewerSelector::new());
    let old = SelectMgrEntityOwner::new(3);
    let new = SelectMgrEntityOwner::new(9);
    let entity = pick_entity(&old.cast(), 2.);
    *entity.selector.write() = selector
        .cast::<dyn SelectMgrViewerSelectorSource>()
        .downgrade();
    *entity.location_owner.write() = Some(new.cast());
    let mut volume = SelectMgrSelectingVolumeManager::new();
    volume.init_point_selecting_volume(Point2::new(0., 0.));
    overlap(&selector, &entity, &volume);
    let fields = selector.viewer_selector_base().fields.read();
    let criterion = fields.results.stored.find_from_key(&new.cast()).unwrap();
    assert_eq!(criterion.selection_priority, 3);
    assert_eq!(*entity.events.read(), [1, 2]);
    drop(fields);
    entity.set(&new.cast());
    *entity.location_owner.write() = Some(old.cast());
    entity.pick.write().set_depth(1.);
    overlap(&selector, &entity, &volume);
    let fields = selector.viewer_selector_base().fields.read();
    assert!(fields.results.stored.seek(&old.cast()).is_none());
    assert_eq!(
        fields
            .results
            .stored
            .find_from_key(&new.cast())
            .unwrap()
            .nb_owner_matches,
        2
    );
}
#[test]
fn failing_match_keeps_owner_changes_but_does_not_add_or_increment_a_criterion() {
    let selector = SelectMgrViewerSelector::new();
    let owner = SelectMgrEntityOwner::new(3);
    let entity = pick_entity(&owner.cast(), 2.);
    let volume = SelectMgrSelectingVolumeManager::new();
    overlap(&selector, &entity, &volume);
    entity.decision.store(2, Ordering::Release);
    *entity.match_owner.write() = Some(Handle::default());
    assert!(
        select_mgr_check_overlap(
            selector.viewer_selector_base(),
            &entity.cast(),
            &GeneralTransform3::default(),
            &volume
        )
        .is_err()
    );
    assert!(entity.owner_id().is_null());
    assert_eq!(
        selector
            .viewer_selector_base()
            .fields
            .read()
            .results
            .stored
            .find_from_key(&owner.cast())
            .unwrap()
            .nb_owner_matches,
        1
    );
}
