//! Native identity-chain policy, absolute transforms and active selection synchronization.
#[path = "support/context_object.rs"]
mod context_object;
#[path = "support/location_owner.rs"]
mod location_owner;
use context_object::ContextObject;
use location_owner::LocationOwner;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::*};
fn location_context() -> Handle<AisInteractiveContext> {
    AisInteractiveContext::new(&V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap()).unwrap()
}
fn translated_location(x: f64) -> Location {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(x, 0.0, 0.0));
    Location::new(transform)
}
fn absolute_offset(
    context: &AisInteractiveContext,
    object: &Handle<dyn AisInteractiveObject>,
) -> f64 {
    context
        .location(object)
        .unwrap()
        .transformation()
        .translation_part()
        .x()
}
#[test]
fn null_location_updates_are_noops_but_query_rejects_null() {
    let context = location_context();
    let object = Handle::default();
    context
        .set_location(&object, &translated_location(2.0))
        .unwrap();
    context.reset_location(&object).unwrap();
    assert!(!context.has_location(&object));
    assert!(context.location(&object).is_err());
}
#[test]
fn empty_identity_preserves_datum() {
    let context = location_context();
    let fixture = ContextObject::new();
    let object = fixture.cast();
    fixture
        .set_local_transformation(&Transform3::default())
        .unwrap();
    let original = fixture.local_transformation_geom();
    context.set_location(&object, &Location::default()).unwrap();
    assert_eq!(original, fixture.local_transformation_geom());
    assert!(!context.has_location(&object));
    assert!(!context.location(&object).unwrap().is_identity());
}
#[test]
fn identity_datum_replaces_prior_datum_and_reset_clears_it() {
    let context = location_context();
    let fixture = ContextObject::new();
    let object = fixture.cast();
    fixture
        .set_local_transformation(&Transform3::default())
        .unwrap();
    let original = fixture.local_transformation_geom();
    let identity = Location::new(Transform3::default());
    context.set_location(&object, &identity).unwrap();
    assert_ne!(original, fixture.local_transformation_geom());
    context.reset_location(&object).unwrap();
    assert!(fixture.local_transformation_geom().is_null());
}
#[test]
fn location_queries_absolute_parent_transform_after_local_reset() {
    let context = location_context();
    let parent = ContextObject::new();
    let object = ContextObject::new();
    parent
        .set_local_transformation(&translated_location(10.0).transformation())
        .unwrap();
    parent.add_child(&object.cast()).unwrap();
    context
        .set_location(&object.cast(), &translated_location(3.0))
        .unwrap();
    assert_eq!(absolute_offset(&context, &object.cast()), 13.0);
    context.reset_location(&object.cast()).unwrap();
    assert!(context.has_location(&object.cast()));
    assert_eq!(absolute_offset(&context, &object.cast()), 10.0);
}
#[test]
fn consecutive_locations_replace_instead_of_compose() {
    let context = location_context();
    let object = ContextObject::new().cast();
    context
        .set_location(&object, &translated_location(3.0))
        .unwrap();
    context
        .set_location(&object, &translated_location(2.0))
        .unwrap();
    assert_eq!(absolute_offset(&context, &object), 2.0);
}
fn assert_selection_clean(object: &ContextObject) {
    let selection = object.selection(0);
    assert_eq!(selection.update_status(), SelectMgrTypeOfUpdate::None);
    assert_eq!(
        selection.bvh_update_status(),
        SelectMgrTypeOfBvhUpdate::None
    );
    assert!(object.events.read().is_empty());
}
#[test]
fn active_selection_updates_without_recomputing_geometry() {
    let context = location_context();
    let object = ContextObject::new();
    context.display(&object.cast(), false).unwrap();
    object.events.write().clear();
    context
        .set_location(&object.cast(), &translated_location(4.0))
        .unwrap();
    assert_selection_clean(&object);
    context.reset_location(&object.cast()).unwrap();
    assert_selection_clean(&object);
}
#[test]
fn selection_failure_preserves_transform_and_skips_highlight() {
    let context = location_context();
    let object = ContextObject::new();
    context.display(&object.cast(), false).unwrap();
    let owner = LocationOwner::new(&object.cast(), &context);
    context.fields.write().last_picked = owner.cast();
    let manager = SelectMgrSelectionManager::new(&Handle::default());
    context.fields.write().selection_manager = Handle::new(manager);
    assert!(
        context
            .set_location(&object.cast(), &translated_location(4.0))
            .is_err()
    );
    assert_eq!(absolute_offset(&context, &object.cast()), 4.0);
    assert!(owner.modes.read().is_empty());
}
#[test]
fn highlight_callback_reenters_context_and_identity_reset_skips_it() {
    let context = location_context();
    let fixture = ContextObject::new();
    let object = fixture.cast();
    fixture.set_hilight_mode(7);
    let owner = LocationOwner::new(&fixture.cast(), &context);
    context.fields.write().last_picked = owner.cast();
    context
        .set_location(&object, &translated_location(2.0))
        .unwrap();
    assert_eq!(*owner.modes.read(), vec![7]);
    context.set_location(&object, &Location::default()).unwrap();
    context.reset_location(&object).unwrap();
    assert_eq!(*owner.modes.read(), vec![7]);
}
#[test]
fn highlights_only_matching_selectable_and_defaults_mode_to_zero() {
    let context = location_context();
    let object = ContextObject::new();
    let owner = LocationOwner::new(&object.cast(), &context);
    context.fields.write().last_picked = owner.cast();
    let other = ContextObject::new().cast();
    context
        .set_location(&other, &translated_location(1.0))
        .unwrap();
    assert!(owner.modes.read().is_empty());
    let identity = Location::new(Transform3::default());
    context.set_location(&object.cast(), &identity).unwrap();
    assert_eq!(*owner.modes.read(), vec![0]);
}
