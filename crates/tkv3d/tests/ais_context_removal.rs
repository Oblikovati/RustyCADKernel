//! Bulk operations preserve native traversal, callback order and removal state.
#[path = "support/context_object.rs"]
mod context_object;
#[path = "support/lifecycle_object.rs"]
mod lifecycle_object;
use context_object::ContextObject;
use lifecycle_object::LifecycleObject;
use rustycad_tkernel::{ncollection::List, standard::Handle};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::*};
fn context() -> Handle<AisInteractiveContext> {
    AisInteractiveContext::new(&V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap()).unwrap()
}
#[test]
fn bulk_display_and_erase_keep_membership_and_selection_modes() {
    let context = context();
    let objects = [ContextObject::new(), ContextObject::new()];
    for object in &objects {
        context.load(&object.cast(), 0).unwrap();
    }
    context.display_all(false).unwrap();
    for object in &objects {
        assert!(context.is_displayed(&object.cast()));
        assert!(
            context
                .selection_manager()
                .is_activated(&object.cast(), 0)
                .unwrap()
        );
    }
    context.erase_all(false).unwrap();
    for object in &objects {
        assert_eq!(
            context.display_status(&object.cast()),
            PrsMgrDisplayStatus::Erased
        );
        assert_eq!(object.presentations().len(), 1);
    }
    context.display_all(false).unwrap();
    for object in &objects {
        assert_eq!(*object.events.read(), [100, 200]);
    }
}
#[test]
fn selected_display_uses_owner_order_and_erase_restarts_from_current_head() {
    let context = context();
    context.set_automatic_hilight(false);
    let objects = [ContextObject::new(), ContextObject::new()];
    for object in &objects {
        context.display(&object.cast(), false).unwrap();
        context.add_select_object(&object.cast()).unwrap();
    }
    context.display_selected(false).unwrap();
    assert_eq!(context.nb_selected(), 2);
    context.erase_selected(false).unwrap();
    assert_eq!(context.nb_selected(), 0);
    for object in &objects {
        assert_eq!(
            context.display_status(&object.cast()),
            PrsMgrDisplayStatus::Erased
        );
    }
    context.erase_selected(true).unwrap();
    context.display_selected(true).unwrap();
}
#[test]
fn permanent_removal_detaches_membership_affinity_and_presentations_then_can_redisplay() {
    let context = context();
    let object = ContextObject::new();
    context.display(&object.cast(), false).unwrap();
    context.add_select_object(&object.cast()).unwrap();
    object.view_affinity().set_visible(false);
    context.fields.write().last_picked = object.global_sel_owner();
    context.remove(&object.cast(), true).unwrap();
    assert!(!object.has_interactive_context());
    assert_eq!(
        context.display_status(&object.cast()),
        PrsMgrDisplayStatus::None
    );
    assert!(object.presentations().is_empty());
    assert!(!context.selection_manager().contains(&object.cast()));
    assert!(
        context
            .current_viewer()
            .structure_manager()
            .object_affinity(&object.cast())
            .is_null()
    );
    assert!(object.view_affinity().is_visible(0));
    assert_eq!(context.nb_selected(), 0);
    assert!(context.fields.read().last_picked.is_null());
    context.display(&object.cast(), false).unwrap();
    assert!(context.is_displayed(&object.cast()));
}
#[test]
fn direct_clear_global_keeps_context_and_remove_rejects_foreign_before_changes() {
    let context = context();
    let foreign = crate::context();
    let object = ContextObject::new();
    context.display(&object.cast(), false).unwrap();
    assert!(foreign.remove(&object.cast(), false).is_err());
    assert!(context.is_displayed(&object.cast()));
    context.clear_global(&object.cast(), false).unwrap();
    assert_eq!(object.get_context(), context);
    context.remove(&object.cast(), false).unwrap();
    assert!(!object.has_interactive_context());
    context.remove(&Handle::default(), true).unwrap();
    assert!(context.clear_global(&Handle::default(), true).is_err());
}
#[test]
fn clear_prs_erases_only_the_registered_mode_and_keeps_active_selection() {
    let context = context();
    let object = ContextObject::new();
    context
        .display_legacy(&object.cast(), 0, 0, false, true, PrsMgrDisplayStatus::None)
        .unwrap();
    context
        .display_with_modes(&object.cast(), 2, 1, false, PrsMgrDisplayStatus::None)
        .unwrap();
    context.clear_prs(&object.cast(), 0, false).unwrap();
    assert_eq!(object.presentations().len(), 2);
    context.clear_global_prs(&object.cast(), 2, false).unwrap();
    assert_eq!(object.presentations().len(), 1);
    assert!(context.is_displayed_mode(&object.cast(), 2));
    assert!(
        context
            .selection_manager()
            .is_activated(&object.cast(), 1)
            .unwrap()
    );
    context.display(&object.cast(), false).unwrap();
    assert!(context.is_displayed_mode(&object.cast(), 0));
    context.clear_prs(&Handle::default(), 0, true).unwrap();
}
#[test]
fn remove_all_clears_detection_and_detaches_registered_children() {
    let context = context();
    let parent = ContextObject::new();
    let child = ContextObject::new();
    parent.add_child(&child.cast()).unwrap();
    context.display(&parent.cast(), false).unwrap();
    context.fields.write().detected.append(9);
    context.fields.write().current_detected = 1;
    context.remove_all(true).unwrap();
    assert!(context.fields.read().objects.is_empty());
    assert!(context.fields.read().detected.is_empty());
    assert_eq!(context.fields.read().current_detected, 0);
    assert!(!parent.has_interactive_context());
    assert!(!child.has_interactive_context());
    context.remove_all(false).unwrap();
}
#[test]
fn query_overloads_append_and_distinguish_status_none_from_unfiltered() {
    let context = context();
    let object = LifecycleObject::new();
    context.display(&object.cast(), false).unwrap();
    let mut output = List::default();
    output.append(Handle::<dyn AisInteractiveObject>::default());
    context
        .objects_by_display_status(PrsMgrDisplayStatus::None, &mut output)
        .unwrap();
    assert_eq!(output.len(), 1);
    context
        .objects_by_display_status_and_kind(
            AisKindOfInteractive::Shape,
            7,
            PrsMgrDisplayStatus::None,
            &mut output,
        )
        .unwrap();
    assert_eq!(output.len(), 2);
    context
        .objects_inside(&mut output, AisKindOfInteractive::None, 7)
        .unwrap();
    assert_eq!(output.len(), 2);
    context
        .objects_inside(&mut output, AisKindOfInteractive::None, -1)
        .unwrap();
    assert_eq!(output.len(), 3);
    context
        .displayed_objects_by_kind(AisKindOfInteractive::Shape, -1, &mut output)
        .unwrap();
    assert_eq!(output.len(), 4);
    context
        .erased_objects_by_kind(AisKindOfInteractive::Shape, 7, &mut output)
        .unwrap();
    assert_eq!(output.len(), 4);
    context.erase(&object.cast(), false).unwrap();
    context.erased_objects(&mut output).unwrap();
    assert_eq!(output.len(), 5);
    context.displayed_objects(&mut output).unwrap();
    assert_eq!(output.len(), 5);
    assert!(output.first().unwrap().is_null());
}
#[test]
fn query_bucket_order_matches_native_map_without_clearing_existing_prefix() {
    let context = context();
    for _ in 0..16 {
        context.load(&ContextObject::new().cast(), -1).unwrap();
    }
    let expected: Vec<_> = context
        .fields
        .read()
        .objects
        .items()
        .map(|(key, _)| key.clone())
        .collect();
    let mut output = List::default();
    context
        .objects_inside(&mut output, AisKindOfInteractive::None, -1)
        .unwrap();
    assert_eq!(output.iter().cloned().collect::<Vec<_>>(), expected);
    context
        .objects_inside(&mut output, AisKindOfInteractive::None, -1)
        .unwrap();
    assert_eq!(output.len(), 32);
}
#[test]
fn objects_for_view_includes_other_states_before_considering_affinity() {
    let context = context();
    let view = context.current_viewer().create_view().unwrap();
    let object = ContextObject::new();
    context.load(&object.cast(), 0).unwrap();
    object.view_affinity().set_visible(false);
    let mut output = List::default();
    context
        .objects_for_view(&mut output, &view, true, PrsMgrDisplayStatus::Displayed)
        .unwrap();
    assert_eq!(output.len(), 1);
    output.clear();
    context
        .objects_for_view(&mut output, &view, true, PrsMgrDisplayStatus::None)
        .unwrap();
    assert!(output.is_empty());
}
#[test]
fn reentrant_kind_callback_sees_changes_in_later_buckets() {
    use rustycad_tkernel::ncollection::HashKey;
    let context = context();
    let mut candidates: Vec<_> = (0..32).map(|_| LifecycleObject::new()).collect();
    candidates.sort_by_key(|object| object.cast::<dyn AisInteractiveObject>().occt_hash() % 101);
    let first = candidates.first().unwrap().clone();
    let last = candidates.last().unwrap().clone();
    assert_ne!(
        first.cast::<dyn AisInteractiveObject>().occt_hash() % 101,
        last.cast::<dyn AisInteractiveObject>().occt_hash() % 101
    );
    context.load(&first.cast(), -1).unwrap();
    let appended = last.clone();
    *first.on_kind.write() = Some(Box::new(move |object| {
        object.get_context().load(&appended.cast(), -1).unwrap()
    }));
    let mut output = List::default();
    context
        .objects_inside(&mut output, AisKindOfInteractive::Shape, 7)
        .unwrap();
    assert_eq!(
        output.iter().cloned().collect::<Vec<_>>(),
        vec![
            first.cast::<dyn AisInteractiveObject>(),
            last.cast::<dyn AisInteractiveObject>()
        ]
    );
}
#[test]
fn display_all_releases_context_guard_before_computation_callbacks() {
    let context = context();
    let object = LifecycleObject::new();
    context.load(&object.cast(), -1).unwrap();
    *object.on_compute.write() = Some(Box::new(|object| {
        object.get_context().set_auto_activate_selection(false)
    }));
    context.display_all(false).unwrap();
    assert!(!context.get_auto_activate_selection());
    object.set_auto_hilight(false);
    *object.on_clear.write() = Some(Box::new(|object| {
        object.get_context().set_automatic_hilight(false)
    }));
    context.erase_all(false).unwrap();
    assert!(!context.automatic_hilight());
}
#[test]
fn deleting_a_future_binding_during_a_query_uses_updated_bucket_links() {
    let context = context();
    let objects: Vec<_> = (0..16).map(|_| LifecycleObject::new()).collect();
    for object in &objects {
        context.load(&object.cast(), -1).unwrap();
    }
    let order: Vec<_> = context
        .fields
        .read()
        .objects
        .items()
        .map(|(key, _)| key.clone())
        .collect();
    let first = order[0].cast::<LifecycleObject>();
    let last = order.last().unwrap().clone();
    *first.on_kind.write() = Some(Box::new(move |object| {
        object.get_context().remove(&last, false).unwrap()
    }));
    let mut output = List::default();
    context
        .objects_inside(&mut output, AisKindOfInteractive::Shape, -1)
        .unwrap();
    assert_eq!(
        output.iter().cloned().collect::<Vec<_>>(),
        order[..order.len() - 1]
    );
}
#[test]
fn invalidated_native_map_traversal_reports_safe_failure_instead_of_stale_memory() {
    let context = context();
    let object = LifecycleObject::new();
    context.load(&object.cast(), -1).unwrap();
    *object.on_kind.write() = Some(Box::new(|object| {
        object.get_context().fields.write().objects.resize(1000)
    }));
    let mut output = List::default();
    let error = context
        .objects_inside(&mut output, AisKindOfInteractive::Shape, 7)
        .unwrap_err();
    assert!(error.message().contains("expected live node"));
    assert_eq!(output.len(), 1);
}
#[test]
fn remove_all_uses_native_snapshot_while_callbacks_register_new_objects() {
    let context = context();
    let original = LifecycleObject::new();
    let added = ContextObject::new();
    original.set_auto_hilight(false);
    context.display(&original.cast(), false).unwrap();
    context.add_select_object(&original.cast()).unwrap();
    let weak = context.downgrade();
    let inserted = added.clone();
    *original.on_clear.write() = Some(Box::new(move |_| {
        weak.upgrade().load(&inserted.cast(), -1).unwrap()
    }));
    context.remove_all(false).unwrap();
    assert_eq!(context.fields.read().objects.len(), 1);
    assert_eq!(
        context.display_status(&added.cast()),
        PrsMgrDisplayStatus::Erased
    );
    assert!(!original.has_interactive_context());
}
#[test]
fn query_callbacks_observe_no_temporary_native_owning_references() {
    use rustycad_tkernel::standard::Transient;
    let context = context();
    let object = LifecycleObject::new();
    context.load(&object.cast(), -1).unwrap();
    let expected = object.transient_base().ref_count();
    *object.on_kind.write() = Some(Box::new(move |object| {
        assert_eq!(object.transient_base().ref_count(), expected)
    }));
    let mut output = List::default();
    context
        .objects_inside(&mut output, AisKindOfInteractive::Shape, 7)
        .unwrap();
    assert_eq!(object.transient_base().ref_count(), expected + 1);
}
