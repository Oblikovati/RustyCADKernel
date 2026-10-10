//! ViewerSelector state, virtual ranking, object lifecycle and background construction.
#[path = "support/boxed_selectable.rs"]
#[allow(dead_code)]
mod boxed_selectable;
#[path = "support/selector_scene.rs"]
#[allow(dead_code)]
mod selector_scene;
use selector_scene::{mode_selection, seeded_pick};
#[path = "support/bvh_pool_fixture.rs"]
#[allow(dead_code)]
mod bvh_pool_fixture;
use boxed_selectable::BoxedSelectable;
use rustycad_tkernel::{
    ncollection::{Array1, List},
    standard::{Failure, Handle, RuntimeTyped, StandardTransient},
};
use rustycad_tkmath::{
    bvh::BvhObjectTransientSource,
    gp::{Axis1, Point3},
};
use rustycad_tkv3d::select_mgr::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn constructor_and_tolerance_preferences_preserve_native_defaults_and_early_returns() {
    let selector = Handle::new(SelectMgrViewerSelector::new());
    assert!(
        !selector
            .clone()
            .cast::<dyn SelectMgrViewerSelectorSource>()
            .is_null()
    );
    assert_eq!(
        SelectMgrViewerSelector::static_type().name(),
        "SelectMgr_ViewerSelector"
    );
    assert_eq!(
        (
            selector.pixel_tolerance(),
            selector.custom_pixel_tolerance(),
            selector.sensitivity()
        ),
        (2, -1, 2.)
    );
    selector.set_pixel_tolerance(2);
    assert_eq!(selector.custom_pixel_tolerance(), -1);
    selector.set_pixel_tolerance(4);
    selector.viewer_selector_base().tolerances.write().add(6);
    assert_eq!(selector.pixel_tolerance(), 10);
    selector.set_pixel_tolerance(-5);
    assert_eq!(
        (
            selector.pixel_tolerance(),
            selector.custom_pixel_tolerance()
        ),
        (6, -1)
    );
    assert!(selector.to_pick_closest());
    assert_eq!(
        selector.depth_tolerance_type(),
        SelectMgrTypeOfDepthTolerance::SensitivityFactor
    );
    selector.set_depth_tolerance(SelectMgrTypeOfDepthTolerance::Uniform, -7.);
    assert_eq!(
        (selector.depth_tolerance_type(), selector.depth_tolerance()),
        (SelectMgrTypeOfDepthTolerance::Uniform, -7.)
    );
}
#[test]
fn lazy_sorting_prefers_depth_until_explicitly_resorted_after_configuration_changes() {
    let selector = SelectMgrViewerSelector::new();
    let near = seeded_pick(&selector, &Handle::default(), 1., 1);
    let far = seeded_pick(&selector, &Handle::default(), 9., 9);
    assert_eq!(selector.one_picked().unwrap(), near);
    selector.set_pick_closest(false);
    assert_eq!(
        selector.picked(1).unwrap(),
        near,
        "native preference setter does not dirty ranks"
    );
    selector.sort_result().unwrap();
    assert_eq!(selector.picked(1).unwrap(), far);
    assert_eq!(selector.picked_point(1).unwrap(), Point3::new(9., 1., 2.));
    assert!(!selector.picked_entity(1).unwrap().is_null());
    let entity = selector.picked_entity(1).unwrap();
    let picked = selector.picked_data(1).unwrap();
    let fields = selector.viewer_selector_base().fields.read();
    assert!(std::ptr::eq(&*entity, &picked.entity));
    assert!(std::ptr::eq(
        &*picked,
        fields.results.stored.find_from_index(2).unwrap()
    ));
}
#[test]
fn clear_and_invalid_ranks_preserve_native_permutation_storage() {
    let selector = SelectMgrViewerSelector::default();
    seeded_pick(&selector, &Handle::default(), 1., 1);
    assert!(selector.picked(0).unwrap().is_null());
    assert!(selector.picked(2).unwrap().is_null());
    assert_eq!(
        selector.picked_data(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(
        !selector
            .viewer_selector_base()
            .fields
            .read()
            .results
            .is_sorted
    );
    selector.sort_result().unwrap();
    selector.clear();
    assert_eq!(selector.nb_picked(), 0);
    assert!(selector.one_picked().unwrap().is_null());
    selector.sort_result().unwrap();
    assert_eq!(
        selector
            .viewer_selector_base()
            .fields
            .read()
            .results
            .indexes
            .len(),
        1
    );
}
struct ReverseSelector {
    transient: StandardTransient,
    base: SelectMgrViewerSelectorBase,
    calls: AtomicUsize,
}
rustycad_tkernel::impl_transient!(ReverseSelector, "Test_ReverseSelector", dyn SelectMgrViewerSelectorSource, transient,
    interfaces: [dyn SelectMgrViewerSelectorSource]);
impl SelectMgrViewerSelectorSource for ReverseSelector {
    fn viewer_selector_base(&self) -> &SelectMgrViewerSelectorBase {
        &self.base
    }
    fn sort_result(&self) -> Result<(), Failure> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        let mut fields = self.base.fields.write();
        let count = fields.results.stored.extent() as i32;
        fields.results.indexes = Array1::with_bounds(1, count).unwrap();
        for (index, rank) in fields.results.indexes.as_mut_slice().iter_mut().enumerate() {
            *rank = count - index as i32;
        }
        fields.results.is_sorted = true;
        Ok(())
    }
}
#[test]
fn lazy_access_dispatches_to_the_derived_sorter() {
    let selector = Handle::new(ReverseSelector {
        transient: StandardTransient::default(),
        base: SelectMgrViewerSelectorBase::new(),
        calls: AtomicUsize::new(0),
    });
    seeded_pick(&*selector, &Handle::default(), 1., 0);
    let second = seeded_pick(&*selector, &Handle::default(), 2., 0);
    let base = selector.clone().cast::<dyn SelectMgrViewerSelectorSource>();
    assert!(base.picked(-1).unwrap().is_null());
    assert_eq!(selector.calls.load(Ordering::Acquire), 0);
    assert_eq!(base.picked(1).unwrap(), second);
    assert_eq!(base.picked_data(1).unwrap().depth, 2.);
    assert_eq!(selector.calls.load(Ordering::Acquire), 1);
}
#[test]
fn absent_mode_queries_append_and_status_reports_containment_separately() {
    let selector = SelectMgrViewerSelector::new();
    let object = BoxedSelectable::new(0., false).cast();
    let selection = mode_selection(&object, 3);
    selection.set_selection_state(SelectMgrStateOfSelection::Deactivated);
    let mut modes = List::default();
    modes.append(99);
    assert!(
        !selector
            .modes(&object, &mut modes, SelectMgrStateOfSelection::Any)
            .unwrap()
    );
    assert_eq!(modes.iter().copied().collect::<Vec<_>>(), vec![99, 3]);
    assert_eq!(
        selector.object_status(&object).unwrap().as_bytes(),
        b"Status Object :\n\tMode 3 present -  Inactive \n\tNot Present in the selector\n\n"
    );
    assert_eq!(
        selector.selection_status(&selection).unwrap(),
        SelectMgrStateOfSelection::Deactivated
    );
    assert!(!selector.is_inside(&object, 3).unwrap());
    selector.add_selectable_object(&object).unwrap();
    assert!(selector.is_inside(&object, 3).unwrap());
    assert!(!selector.is_active(&object, 3).unwrap());
    selection.set_selection_state(SelectMgrStateOfSelection::Activated);
    assert!(selector.is_active(&object, 3).unwrap());
    assert!(!selector.is_active(&object, -1).unwrap());
}
#[test]
fn object_removal_removes_all_matching_picks_and_keeps_other_owners() {
    let selector = SelectMgrViewerSelector::new();
    let a = BoxedSelectable::new(0., false).cast();
    let b = BoxedSelectable::new(3., false).cast();
    seeded_pick(&selector, &a, 1., 0);
    seeded_pick(&selector, &a, 2., 0);
    let remaining = seeded_pick(&selector, &b, 3., 0);
    assert!(!selector.remove_picked(&a).unwrap());
    selector.add_selectable_object(&a).unwrap();
    selector.add_selectable_object(&b).unwrap();
    selector.remove_selectable_object(&a).unwrap();
    assert!(!selector.contains(&a));
    assert!(selector.contains(&b));
    assert_eq!(selector.one_picked().unwrap(), remaining);
    assert!(!selector.remove_picked(&a).unwrap());
    assert_eq!(selector.nb_picked(), 1);
}
#[test]
fn entity_sets_rebuild_remove_and_preserve_their_tree_when_builder_changes() {
    let selector = SelectMgrViewerSelector::new();
    let object = BoxedSelectable::new(0., false).cast();
    let selection = mode_selection(&object, 0);
    selector
        .add_selection_to_object(&object, &selection)
        .unwrap();
    let set = selector
        .viewer_selector_base()
        .object_sensitives
        .read()
        .find(&object)
        .unwrap()
        .clone();
    let tree = set.bvh().unwrap();
    assert_eq!(set.size(), 1);
    selector.set_entity_set_builder(None);
    assert!(selector.entity_set_builder().is_none());
    assert!(!set.is_dirty());
    assert!(tree.ptr_eq(&set.bvh().unwrap()));
    selector.rebuild_sensitives_tree(&object, false).unwrap();
    assert!(set.is_dirty());
    assert!(selector.rebuild_sensitives_tree(&object, true).is_err());
    selector
        .remove_selection_of_object(&object, &selection)
        .unwrap();
    assert_eq!(set.size(), 0);
    selector.remove_selectable_object(&object).unwrap();
    selector.rebuild_sensitives_tree(&object, true).unwrap();
    selector
        .remove_selection_of_object(&object, &Handle::default())
        .unwrap();
}
#[test]
fn activation_reset_preserves_mode_state_and_owner_output_keeps_duplicates() {
    let selector = SelectMgrViewerSelector::new();
    let object = BoxedSelectable::new(0., false).cast();
    let selection = mode_selection(&object, 0);
    selection.entities()[0].set_active_for_selection();
    selection.set_selection_state(SelectMgrStateOfSelection::Activated);
    selector
        .add_selection_to_object(&object, &selection)
        .unwrap();
    let mut owners = List::default();
    owners.append(Handle::default());
    selector.active_owners(&mut owners).unwrap();
    assert_eq!(owners.len(), 2);
    selector.active_owners(&mut owners).unwrap();
    assert_eq!(owners.len(), 3);
    selector.reset_selection_activation_status().unwrap();
    selector.active_owners(&mut owners).unwrap();
    assert_eq!(owners.len(), 3);
    assert_eq!(
        selection.get_selection_state(),
        SelectMgrStateOfSelection::Activated
    );
}
#[test]
fn selector_volume_and_object_set_are_the_actual_owned_instances() {
    let selector = SelectMgrViewerSelector::new();
    let object = BoxedSelectable::new(0., false).cast();
    selector.add_selectable_object(&object).unwrap();
    assert!(selector.selectable_objects().contains(&object));
    selector.move_selectable_object(&object).unwrap();
    selector
        .get_manager()
        .init_axis_selecting_volume(Axis1::default());
    selector.get_manager().build_selecting_volume().unwrap();
    selector.allow_overlap_detection(false);
    selector.rebuild_objects_tree(false).unwrap();
    selector.rebuild_objects_tree(true).unwrap();
}
#[test]
fn repeated_prebuild_enable_replaces_the_pool_and_disabled_queue_is_a_no_op() {
    let selector = SelectMgrViewerSelector::new();
    selector.queue_bvh_build(&Handle::default()).unwrap();
    selector.wait_for_bvh_build();
    selector.set_to_prebuild_bvh(true, 0);
    let first = selector
        .viewer_selector_base()
        .fields
        .read()
        .bvh_thread_pool
        .clone();
    assert_eq!(first.threads().len(), 1);
    selector.set_to_prebuild_bvh(true, 2);
    assert_ne!(
        first,
        selector
            .viewer_selector_base()
            .fields
            .read()
            .bvh_thread_pool
    );
    let job = Handle::new(bvh_pool_fixture::PoolBuildJob::new(0, 0));
    selector.queue_bvh_build(&job.clone().cast()).unwrap();
    selector.wait_for_bvh_build();
    selector.set_to_prebuild_bvh(false, 2);
    assert!(!selector.to_prebuild_bvh());
    assert_eq!(job.calls.load(Ordering::Acquire), 1);
}

#[test]
fn selector_dump_preserves_native_children_counts_and_null_pointer_fields() {
    use rustycad_tkernel::standard::dump::DumpStream;
    let selector = SelectMgrViewerSelector::new();
    let object = BoxedSelectable::new(0., false).cast();
    selector.add_selectable_object(&object).unwrap();
    selector.set_entity_set_builder(None);
    selector
        .viewer_selector_base()
        .fields
        .write()
        .debug_structures
        .append(Handle::default());
    let mut shallow = DumpStream::default();
    selector.dump_json(&mut shallow, 0).unwrap();
    assert!(!shallow.text().contains("CameraEye"));
    assert!(shallow.text().contains("\"NbOfSelectableObjects\": 1"));
    assert!(shallow.text().contains("\"EntitySetBuilder\": \"\""));
    assert!(shallow.text().contains("\"Structure\": \"\""));
    let mut nested = DumpStream::default();
    selector.dump_json(&mut nested, -1).unwrap();
    assert!(nested.text().contains("\"CameraEye\": {"));
    assert!(nested.text().contains("\"SelectingVolumeMgr\": {"));
    assert!(nested.text().contains("\"CameraScale\": 1"));
}
