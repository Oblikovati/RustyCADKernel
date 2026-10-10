//! Native construction, borrowed handle identity and real object presentation/selection updates.
#[path = "support/context_object.rs"]
mod context_object;
use context_object::ContextObject;
use rustycad_tkernel::{
    ncollection::{List, Vec2},
    quantity::{Color, NamedColor},
    standard::{Handle, RuntimeTyped, StandardTransient},
};
use rustycad_tkservice::{
    aspect::{AspectTypeOfFacingModel, AspectTypeOfLine},
    graphic3d::*,
    null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, v3d::V3dViewer};
fn context() -> Handle<AisInteractiveContext> {
    AisInteractiveContext::new(&V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap()).unwrap()
}
#[test]
fn context_constructor_creates_native_managers_flags_and_style_identity() {
    let context = context();
    assert_eq!(
        AisInteractiveContext::static_type().name(),
        "AIS_InteractiveContext"
    );
    assert!(
        context.to_hilight_selected()
            && context.automatic_hilight()
            && context.get_auto_activate_selection()
    );
    assert_eq!(
        context.picking_strategy(),
        SelectMgrPickingStrategy::FirstAcceptable
    );
    assert!(context.selection().is_empty());
    let fields = context.fields.read();
    assert!(fields.objects.is_empty());
    assert!(fields.last_picked.is_null());
    assert!(fields.detected.is_empty());
    assert_eq!(
        (fields.current_detected, fields.current_highlighted),
        (0, 0)
    );
    drop(fields);
    assert!(!context.main_prs_mgr().is_null());
    assert!(!context.selection_manager().is_null());
    assert_eq!(context.main_selector().pixel_tolerance(), 2);
    assert_eq!(context.main_selector().custom_pixel_tolerance(), -1);
    assert_eq!(
        *context.default_drawer(),
        *context.highlight_style(Prs3dTypeOfHighlight::None).unwrap()
    );
    assert!(std::ptr::eq(
        &*context.default_drawer(),
        &context.fields.read().default_drawer
    ));
    assert!(std::ptr::eq(
        &*context.main_selector(),
        context.fields.read().selection_manager.selector()
    ));
    assert!(context.last_active_view().is_null());
    assert!(context.highlight_style(Prs3dTypeOfHighlight::Nb).is_err());
    assert!(AisInteractiveContext::new(&Handle::default()).is_err());
}
#[test]
fn highlight_plane_edges_share_default_aspects_while_lines_are_independent() {
    let context = context();
    let default = context.default_drawer().clone();
    let dynamic = context.dynamic_highlight_style().clone();
    assert_eq!(
        default.plane_aspect().edges_aspect().aspect(),
        dynamic.plane_aspect().edges_aspect().aspect()
    );
    assert_ne!(
        default.line_aspect().aspect(),
        dynamic.line_aspect().aspect()
    );
    assert_eq!(
        dynamic
            .plane_aspect()
            .edges_aspect()
            .aspect()
            .interior_color(),
        Color::named(NamedColor::SKYBLUE)
    );
    assert_eq!(
        dynamic.line_aspect().aspect().interior_color(),
        Color::named(NamedColor::CYAN1)
    );
    assert_eq!(default.plane_aspect().edges_aspect().aspect().width(), 2.);
    assert_eq!(
        default.hidden_line_aspect().aspect().line_kind(),
        AspectTypeOfLine::Dash
    );
    assert_eq!(
        default.datum_aspect().axis_length(Prs3dDatumParts::XAxis),
        100.
    );
    assert_eq!(default.plane_aspect().plane_x_length(), 200.);
    assert_eq!(
        default
            .shading_aspect()
            .material(AspectTypeOfFacingModel::FrontSide)
            .name(),
        Graphic3dNameOfMaterial::Brass
    );
    assert!(!dynamic.is_auto_triangulation());
}
#[test]
fn style_setters_preserve_none_default_pair_without_relinking_objects() {
    let context = context();
    let object = ContextObject::new();
    object.set_context(&context);
    let original = object.attributes().link();
    let drawer = Handle::new(Prs3dDrawer::new());
    context.set_default_drawer(&drawer);
    assert_eq!(
        *context.highlight_style(Prs3dTypeOfHighlight::None).unwrap(),
        drawer
    );
    assert_eq!(object.attributes().link(), original);
    context.set_dynamic_highlight_style(&drawer);
    context.set_selection_style(&drawer);
    assert_eq!(*context.dynamic_highlight_style(), drawer);
    assert_eq!(*context.selection_style(), drawer);
    context
        .set_highlight_style(Prs3dTypeOfHighlight::None, &original)
        .unwrap();
    assert_eq!(*context.default_drawer(), original);
    assert!(
        context
            .set_highlight_style(Prs3dTypeOfHighlight::Nb, &drawer)
            .is_err()
    );
    context.set_to_hilight_selected(false);
    context.set_automatic_hilight(false);
    context.set_auto_activate_selection(false);
    context.set_picking_strategy(SelectMgrPickingStrategy::OnlyTopmost);
    assert!(
        !context.to_hilight_selected()
            && !context.automatic_hilight()
            && !context.get_auto_activate_selection()
    );
    assert_eq!(
        context.picking_strategy(),
        SelectMgrPickingStrategy::OnlyTopmost
    );
    context.update_current_viewer();
}
#[test]
fn object_context_link_owner_and_default_virtual_members_are_native() {
    let object = ContextObject::new();
    assert!(!object.has_interactive_context());
    assert!(!object.has_owner());
    assert_eq!(object.kind_of_interactive(), AisKindOfInteractive::None);
    assert_eq!(object.signature(), -1);
    let context = context();
    object.set_context(&context);
    assert_eq!(object.get_context(), context);
    assert_eq!(
        object.interactive_context().object_address(),
        context.object_address()
    );
    let replacement = Handle::new(Prs3dDrawer::new());
    object.attributes().link_to(&replacement);
    object.set_context(&context);
    assert_eq!(object.attributes().link(), replacement);
    object.set_context(&Handle::default());
    assert!(!object.has_interactive_context());
    assert_eq!(object.attributes().link(), replacement);
    let owner = Handle::new(StandardTransient::default());
    object.set_owner(&owner);
    assert!(object.has_owner());
    assert_eq!(*object.get_owner(), owner);
    assert!(std::ptr::eq(
        &*object.get_owner(),
        &object.interactive_base().fields.read().owner
    ));
    object.clear_owner();
    assert!(!object.has_owner());
    for action in [
        AisDragAction::Start,
        AisDragAction::Confirmed,
        AisDragAction::Update,
        AisDragAction::Stop,
        AisDragAction::Abort,
    ] {
        assert!(!object.process_dragging(
            &context,
            &Handle::default(),
            &Handle::default(),
            Vec2::new(0, 0),
            Vec2::new(1, 2),
            action
        ));
    }
    object.redisplay(false).unwrap();
    assert!(!object.has_presentation());
    assert!(object.presentation().unwrap().is_null());
}
#[test]
fn object_presentation_lookup_uses_its_context_manager_and_display_mode() {
    let context = context();
    let object = ContextObject::new();
    object.set_context(&context);
    object.attributes().set_display_mode(2);
    assert!(!object.has_presentation());
    let manager = context.main_prs_mgr().clone();
    let prs = manager
        .presentation(&object.cast(), 2, true, &Handle::default())
        .unwrap();
    assert!(object.has_presentation());
    assert_eq!(
        object.presentation().unwrap(),
        prs.cast::<dyn Graphic3dStructureSource>()
    );
    object.attributes().set_display_mode(3);
    assert!(!object.has_presentation());
    object.attributes().set_display_mode(2);
    let first = prs.groups()[0].clone();
    let last = prs.new_group().unwrap();
    let line = Handle::new(
        Prs3dLineAspect::new(Color::named(NamedColor::RED), AspectTypeOfLine::Solid, 3.).unwrap(),
    );
    object.set_aspect(&line.cast()).unwrap();
    assert!(first.aspects().is_null());
    assert_eq!(
        last.aspects(),
        line.aspect().cast::<dyn Graphic3dAspectsSource>()
    );
    object
        .set_aspect(&Handle::new(Prs3dArrowAspect::new().unwrap()).cast())
        .unwrap();
    assert_eq!(
        last.aspects(),
        line.aspect().cast::<dyn Graphic3dAspectsSource>()
    );
}
#[test]
fn redisplay_recomputes_real_presentations_and_restores_recorded_selection_modes() {
    let context = context();
    let object = ContextObject::new();
    object.set_context(&context);
    object.set_display_status(PrsMgrDisplayStatus::Displayed);
    let status = Handle::new(AisGlobalStatus::new());
    status.add_selection_mode(1);
    context.fields.write().objects.bind(object.cast(), status);
    let manager = context.main_prs_mgr().clone();
    let prs = manager
        .presentation(&object.cast(), 0, true, &Handle::default())
        .unwrap();
    prs.display();
    let selection_manager = context.selection_manager().clone();
    selection_manager.load(&object.cast(), 1).unwrap();
    selection_manager.activate(&object.cast(), 1).unwrap();
    object.events.write().clear();
    object.redisplay(true).unwrap();
    assert_eq!(*object.events.read(), [100, 201]);
    assert!(selection_manager.is_activated(&object.cast(), 1).unwrap());
    let mut modes = List::default();
    modes.append(-5);
    context.activated_modes(&object.cast(), &mut modes);
    assert_eq!(modes.iter().copied().collect::<Vec<_>>(), [-5, 1]);
    context.redisplay(&Handle::default(), true, true).unwrap();
    context.update(&Handle::default(), true).unwrap();
    context.update(&object.cast(), false).unwrap();
}
#[test]
fn dropping_context_clears_registered_object_links_selection_flags_and_bvh_status() {
    let context = context();
    let object = ContextObject::new();
    object.set_context(&context);
    context
        .fields
        .write()
        .objects
        .bind(object.cast(), Handle::new(AisGlobalStatus::new()));
    let manager = context.selection_manager().clone();
    manager.load(&object.cast(), 1).unwrap();
    let selection = object.selection(1);
    selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::None);
    let owner = SelectMgrEntityOwner::with_selectable(&object.cast(), 0);
    let selected = context.selection().clone();
    selected.add_select(&owner.cast()).unwrap();
    drop(context);
    assert!(!object.has_interactive_context());
    assert!(!owner.is_selected());
    assert!(selected.is_empty());
    assert_eq!(
        selection.bvh_update_status(),
        SelectMgrTypeOfBvhUpdate::Renew
    );
}
