//! Selection orchestration is tested through real selectors and named selectable callbacks.
#[path = "support/managed_selectable.rs"]
mod managed_selectable;
use managed_selectable::ManagedSelectionObject;
use rustycad_tkernel::standard::{Handle, SharedState};
use rustycad_tkv3d::{prs_mgr::*, select_mgr::*};
use std::{sync::Arc, sync::atomic::Ordering};
struct ManagedScene {
    selector: Handle<SelectMgrViewerSelector>,
    manager: SelectMgrSelectionManager,
    objects: Vec<Handle<ManagedSelectionObject>>,
    events: Arc<SharedState<Vec<i32>>>,
}
impl ManagedScene {
    fn new() -> Self {
        let selector = Handle::new(SelectMgrViewerSelector::new());
        let manager = SelectMgrSelectionManager::new(&selector.cast());
        let events = Arc::new(SharedState::default());
        let objects = (0..3)
            .map(|tag| ManagedSelectionObject::new(tag, events.clone()))
            .collect();
        Self {
            selector,
            manager,
            objects,
            events,
        }
    }
    fn object(&self, index: usize) -> Handle<dyn SelectMgrSelectableObject> {
        self.objects[index].cast()
    }
}
#[test]
fn load_is_child_first_and_repeated_load_does_not_compute_a_new_mode() {
    let scene = ManagedScene::new();
    scene.objects[0]
        .add_child(&scene.objects[1].cast())
        .unwrap();
    scene.events.write().clear();
    scene.manager.load(&scene.object(0), 0).unwrap();
    assert_eq!(*scene.events.read(), [10, 0]);
    assert!(scene.manager.contains(&scene.object(1)));
    scene.manager.load(&scene.object(0), 1).unwrap();
    assert!(!scene.objects[0].has_selection(1));
    assert_eq!(
        scene.manager.selector(),
        &scene.selector.cast::<dyn SelectMgrViewerSelectorSource>()
    );
}
#[test]
fn activation_skips_erased_children_and_counts_tolerance_once() {
    let scene = ManagedScene::new();
    scene.objects[0]
        .add_child(&scene.objects[1].cast())
        .unwrap();
    scene.objects[1].erased(true);
    scene.manager.load(&scene.object(0), 2).unwrap();
    scene.manager.activate(&scene.object(0), 2).unwrap();
    scene.manager.activate(&scene.object(0), 2).unwrap();
    assert!(scene.manager.is_activated(&scene.object(0), -1).unwrap());
    assert!(!scene.manager.is_activated(&scene.object(1), 2).unwrap());
    assert_eq!(scene.selector.pixel_tolerance(), 4);
    scene.manager.deactivate(&scene.object(0), -1).unwrap();
    assert_eq!(scene.selector.pixel_tolerance(), 2);
    assert!(!scene.manager.is_activated(&scene.object(0), -1).unwrap());
}
#[test]
fn non_presenting_parent_still_loads_and_removes_its_children() {
    let scene = ManagedScene::new();
    scene.objects[0]
        .add_child(&scene.objects[1].cast())
        .unwrap();
    scene.objects[0].own_presentations(false);
    scene.manager.load(&scene.object(0), 0).unwrap();
    assert!(!scene.manager.contains(&scene.object(0)));
    assert!(scene.manager.contains(&scene.object(1)));
    scene.manager.remove(&scene.object(0)).unwrap();
    assert!(!scene.manager.contains(&scene.object(1)));
    assert!(scene.objects[1].selection(0).is_empty());
}
#[test]
fn removal_keeps_the_selection_update_flag_and_reactivation_replaces_cleared_selection() {
    let scene = ManagedScene::new();
    let object = scene.object(0);
    scene.manager.load(&object, 1).unwrap();
    scene.manager.activate(&object, 1).unwrap();
    let previous = object.selection(1);
    scene.manager.remove(&object).unwrap();
    assert_eq!(previous.update_status(), SelectMgrTypeOfUpdate::None);
    assert_eq!(
        previous.bvh_update_status(),
        SelectMgrTypeOfBvhUpdate::Remove
    );
    scene.manager.load(&object, -1).unwrap();
    scene.manager.activate(&object, 1).unwrap();
    assert_ne!(previous, object.selection(1));
    assert!(!object.selection(1).is_empty());
    assert!(scene.manager.is_activated(&object, 1).unwrap());
}
#[test]
fn non_forced_recompute_includes_inactive_modes_but_update_does_not() {
    let scene = ManagedScene::new();
    let object = scene.object(0);
    scene.manager.load(&object, 1).unwrap();
    scene.events.write().clear();
    scene
        .manager
        .recompute_selection(&object, false, -1)
        .unwrap();
    assert_eq!(*scene.events.read(), [1]);
    scene
        .manager
        .set_update_mode(&object, SelectMgrTypeOfUpdate::Full)
        .unwrap();
    scene.manager.update(&object, false).unwrap();
    assert_eq!(*scene.events.read(), [1]);
    scene.manager.update(&object, true).unwrap();
    assert_eq!(*scene.events.read(), [1, 1, 100]);
}
#[test]
fn failed_computation_preserves_registration_and_recomputation_flags() {
    let scene = ManagedScene::new();
    let object = scene.object(0);
    scene.objects[0].fail_mode.store(0, Ordering::Release);
    assert!(scene.manager.load(&object, 0).is_err());
    assert!(scene.manager.contains(&object) && scene.selector.contains(&object));
    scene.objects[0].fail_mode.store(-999, Ordering::Release);
    scene.manager.activate(&object, 0).unwrap();
    scene.objects[0].fail_mode.store(0, Ordering::Release);
    assert!(
        scene
            .manager
            .recompute_selection(&object, false, 0)
            .is_err()
    );
    assert_eq!(
        object.selection(0).update_status(),
        SelectMgrTypeOfUpdate::Full
    );
    assert_eq!(
        object.selection(0).bvh_update_status(),
        SelectMgrTypeOfBvhUpdate::Add
    );
    assert!(object.selection(0).is_empty());
}
#[test]
fn sensitivity_validation_precedes_null_checks_and_updates_retained_tolerance_counts() {
    let scene = ManagedScene::new();
    let object = scene.object(0);
    assert_eq!(
        scene
            .manager
            .set_selection_sensitivity(&Handle::default(), 0, -1)
            .unwrap_err()
            .exception_type(),
        "Standard_ProgramError"
    );
    scene
        .manager
        .set_selection_sensitivity(&Handle::default(), 0, 2)
        .unwrap();
    scene.manager.load(&object, 0).unwrap();
    scene.manager.activate(&object, 0).unwrap();
    scene
        .manager
        .set_selection_sensitivity(&object, 0, 8)
        .unwrap();
    assert_eq!(scene.selector.pixel_tolerance(), 8);
    scene.manager.deactivate(&object, 0).unwrap();
    assert_eq!(scene.selector.pixel_tolerance(), 2);
    scene
        .manager
        .set_selection_sensitivity(&object, 0, 4)
        .unwrap_err();
    assert_eq!(object.selection(0).sensitivity(), 4);
}

