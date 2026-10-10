//! Public AIS lifecycle retains geometry, selection membership and native failure state.
#[path = "support/context_object.rs"]
mod context_object;
use context_object::ContextObject;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::*};
use std::sync::atomic::Ordering;
fn display_fixture() -> (Handle<AisInteractiveContext>, Handle<ContextObject>) {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    (
        AisInteractiveContext::new(&viewer).unwrap(),
        ContextObject::new(),
    )
}
fn retained_mode(
    context: &AisInteractiveContext,
    object: &Handle<ContextObject>,
    mode: i32,
) -> Handle<PrsMgrPresentation> {
    let presentations = context.main_prs_mgr().clone();
    presentations
        .presentation(&object.cast(), mode, false, &Handle::default())
        .unwrap()
}
#[test]
fn display_erase_and_redisplay_reuse_geometry_and_reactivate_selection() {
    let (context, object) = display_fixture();
    context.display(&object.cast(), false).unwrap();
    let presentation = retained_mode(&context, &object, 0);
    assert!(context.is_displayed_mode(&object.cast(), 0));
    assert!(
        context
            .selection_manager()
            .is_activated(&object.cast(), 0)
            .unwrap()
    );
    context.erase(&object.cast(), false).unwrap();
    assert!(!presentation.is_visible());
    assert!(context.selection_manager().contains(&object.cast()));
    assert!(
        !context
            .selection_manager()
            .is_activated(&object.cast(), 0)
            .unwrap()
    );
    context.display(&object.cast(), false).unwrap();
    assert_eq!(retained_mode(&context, &object, 0), presentation);
    assert!(presentation.is_visible());
    assert_eq!(*object.events.read(), [100, 200]);
}
#[test]
fn explicit_modes_hide_old_presentations_without_changing_object_defaults() {
    let (context, object) = display_fixture();
    context.display(&object.cast(), false).unwrap();
    context
        .display_with_modes(&object.cast(), 2, 1, false, PrsMgrDisplayStatus::None)
        .unwrap();
    assert!(!retained_mode(&context, &object, 0).is_visible());
    assert!(retained_mode(&context, &object, 2).is_visible());
    assert_eq!(object.display_mode(), -1);
    assert!(context.is_displayed_mode(&object.cast(), 2));
    assert!(
        context
            .selection_manager()
            .is_activated(&object.cast(), 0)
            .unwrap()
    );
    assert!(
        context
            .selection_manager()
            .is_activated(&object.cast(), 1)
            .unwrap()
    );
    context.erase(&object.cast(), false).unwrap();
    assert!(
        !context
            .selection_manager()
            .is_activated(&object.cast(), -1)
            .unwrap()
    );
    assert_eq!(object.selections().len(), 2);
}
#[test]
fn erased_display_loads_without_computing_and_later_uses_default_mode() {
    let (context, object) = display_fixture();
    context
        .display_with_modes(&object.cast(), 7, 1, false, PrsMgrDisplayStatus::Erased)
        .unwrap();
    assert_eq!(
        context.display_status(&object.cast()),
        PrsMgrDisplayStatus::Erased
    );
    assert!(object.events.read().is_empty());
    assert!(object.presentations().is_empty());
    assert!(object.selections().is_empty());
    context.display(&object.cast(), false).unwrap();
    assert!(context.is_displayed_mode(&object.cast(), 0));
    assert_eq!(*object.events.read(), [100, 200]);
}
#[test]
fn disabling_automatic_selection_still_displays_and_explicit_modes_override_it() {
    let (context, object) = display_fixture();
    context.set_auto_activate_selection(false);
    context.display(&object.cast(), false).unwrap();
    assert!(context.is_displayed(&object.cast()));
    assert!(!context.selection_manager().contains(&object.cast()));
    context
        .display_with_modes(&object.cast(), 0, 3, false, PrsMgrDisplayStatus::Displayed)
        .unwrap();
    assert!(
        context
            .selection_manager()
            .is_activated(&object.cast(), 3)
            .unwrap()
    );
    assert_eq!(*object.events.read(), [100, 203]);
}
#[test]
fn first_compute_failure_retains_registration_before_presentation_and_selection() {
    let (context, object) = display_fixture();
    object.fail_mode.store(0, Ordering::Release);
    assert!(context.display(&object.cast(), false).is_err());
    assert!(context.is_displayed_mode(&object.cast(), 0));
    assert_eq!(object.get_context(), context);
    assert!(!context.selection_manager().contains(&object.cast()));
    assert_eq!(*object.events.read(), [100]);
}
#[test]
fn selection_failure_keeps_the_displayed_presentation_and_can_be_retried() {
    let (context, object) = display_fixture();
    object.fail_mode.store(10, Ordering::Release);
    assert!(context.display(&object.cast(), false).is_err());
    assert!(retained_mode(&context, &object, 0).is_visible());
    assert!(context.selection_manager().contains(&object.cast()));
    assert!(
        !context
            .selection_manager()
            .is_activated(&object.cast(), 0)
            .unwrap()
    );
    object.fail_mode.store(-999, Ordering::Release);
    context.display(&object.cast(), false).unwrap();
    assert!(
        context
            .selection_manager()
            .is_activated(&object.cast(), 0)
            .unwrap()
    );
    assert_eq!(*object.events.read(), [100, 200, 200]);
}
#[test]
fn erase_clears_matching_selection_and_dynamic_highlight_only() {
    let (context, object) = display_fixture();
    let other = ContextObject::new();
    context.display(&object.cast(), false).unwrap();
    context.display(&other.cast(), false).unwrap();
    context.set_automatic_hilight(false);
    context.add_select_object(&object.cast()).unwrap();
    context.add_select_object(&other.cast()).unwrap();
    context.fields.write().last_picked = object.global_sel_owner();
    context.erase(&object.cast(), false).unwrap();
    assert!(!object.global_sel_owner().is_selected());
    assert!(other.global_sel_owner().is_selected());
    assert_eq!(context.nb_selected(), 1);
    assert_eq!(*object.events.read(), [100, 200, 401]);
}
#[test]
fn erased_foreign_object_clears_custom_selection_before_context_rejection() {
    let (context, object) = display_fixture();
    let (foreign, _) = display_fixture();
    object.set_context(&foreign);
    object.set_auto_hilight(false);
    assert!(context.display(&object.cast(), false).is_err());
    assert!(object.events.read().is_empty());
    assert!(
        context
            .display_with_modes(&object.cast(), 0, 0, false, PrsMgrDisplayStatus::Erased)
            .is_err()
    );
    assert_eq!(*object.events.read(), [400]);
    assert_eq!(object.get_context(), foreign);
}
#[test]
fn null_operations_are_noops_and_new_display_resets_only_new_affinity() {
    let (context, object) = display_fixture();
    context.display(&Handle::default(), true).unwrap();
    context
        .display_with_modes(&Handle::default(), 1, 2, true, PrsMgrDisplayStatus::Erased)
        .unwrap();
    context.erase(&Handle::default(), true).unwrap();
    context.erase_global(&Handle::default(), true).unwrap();
    object.view_affinity().set_visible(false);
    context.display(&object.cast(), false).unwrap();
    assert!(object.view_affinity().is_visible(0));
    object.view_affinity().set_view_visible(0, false);
    context.display(&object.cast(), false).unwrap();
    assert!(!object.view_affinity().is_visible(0));
}
