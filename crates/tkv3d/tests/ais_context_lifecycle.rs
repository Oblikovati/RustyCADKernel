//! Failure ordering, borrowed selection replacement, live-view validation and object dump dispatch.
#[path = "support/context_object.rs"]
mod context_object;
use context_object::ContextObject;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Handle, SharedState, StandardTransient, dump::DumpStream},
};
use rustycad_tkservice::{aspect::*, graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, v3d::*};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
fn live_context() -> Handle<AisInteractiveContext> {
    AisInteractiveContext::new(&V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap()).unwrap()
}
fn prepared_object(context: &AisInteractiveContext) -> Handle<ContextObject> {
    let object = ContextObject::new();
    object.set_display_status(PrsMgrDisplayStatus::Displayed);
    let status = Handle::new(AisGlobalStatus::new());
    status.add_selection_mode(1);
    context.fields.write().objects.bind(object.cast(), status);
    let manager = context.main_prs_mgr().clone();
    manager
        .presentation(&object.cast(), 0, true, &Handle::default())
        .unwrap()
        .display();
    let manager = context.selection_manager().clone();
    manager.load(&object.cast(), 1).unwrap();
    manager.activate(&object.cast(), 1).unwrap();
    object.events.write().clear();
    object
}
#[test]
fn redisplay_failure_stops_in_native_order_and_retains_partial_changes() {
    let context = live_context();
    let object = prepared_object(&context);
    object.set_context(&context);
    object.fail_mode.store(0, Ordering::Release);
    assert!(context.redisplay(&object.cast(), false, true).is_err());
    assert_eq!(*object.events.read(), [100]);
    assert!(
        context
            .selection_manager()
            .is_activated(&object.cast(), 1)
            .unwrap()
    );
    object.events.write().clear();
    object.fail_mode.store(11, Ordering::Release);
    assert!(context.redisplay(&object.cast(), false, true).is_err());
    assert_eq!(*object.events.read(), [100, 201]);
    assert!(object.selection(1).entities().is_empty());
    assert!(
        !context
            .selection_manager()
            .is_activated(&object.cast(), 1)
            .unwrap()
    );
    object.fail_mode.store(-999, Ordering::Release);
    object.redisplay(true).unwrap();
    assert!(
        context
            .selection_manager()
            .is_activated(&object.cast(), 1)
            .unwrap()
    );
}
#[test]
fn remembered_view_requires_current_active_viewer_membership() {
    let context = live_context();
    let viewer = context.current_viewer().clone();
    let view = viewer.create_view().unwrap();
    context.fields.write().last_active_view = view.downgrade();
    assert!(context.last_active_view().is_null());
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(80, 60);
    view.set_window(&window.cast(), 0).unwrap();
    assert_eq!(context.last_active_view(), view);
    viewer.set_view_off_one(&view);
    assert!(context.last_active_view().is_null());
    viewer.set_view_on_one(&view).unwrap();
    assert_eq!(context.last_active_view(), view);
    let object = ContextObject::new();
    context
        .recompute_prs_only(&object.cast(), true, false)
        .unwrap();
    assert_eq!(*object.events.read(), [300]);
}
#[test]
fn all_supported_aspects_change_only_the_last_group() {
    let context = live_context();
    let object = prepared_object(&context);
    object.set_context(&context);
    object.attributes().set_display_mode(0);
    let prs = object.presentation().unwrap();
    let first = prs.groups()[0].clone();
    let last = prs.new_group().unwrap();
    let shading = Handle::new(Prs3dShadingAspect::new().unwrap());
    let text = Handle::new(Prs3dTextAspect::new());
    let point = Handle::new(
        Prs3dPointAspect::new(AspectTypeOfMarker::Plus, Color::named(NamedColor::RED), 2.).unwrap(),
    );
    let aspects: [(
        Handle<dyn Prs3dBasicAspect>,
        Handle<dyn Graphic3dAspectsSource>,
    ); 3] = [
        (shading.cast(), shading.aspect().cast()),
        (point.cast(), point.aspect().cast()),
        (text.cast(), text.aspect().cast()),
    ];
    for (aspect, expected) in aspects {
        object.set_aspect(&aspect).unwrap();
        assert_eq!(last.aspects(), expected);
        assert!(first.aspects().is_null());
    }
    object.set_aspect(&Handle::default()).unwrap();
    assert_eq!(
        last.aspects(),
        text.aspect().cast::<dyn Graphic3dAspectsSource>()
    );
    prs.clear(true);
    object.set_aspect(&text.cast()).unwrap();
    assert!(prs.groups().is_empty());
}
#[test]
fn interactive_dump_override_and_presentation_base_preserve_native_identity() {
    let object = ContextObject::new();
    let mut stream = DumpStream::default();
    object.dump_json(&mut stream, 0).unwrap();
    assert_eq!(
        stream.text(),
        "\"className\": \"AIS_InteractiveObject\", \"CTXPtr\": \"\", \"Owner\": \"\""
    );
    let mut stream = DumpStream::default();
    object.dump_json(&mut stream, 2).unwrap();
    assert!(
        stream
            .text()
            .contains("\"className\": \"SelectMgr_SelectableObject\"")
    );
    assert!(
        stream
            .text()
            .contains("\"className\": \"PrsMgr_PresentableObject\"")
    );
    let base = AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::ProjectorDependent);
    assert_eq!(
        base.presentable.fields.read().type_of_presentation3d,
        PrsMgrTypeOfPresentation3d::ProjectorDependent
    );
    assert!(base.fields.read().context.is_null());
    assert!(base.fields.read().owner.is_null());
}
struct ObservedSelection {
    transient: StandardTransient,
    base: AisSelectionBase,
    clears: Arc<AtomicUsize>,
}
rustycad_tkernel::impl_transient!(ObservedSelection,"ObservedSelection",dyn AisSelectionSource,transient,interfaces:[dyn AisSelectionSource]);
impl AisSelectionSource for ObservedSelection {
    fn selection_base(&self) -> &AisSelectionBase {
        &self.base
    }
    fn clear(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
        self.clears.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }
}
#[test]
fn replacement_selection_is_borrowed_and_destructor_dispatches_virtual_clear() {
    let context = live_context();
    let previous = context.selection().clone();
    let clears = Arc::new(AtomicUsize::new(0));
    let replacement = Handle::new(ObservedSelection {
        transient: StandardTransient::default(),
        base: AisSelectionBase::new(),
        clears: clears.clone(),
    });
    context.set_selection(&replacement.cast());
    assert_eq!(
        *context.selection(),
        replacement.cast::<dyn AisSelectionSource>()
    );
    assert!(std::ptr::eq(
        &*context.selection(),
        &context.fields.read().selection
    ));
    assert_eq!(clears.load(Ordering::Acquire), 0);
    drop(context);
    assert_eq!(clears.load(Ordering::Acquire), 1);
    assert!(previous.is_empty());
}
#[test]
fn interactive_enum_values_and_legacy_aliases_are_native() {
    let kinds = [
        AisKindOfInteractive::None,
        AisKindOfInteractive::Datum,
        AisKindOfInteractive::Shape,
        AisKindOfInteractive::Object,
        AisKindOfInteractive::Relation,
        AisKindOfInteractive::Dimension,
        AisKindOfInteractive::LightSource,
    ];
    assert_eq!(kinds.map(|kind| kind as i32), [0, 1, 2, 3, 4, 5, 6]);
    let aliases = [
        AisKindOfInteractive::KOI_NONE,
        AisKindOfInteractive::KOI_DATUM,
        AisKindOfInteractive::KOI_SHAPE,
        AisKindOfInteractive::KOI_OBJECT,
        AisKindOfInteractive::KOI_RELATION,
        AisKindOfInteractive::KOI_DIMENSION,
    ];
    assert_eq!(aliases, kinds[..6]);
    assert_eq!(
        [
            AisDragAction::Start,
            AisDragAction::Confirmed,
            AisDragAction::Update,
            AisDragAction::Stop,
            AisDragAction::Abort
        ]
        .map(|action| action as i32),
        [0, 1, 2, 3, 4]
    );
    let fields = AisInteractiveObjectFields::default();
    let shared = SharedState::new(fields);
    assert!(shared.read().owner.is_null());
}
