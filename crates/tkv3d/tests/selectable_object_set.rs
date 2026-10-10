//! Subset transitions, live traversal and camera invalidation regressions.
#[path = "support/boxed_selectable.rs"]
mod boxed_selectable;
use SelectMgrBvhSubset as Subset;
use boxed_selectable::BoxedSelectable;
use rustycad_tkernel::standard::{Handle, dump::DumpStream};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{prs_mgr::PrsMgrPresentableObject, select_mgr::*};

fn object(offset: f64) -> Handle<dyn SelectMgrSelectableObject> {
    BoxedSelectable::new(offset, false).cast()
}
fn persistence(mode: i32) -> Handle<dyn Graphic3dTransformPersSource> {
    if mode == 0 {
        return Handle::default();
    }
    Handle::new(Graphic3dTransformPers::new(Graphic3dTransModeFlags(mode)).unwrap()).cast()
}
fn dump(set: &SelectMgrSelectableObjectSet) -> String {
    let mut stream = DumpStream::default();
    set.dump_json(&mut stream, -1).unwrap();
    String::from_utf8(stream.bytes().to_vec()).unwrap()
}
#[test]
fn empty_sets_preserve_regular_dirty_state_and_validate_subset_values() {
    let set = SelectMgrSelectableObjectSet::default();
    assert!(set.is_empty());
    for value in 0..5 {
        assert!(set.is_subset_empty(Subset::try_from(value).unwrap()));
    }
    assert!(Subset::try_from(-1).is_err());
    assert!(Subset::try_from(5).is_err());
    set.mark_dirty();
    set.update_bvh(&Handle::default(), [10, 20]).unwrap();
    assert_eq!(dump(&set).matches("\"IsDirty\": 1").count(), 5);
    set.update_bvh(&Handle::new(Graphic3dCamera::default()), [10, 20])
        .unwrap();
    assert_eq!(dump(&set).matches("\"IsDirty\": 1").count(), 1);
}
#[test]
fn unique_membership_swap_last_removal_and_null_contracts() {
    let set = SelectMgrSelectableObjectSet::new();
    let (a, b, c) = (object(0.), object(2.), object(4.));
    for entry in [&a, &b, &c] {
        assert!(set.append(entry).unwrap());
    }
    assert!(!set.append(&a).unwrap());
    assert!(set.remove(&a).unwrap());
    assert_eq!(set.get_object_by_id(Subset::ThreeD, 0).unwrap(), c);
    assert_eq!(set.get_object_by_id(Subset::ThreeD, 1).unwrap(), b);
    assert!(set.get_object_by_id(Subset::ThreeD, -1).is_err());
    assert!(set.get_object_by_id(Subset::ThreeD, i32::MAX).is_err());
    assert!(set.append(&Handle::default()).is_err());
    assert!(!set.remove(&Handle::default()).unwrap());
    set.change_subset(&Handle::default()).unwrap();
}
#[test]
fn persistence_transitions_follow_all_five_native_classes() {
    let set = SelectMgrSelectableObjectSet::new();
    let entry = object(0.);
    set.append(&entry).unwrap();
    for (index, mode) in [0, 2, 64, 258, 320].into_iter().enumerate() {
        entry.set_transform_persistence(&persistence(mode));
        set.change_subset(&entry).unwrap();
        assert_eq!(
            set.get_object_by_id(Subset::try_from(index as i32).unwrap(), 0)
                .unwrap(),
            entry
        );
        assert!(!set.append(&entry).unwrap());
    }
    assert!(set.remove(&entry).unwrap());
    assert!(set.is_empty());
}
#[test]
fn iterator_observes_live_swaps_and_requires_more_to_cross_subsets() {
    let set = SelectMgrSelectableObjectSet::new();
    let a = object(0.);
    let b = object(1.);
    let persistent = object(2.);
    persistent.set_transform_persistence(&persistence(64));
    for entry in [&a, &b, &persistent] {
        set.append(entry).unwrap();
    }
    let mut cursor = SelectMgrSelectableObjectSetIterator::new(&set);
    assert_eq!(cursor.value().unwrap(), a);
    set.remove(&a).unwrap();
    assert_eq!(cursor.value().unwrap(), b);
    cursor.next();
    assert!(cursor.value().is_err());
    assert!(cursor.more());
    assert_eq!(cursor.value().unwrap(), persistent);
    cursor.next();
    assert!(!cursor.more());
    cursor.init(&set);
    assert_eq!(cursor.value().unwrap(), b);
    let mut empty = SelectMgrSelectableObjectSetIterator::default();
    empty.next();
    assert!(!empty.more());
    assert!(empty.value().is_err());
}
#[test]
fn retained_regular_tree_is_rebuilt_in_place_and_kept_when_empty() {
    let set = SelectMgrSelectableObjectSet::new();
    let entry = object(3.);
    let retained = set.bvh(Subset::ThreeD);
    assert_eq!(retained.read().length(), 0);
    set.append(&entry).unwrap();
    set.update_bvh(&Handle::default(), [1, 1]).unwrap();
    assert!(retained.ptr_eq(&set.bvh(Subset::ThreeD)));
    assert_eq!(retained.read().min_point(0).unwrap().x, 3.);
    entry.cast::<BoxedSelectable>().set_bounds(9., false);
    set.mark_dirty();
    set.update_bvh(&Handle::default(), [1, 1]).unwrap();
    assert_eq!(retained.read().min_point(0).unwrap().x, 9.);
    set.remove(&entry).unwrap();
    set.update_bvh(&Handle::default(), [1, 1]).unwrap();
    assert_eq!(retained.read().length(), 1);
    drop(set);
    assert_eq!(retained.read().max_point(0).unwrap().x, 10.);
}
#[test]
fn camera_orientation_only_rebuilds_world_space_persistent_subsets() {
    let set = SelectMgrSelectableObjectSet::new();
    let entries: Vec<_> = [2, 64, 258, 320]
        .into_iter()
        .map(|mode| {
            let entry = BoxedSelectable::new(1., false);
            entry.set_transform_persistence(&persistence(mode));
            set.append(&entry.clone().cast()).unwrap();
            entry
        })
        .collect();
    let camera = Handle::new(Graphic3dCamera::default());
    set.update_bvh(&camera, [800, 600]).unwrap();
    assert!(entries.iter().all(|entry| entry.bounds_calls() == 1));
    set.update_bvh(&camera, [800, 600]).unwrap();
    assert!(entries.iter().all(|entry| entry.bounds_calls() == 1));
    camera.set_eye(&Point3::new(4., 5., 10.)).unwrap();
    set.update_bvh(&camera, [800, 600]).unwrap();
    assert_eq!(
        entries.iter().map(|e| e.bounds_calls()).collect::<Vec<_>>(),
        vec![2, 1, 2, 1]
    );
    set.update_bvh(&camera, [801, 600]).unwrap();
    assert_eq!(
        entries.iter().map(|e| e.bounds_calls()).collect::<Vec<_>>(),
        vec![3, 2, 3, 2]
    );
}

