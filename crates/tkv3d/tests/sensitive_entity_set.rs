//! Indexed sensitive set behavior and shared BVH integration.
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkmath::{
    bvh::*,
    gp::{Axis2, Point3},
};
use rustycad_tkservice::graphic3d::{Graphic3dTransModeFlags, Graphic3dTransformPers};
use rustycad_tkv3d::{select_mgr::*, select3d::Select3DSensitivePoint};
use std::sync::Arc;
fn builder() -> BvhBuilderHandle<BvhVec3d> {
    Some(Arc::new(BvhBinnedBuilder::<BvhVec3d, 32>::new(
        1, 32, false, 1,
    )))
}
fn owner() -> Handle<dyn SelectMgrEntityOwnerSource> {
    SelectMgrEntityOwner::new(3).cast()
}
fn entity(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    x: f64,
) -> Handle<SelectMgrSensitiveEntity> {
    Handle::new(SelectMgrSensitiveEntity::new(
        &Handle::new(Select3DSensitivePoint::new(owner, Point3::new(x, 2., 3.))).cast(),
    ))
}
fn selection(entities: &[Handle<SelectMgrSensitiveEntity>]) -> Handle<SelectMgrSelection> {
    let selection = Handle::new(SelectMgrSelection::new(0));
    selection.change_entities(&mut |items| items.extend_from_slice(entities));
    selection
}
fn persistence(entity: &Handle<SelectMgrSensitiveEntity>, enabled: bool) {
    let transform = if enabled {
        Handle::new(
            Graphic3dTransformPers::with_point(
                Graphic3dTransModeFlags::ZOOM_PERS,
                &Point3::default(),
            )
            .unwrap(),
        )
        .cast()
    } else {
        Handle::default()
    };
    entity
        .base_sensitive()
        .set_transform_persistence(&transform);
    entity
        .base_sensitive()
        .set_flipping_options(enabled, &Axis2::default());
}
#[test]
fn runtime_base_casts_preserve_properties_and_dirty_state() {
    let set = Handle::new(SelectMgrSensitiveEntitySet::new(None));
    let base = set.cast::<dyn BvhObjectTransientSource>();
    assert!(!base.is_null());
    assert_eq!(base.cast::<SelectMgrSensitiveEntitySet>(), set);
    assert!(set.is_kind_name("BVH_ObjectTransient"));
    assert!(set.is_kind_name("Standard_Transient"));
    assert_eq!(
        SelectMgrSensitiveEntitySet::static_type()
            .parent()
            .unwrap()
            .name(),
        "BVH_ObjectTransient"
    );
    assert!(!base.is_dirty());
    assert_eq!(set.bvh().unwrap().read().length(), 0);
    let properties: Arc<dyn BvhProperties> = Arc::new(BvhTransform::<f64>::default());
    base.set_properties(Some(properties.clone()));
    assert!(Arc::ptr_eq(&base.properties().unwrap(), &properties));
    assert!(!base.is_dirty());
    base.mark_dirty();
    assert!(set.is_dirty());
    assert!(set.bvh().is_err());
    base.set_properties(None);
    assert!(base.properties().is_none());
}
#[test]
fn unique_map_counts_owners_and_removes_by_swap_last() {
    let set = SelectMgrSensitiveEntitySet::new(builder());
    let owner = owner();
    let (a, b, c) = (
        entity(&owner, 1.),
        entity(&owner, 2.),
        entity(&Handle::default(), 3.),
    );
    let initial = selection(&[a.clone(), b.clone(), c.clone(), a.clone()]);
    set.append_selection(&initial).unwrap();
    assert_eq!(set.size(), 3);
    assert_eq!(*set.owners().find(&owner).unwrap(), 2);
    set.remove(&selection(&[a])).unwrap();
    assert_eq!(set.get_sensitive_by_id(0).unwrap(), c);
    assert_eq!(*set.owners().find(&owner).unwrap(), 1);
    set.remove(&selection(&[b])).unwrap();
    assert!(set.owners().is_empty());
    assert_eq!(set.sensitives().len(), 1);
}
#[test]
fn duplicate_persistence_counters_survive_removal_and_use_current_flags() {
    let set = SelectMgrSensitiveEntitySet::new(builder());
    let a = entity(&owner(), 1.);
    persistence(&a, true);
    set.append(&a).unwrap();
    set.append(&a).unwrap();
    assert_eq!(set.size(), 1);
    assert!(set.has_entity_with_persistence() && set.has_entity_with_flipping());
    assert!(!set.element_box(0).unwrap().is_valid());
    assert_eq!(set.center(0, 0).unwrap(), 1.);
    set.remove(&selection(std::slice::from_ref(&a))).unwrap();
    assert_eq!(set.size(), 0);
    assert!(set.has_entity_with_persistence() && set.has_entity_with_flipping());
    persistence(&a, false);
    set.append(&a).unwrap();
    set.remove(&selection(&[a])).unwrap();
    assert!(set.has_entity_with_persistence());
}
#[test]
fn query_indices_and_owner_helpers_follow_native_rules() {
    let set = SelectMgrSensitiveEntitySet::new(builder());
    let owner = owner();
    set.add_owner(&owner);
    set.add_owner(&owner);
    set.add_owner(&Handle::default());
    assert_eq!(*set.owners().find(&owner).unwrap(), 2);
    set.remove_owner(&owner);
    set.remove_owner(&owner);
    set.remove_owner(&owner);
    set.remove_owner(&Handle::default());
    assert!(set.owners().is_empty());
    let a = entity(&owner, 7.);
    set.append(&a).unwrap();
    assert_eq!(set.element_box(0).unwrap().corner_min().x, 7.);
    for axis in [-1, 2, 3, 100] {
        assert_eq!(set.center(0, axis).unwrap(), 3.);
    }
    assert_eq!(set.center(0, 1).unwrap(), 2.);
    assert!(set.swap(-1, 0).is_err());
    assert!(set.get_sensitive_by_id(i32::MAX).is_err());
    assert!(set.element_box(2).is_err());
    set.swap(0, 0).unwrap();
    assert_eq!(set.get_sensitive_by_id(0).unwrap(), a);
}
#[test]
fn retained_tree_updates_while_builder_assignment_does_not_mark_dirty() {
    let set = SelectMgrSensitiveEntitySet::new(builder());
    let retained = set.bvh().unwrap();
    set.append(&entity(&owner(), 1.)).unwrap();
    set.bvh().unwrap();
    assert_eq!(retained.read().length(), 1);
    set.append(&entity(&owner(), 10.)).unwrap();
    assert!(set.bounding_box().unwrap().is_valid());
    let second = set.bvh().unwrap();
    assert!(retained.ptr_eq(&second));
    assert_eq!(retained.read().length(), 3);
    set.set_builder(None);
    assert!(!set.is_dirty());
    assert!(set.builder().is_none());
    assert!(set.bvh().is_ok());
    set.mark_dirty();
    assert!(set.bvh().is_err());
    set.set_builder(builder());
    set.bvh().unwrap();
    assert!(retained.ptr_eq(&set.bvh().unwrap()));
}
#[test]
fn empty_selection_marks_dirty_and_null_handles_fail_explicitly() {
    let set = SelectMgrSensitiveEntitySet::new(builder());
    assert!(!set.is_dirty());
    assert!(!set.bounding_box().unwrap().is_valid());
    set.append_selection(&selection(&[])).unwrap();
    assert!(set.is_dirty());
    set.bvh().unwrap();
    assert!(!set.is_dirty());
    set.remove(&selection(&[])).unwrap();
    assert!(set.is_dirty());
    assert!(set.append(&Handle::default()).is_err());
    assert!(set.append_selection(&Handle::default()).is_err());
    assert!(set.remove(&Handle::default()).is_err());
    let cleared = entity(&owner(), 2.);
    cleared.clear();
    assert!(set.append(&cleared).is_err());
}

#[path = "support/rejected_sensitive.rs"]
mod rejected_sensitive;
#[test]
fn rejected_runtime_type_deactivates_without_inserting() {
    let set = SelectMgrSensitiveEntitySet::new(builder());
    let fake = Handle::new(rejected_sensitive::RejectedSensitivePoint::new(
        &owner(),
        Point3::default(),
    ));
    let wrapper = Handle::new(SelectMgrSensitiveEntity::new(&fake.cast()));
    wrapper.set_active_for_selection();
    set.append(&wrapper).unwrap();
    assert_eq!(set.size(), 0);
    assert!(!wrapper.is_active_for_selection());
    assert!(!set.is_dirty());
    wrapper.set_active_for_selection();
    set.append_selection(&selection(std::slice::from_ref(&wrapper)))
        .unwrap();
    assert!(set.is_dirty());
    assert!(!wrapper.is_active_for_selection());
    assert!(set.owners().is_empty());
}
