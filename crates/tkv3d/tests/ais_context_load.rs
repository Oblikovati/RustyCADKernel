//! Load must prepare membership without computing or activating any selection mode.
#[path = "support/context_object.rs"]
mod context_object;
use context_object::ContextObject;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::*};
use std::sync::atomic::Ordering;
fn load_context() -> Handle<AisInteractiveContext> {
    AisInteractiveContext::new(&V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap()).unwrap()
}
fn load_status(
    context: &AisInteractiveContext,
    object: &Handle<ContextObject>,
) -> Handle<AisGlobalStatus> {
    context
        .fields
        .read()
        .objects
        .seek(&object.cast())
        .cloned()
        .unwrap()
}
#[test]
fn first_load_resets_affinity_and_registers_without_computing() {
    let context = load_context();
    let object = ContextObject::new();
    object.view_affinity().set_visible(false);
    object.set_global_sel_mode(3);
    context.load(&object.cast(), -1).unwrap();
    assert_eq!(object.get_context(), context);
    assert_eq!(
        context.display_status(&object.cast()),
        PrsMgrDisplayStatus::Erased
    );
    assert!(load_status(&context, &object).is_s_mode_in(3));
    assert!(context.selection_manager().contains(&object.cast()));
    assert!(context.main_selector().contains(&object.cast()));
    assert!(object.selections().is_empty());
    assert!(object.presentations().is_empty());
    assert!(object.events.read().is_empty());
    assert!(object.view_affinity().is_visible(1));
    assert_eq!(
        context
            .current_viewer()
            .structure_manager()
            .object_affinity(&object.cast()),
        object.view_affinity()
    );
}
#[test]
fn repeated_and_legacy_load_preserve_mode_status_and_affinity() {
    let context = load_context();
    let object = ContextObject::new();
    context.load_legacy(&object.cast(), -2, true).unwrap();
    let original = load_status(&context, &object);
    object.set_display_status(PrsMgrDisplayStatus::Displayed);
    object.view_affinity().set_view_visible(2, false);
    context.load_legacy(&object.cast(), 4, false).unwrap();
    assert_eq!(load_status(&context, &object), original);
    assert!(original.is_s_mode_in(-2));
    assert!(!original.is_s_mode_in(4));
    assert!(context.is_displayed(&object.cast()));
    assert!(context.is_displayed_mode(&object.cast(), original.display_mode()));
    assert!(!context.is_displayed_mode(&object.cast(), 99));
    assert!(!object.view_affinity().is_visible(2));
}
#[test]
fn defaults_honor_acceptance_explicit_modes_and_null_outputs() {
    let context = load_context();
    let object = ContextObject::new();
    let (mut display, mut highlight, mut selection) = (7, 8, 9);
    context.get_def_modes(
        &Handle::default(),
        &mut display,
        &mut highlight,
        &mut selection,
    );
    assert_eq!((display, highlight, selection), (7, 8, 9));
    context.default_drawer().set_display_mode(4);
    object.accepted_mode.store(2, Ordering::Release);
    context.get_def_modes(&object.cast(), &mut display, &mut highlight, &mut selection);
    assert_eq!((display, highlight, selection), (0, 0, 0));
    object.set_display_mode(2);
    object.set_hilight_mode(8);
    object.set_global_sel_mode(6);
    context.get_def_modes(&object.cast(), &mut display, &mut highlight, &mut selection);
    assert_eq!((display, highlight, selection), (2, 8, 6));
}
#[test]
fn child_status_propagates_even_without_visual_propagation() {
    let context = load_context();
    let parent = ContextObject::new();
    let child = ContextObject::new();
    parent.add_child(&child.cast()).unwrap();
    parent.set_propagate_visual_state(false);
    context.load(&parent.cast(), 5).unwrap();
    assert_eq!(child.get_context(), context);
    assert_eq!(
        context.display_status(&child.cast()),
        PrsMgrDisplayStatus::Erased
    );
    assert!(load_status(&context, &child).is_s_mode_in(5));
    assert!(context.selection_manager().contains(&child.cast()));
    assert!(
        context
            .current_viewer()
            .structure_manager()
            .object_affinity(&child.cast())
            .is_null()
    );
    context
        .set_object_status(&parent.cast(), PrsMgrDisplayStatus::None, -1, -1)
        .unwrap();
    assert!(context.fields.read().objects.is_empty());
    assert!(context.selection_manager().contains(&child.cast()));
}
#[test]
fn foreign_context_rejection_preserves_partial_child_assignment() {
    let context = load_context();
    let foreign = load_context();
    let parent = ContextObject::new();
    let child = ContextObject::new();
    child.set_context(&foreign);
    parent.add_child(&child.cast()).unwrap();
    assert!(context.load(&parent.cast(), 0).is_err());
    assert_eq!(parent.get_context(), context);
    assert_eq!(child.get_context(), foreign);
    assert!(context.fields.read().objects.is_empty());
    assert!(!context.selection_manager().contains(&parent.cast()));
}
#[test]
fn null_and_unregistered_objects_are_not_displayed_or_assigned_affinity() {
    let context = load_context();
    let object = ContextObject::new();
    object.set_display_status(PrsMgrDisplayStatus::Displayed);
    context.load(&Handle::default(), 1).unwrap();
    assert!(!context.is_displayed(&object.cast()));
    assert!(!context.is_displayed_mode(&Handle::default(), 0));
    context
        .set_view_affinity(&object.cast(), &Handle::default(), false)
        .unwrap();
    context
        .set_view_affinity(&Handle::default(), &Handle::default(), false)
        .unwrap();
    assert!(object.view_affinity().is_visible(0));
    assert!(
        context
            .set_object_status(&Handle::default(), PrsMgrDisplayStatus::None, 0, 0)
            .is_err()
    );
}
#[test]
fn registered_view_affinity_changes_only_the_requested_view() {
    let context = load_context();
    let object = ContextObject::new();
    context.load(&object.cast(), 0).unwrap();
    let first = context.current_viewer().create_view().unwrap();
    let second = context.current_viewer().create_view().unwrap();
    context
        .set_view_affinity(&object.cast(), &first, false)
        .unwrap();
    assert!(
        !object
            .view_affinity()
            .is_visible(first.get().unwrap().view().identification())
    );
    assert!(
        object
            .view_affinity()
            .is_visible(second.get().unwrap().view().identification())
    );
    context
        .set_view_affinity(&object.cast(), &first, true)
        .unwrap();
    assert!(
        object
            .view_affinity()
            .is_visible(first.get().unwrap().view().identification())
    );
    assert!(
        context
            .set_view_affinity(&object.cast(), &Handle::default(), false)
            .is_err()
    );
}