#[path = "support/prebuild_entity.rs"]
mod prebuild_entity;
use prebuild_entity::PrebuildEntity;
use rustycad_tkv3d::{select3d::Select3DSensitiveGroup, std_select::StdSelectBrepSelectionTool};
#[test]
fn foreground_prebuild_uses_inclusive_threshold_and_only_direct_group_children() {
    let small = PrebuildEntity::new(799_999, false);
    let heavy = PrebuildEntity::new(800_000, false);
    let nested = Handle::new(Select3DSensitiveGroup::with_entities(
        &Handle::default(),
        &[heavy.cast()],
        true,
    ));
    let group = Handle::new(Select3DSensitiveGroup::with_entities(
        &Handle::default(),
        &[small.cast(), heavy.cast(), nested.cast()],
        true,
    ));
    let selection = SelectMgrSelection::new(0);
    selection.add(&group.cast()).unwrap();
    StdSelectBrepSelectionTool::pre_build_bvh(&selection).unwrap();
    assert_eq!(small.calls.load(Ordering::Acquire), 0);
    assert_eq!(heavy.calls.load(Ordering::Acquire), 1);
}
#[test]
fn foreground_prebuild_failure_stops_later_entities() {
    let first = PrebuildEntity::new(800_001, true);
    let second = PrebuildEntity::new(800_000, false);
    let selection = SelectMgrSelection::new(0);
    selection.add(&first.cast()).unwrap();
    selection.add(&second.cast()).unwrap();
    assert!(StdSelectBrepSelectionTool::pre_build_bvh(&selection).is_err());
    assert_eq!(first.calls.load(Ordering::Acquire), 1);
    assert_eq!(second.calls.load(Ordering::Acquire), 0);
}
#[test]
fn manager_builds_foreground_heavy_entities_and_background_group_children() {
    let scene = ManagedScene::new();
    let heavy = PrebuildEntity::new(800_000, false);
    *scene.objects[0].provided.write() = Some(vec![heavy.cast()]);
    scene.manager.load(&scene.object(0), 0).unwrap();
    assert_eq!(heavy.calls.load(Ordering::Acquire), 1);
    let small = PrebuildEntity::new(1, false);
    let group = Handle::new(Select3DSensitiveGroup::with_entities(
        &Handle::default(),
        &[small.cast()],
        true,
    ));
    *scene.objects[1].provided.write() = Some(vec![group.cast()]);
    scene.selector.set_to_prebuild_bvh(true, 1);
    scene.manager.load(&scene.object(1), 0).unwrap();
    small.wait_built();
    scene.selector.set_to_prebuild_bvh(false, 1);
    assert_eq!(small.calls.load(Ordering::Acquire), 1);
}