#[path = "support/boxed_selectable_scene.rs"]
mod boxed_selectable_scene;
use boxed_selectable_scene::*;
#[test]
fn group_local_transforms_are_excluded_from_flipping_bounds() {
    let set = SelectMgrSelectableObjectSet::new();
    let entry = BoxedSelectable::new(0., true);
    let manager = boxed_scene_manager();
    boxed_scene_groups(&manager, &entry, 1).unwrap();
    set.append(&entry.clone().cast()).unwrap();
    assert!(!set.is_subset_empty(Subset::ThreeDPersistent));
    let camera = Handle::new(Graphic3dCamera::default());
    set.update_bvh(&camera, [800, 600]).unwrap();
    let tree = set.bvh(Subset::ThreeDPersistent);
    let before = (
        *tree.read().min_point(0).unwrap(),
        *tree.read().max_point(0).unwrap(),
    );
    assert!(
        before.0.x.abs() < 100. && before.1.x.abs() < 100.,
        "bounds: {before:?}"
    );
    boxed_group_transform(&entry, 17).unwrap();
    set.mark_dirty();
    set.update_bvh(&camera, [800, 600]).unwrap();
    assert_eq!(
        before,
        (
            *tree.read().min_point(0).unwrap(),
            *tree.read().max_point(0).unwrap()
        )
    );
}
#[test]
fn null_camera_updates_viewport_without_rebuilding_persistent_objects() {
    let set = SelectMgrSelectableObjectSet::new();
    let entry = BoxedSelectable::new(1., false);
    entry.set_transform_persistence(&boxed_persistence(2).unwrap());
    set.append(&entry.clone().cast()).unwrap();
    let camera = Handle::new(Graphic3dCamera::default());
    set.update_bvh(&camera, [800, 600]).unwrap();
    set.update_bvh(&Handle::default(), [900, 700]).unwrap();
    set.update_bvh(&camera, [900, 700]).unwrap();
    assert_eq!(entry.bounds_calls(), 1);
    camera.set_scale(20.);
    set.update_bvh(&camera, [900, 700]).unwrap();
    assert_eq!(entry.bounds_calls(), 2);
}
#[test]
fn persistent_failure_keeps_dirty_flags_and_partial_update_order() {
    let set = SelectMgrSelectableObjectSet::new();
    let ordinary = object(0.);
    let failing = BoxedSelectable::new(1., false);
    failing.set_transform_persistence(&boxed_persistence(10).unwrap());
    set.append(&ordinary).unwrap();
    set.append(&failing.clone().cast()).unwrap();
    set.mark_dirty();
    let camera = Handle::new(Graphic3dCamera::default());
    assert_eq!(
        set.update_bvh(&camera, [800, 600])
            .unwrap_err()
            .exception_type(),
        "Standard_ProgramError"
    );
    assert_eq!(set.bvh(Subset::ThreeD).read().length(), 1);
    assert_eq!(set.bvh(Subset::ThreeDPersistent).read().length(), 0);
    assert_eq!(dump(&set).matches("\"IsDirty\": 1").count(), 4);
    failing.set_transform_persistence(&boxed_persistence(1).unwrap());
    set.update_bvh(&camera, [800, 600]).unwrap();
    assert_eq!(dump(&set).matches("\"IsDirty\": 1").count(), 0);
}
#[test]
fn iterator_initialized_empty_does_not_revisit_preceding_subsets() {
    let set = SelectMgrSelectableObjectSet::new();
    let mut cursor = SelectMgrSelectableObjectSetIterator::new(&set);
    assert!(!cursor.more());
    let entry = object(0.);
    set.append(&entry).unwrap();
    assert!(!cursor.more());
    cursor.init(&set);
    assert!(cursor.more());
    assert_eq!(cursor.value().unwrap(), entry);
}
#[test]
fn object_set_dump_ignores_depth() {
    let set = SelectMgrSelectableObjectSet::new();
    set.append(&object(0.)).unwrap();
    let expected = dump(&set);
    for depth in [0, 1, 8] {
        let mut stream = DumpStream::default();
        set.dump_json(&mut stream, depth).unwrap();
        assert_eq!(stream.bytes(), expected.as_bytes());
    }
}

#[test]
fn copied_sets_keep_independent_maps_and_share_the_native_tree_handles() {
    let set = SelectMgrSelectableObjectSet::new();
    let (a, b) = (object(0.), object(10.));
    set.append(&a).unwrap();
    set.append(&b).unwrap();
    set.update_bvh(&Handle::default(), [1, 1]).unwrap();
    let copy = set.clone();
    let retained = set.bvh(Subset::ThreeD);
    assert!(retained.ptr_eq(&copy.bvh(Subset::ThreeD)));
    copy.remove(&a).unwrap();
    copy.update_bvh(&Handle::default(), [1, 1]).unwrap();
    assert!(set.contains(&a));
    assert!(!copy.contains(&a));
    assert_eq!(retained.read().min_point(0).unwrap().x, 10.);
    let mut first = SelectMgrSelectableObjectSetIterator::new(&set);
    let second = first;
    first.next();
    assert_eq!(second.value().unwrap(), a);
    assert_eq!(first.value().unwrap(), b);
}
