//! Actual context/object redisplay and callbacks, including retained failure state.
use rustycad_tkv3d::v3d::V3dViewSource;
#[path = "../../tests/support/context_object.rs"]
pub(super) mod context_object;
use super::common::{Reader, attempt, end, flag, int};
use context_object::ContextObject;
use rustycad_tkernel::{
    ncollection::{List, Vec2},
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkservice::{
    aspect::{AspectNeutralWindow, AspectWindow},
    graphic3d::*,
    null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, v3d::*};
use std::sync::atomic::Ordering;
struct ContextScene {
    ctx: Handle<AisInteractiveContext>,
    object: Handle<ContextObject>,
    original: Handle<Prs3dDrawer>,
    extra: Handle<Prs3dDrawer>,
    owner: Handle<StandardTransient>,
    view: Handle<dyn V3dViewSource>,
}
impl ContextScene {
    fn new(active: bool) -> Self {
        let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
        let ctx = AisInteractiveContext::new(&viewer).unwrap();
        let original = ctx.default_drawer().clone();
        let view = if active {
            let view = viewer.create_view().unwrap();
            let window = Handle::new(AspectNeutralWindow::default());
            window.set_size(80, 60);
            view.set_window(&window.cast::<dyn AspectWindow>(), 0)
                .unwrap();
            view
        } else {
            Handle::default()
        };
        Self {
            ctx,
            original,
            object: ContextObject::new(),
            extra: Handle::new(Prs3dDrawer::new()),
            owner: Handle::new(StandardTransient::default()),
            view,
        }
    }
    fn operation(&self, op: i32, value: i32, other: i32) -> Result<(), Failure> {
        let object = &self.object;
        let ctx = &self.ctx;
        match op {
            0 => object.set_context(&if value != 0 {
                ctx.clone()
            } else {
                Handle::default()
            }),
            1 => {
                let mut fields = ctx.fields.write();
                if !fields.objects.is_bound(&object.cast()) {
                    fields
                        .objects
                        .bind(object.cast(), Handle::new(AisGlobalStatus::new()));
                }
                fields
                    .objects
                    .seek(&object.cast())
                    .unwrap()
                    .add_selection_mode(value);
            }
            2 => object.set_display_status(match value {
                0 => PrsMgrDisplayStatus::Displayed,
                1 => PrsMgrDisplayStatus::Erased,
                _ => PrsMgrDisplayStatus::None,
            }),
            3 => {
                let manager = ctx.main_prs_mgr().clone();
                let prs = manager.presentation(&object.cast(), value, true, &Handle::default())?;
                if other != 0 {
                    prs.display();
                }
            }
            4 => {
                let manager = ctx.selection_manager().clone();
                manager.load(&object.cast(), value)?;
                if other != 0 {
                    manager.activate(&object.cast(), value)?;
                }
            }
            5 => object.redisplay(value != 0)?,
            6 => ctx.recompute_prs_only(&object.cast(), value != 0, other != 0)?,
            7 => ctx.recompute_selection_only(&object.cast())?,
            8 => ctx.update(&object.cast(), value != 0)?,
            9 => object.fail_mode.store(value, Ordering::Release),
            10 => object.attributes().set_display_mode(value),
            11 => {
                if value != 0 {
                    object.set_owner(&self.owner)
                } else {
                    object.clear_owner()
                }
            }
            12 => object.attributes().link_to(&self.extra),
            13 => ctx.set_default_drawer(if value != 0 {
                &self.extra
            } else {
                &self.original
            }),
            14 => {
                if let Some(status) = ctx.fields.read().objects.seek(&object.cast()) {
                    status.remove_selection_mode(value);
                }
            }
            15 => object.set_to_update_all(),
            16 => ctx.redisplay(&object.cast(), value != 0, other != 0)?,
            17 => {
                ctx.fields.write().last_active_view = if value != 0 {
                    self.view.downgrade()
                } else {
                    Default::default()
                }
            }
            18 => {
                ctx.redisplay(&Handle::default(), value != 0, other != 0)?;
                ctx.update(&Handle::default(), true)?;
                ctx.recompute_selection_only(&Handle::default())?;
            }
            19 => object_dump(object, value),
            20 => object_aspect(object, value)?,
            21 => {
                let prs = object.presentation()?;
                if !prs.is_null() {
                    if value != 0 {
                        prs.new_group()?;
                    } else {
                        prs.clear(true);
                    }
                }
            }
            22 => {
                if !self.view.is_null() {
                    let viewer = ctx.current_viewer().clone();
                    if value != 0 {
                        viewer.set_view_on_one(&self.view)?;
                    } else {
                        viewer.set_view_off_one(&self.view);
                    }
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn state(&self) {
        let object = &self.object;
        let ctx = &self.ctx;
        flag(object.has_interactive_context());
        flag(object.interactive_context().object_address() == ctx.object_address());
        flag(object.get_context() == *ctx);
        flag(object.has_owner());
        flag(*object.get_owner() == self.owner);
        int(object.display_status() as i32);
        flag(object.attributes().link() == self.original);
        flag(object.attributes().link() == self.extra);
        flag(object.has_presentation());
        flag(!object.presentation().unwrap().is_null());
        flag(!ctx.last_active_view().is_null());
        let mut modes = List::default();
        modes.append(-5);
        ctx.activated_modes(&object.cast(), &mut modes);
        int(modes.len() as i32);
        for mode in modes.iter() {
            int(*mode);
        }
        self.presentation_state();
        self.selection_state();
        let mut events = object.events.write();
        int(events.len() as i32);
        for event in events.iter() {
            int(*event);
        }
        events.clear();
    }
    fn presentation_state(&self) {
        let manager = self.ctx.main_prs_mgr().clone();
        for mode in 0..3 {
            let prs = manager
                .presentation(&self.object.cast(), mode, false, &Handle::default())
                .unwrap();
            flag(!prs.is_null());
            if let Some(prs) = prs.get() {
                flag(prs.must_be_updated());
                flag(prs.is_displayed());
                int(prs.groups().len() as i32);
            }
        }
    }
    fn selection_state(&self) {
        let manager = self.ctx.selection_manager().clone();
        flag(manager.contains(&self.object.cast()));
        let selections = self.object.selections();
        int(selections.len() as i32);
        for selection in selections {
            int(selection.mode());
            int(selection.update_status() as i32);
            int(selection.bvh_update_status() as i32);
            int(selection.get_selection_state() as i32);
            int(selection.entities().len() as i32);
            flag(
                manager
                    .is_activated(&self.object.cast(), selection.mode())
                    .unwrap(),
            );
        }
    }
}
pub fn ais_object_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let scene = ContextScene::new(r.integer() != 0);
    int(scene.object.kind_of_interactive() as i32);
    int(scene.object.signature());
    for action in [
        AisDragAction::Start,
        AisDragAction::Confirmed,
        AisDragAction::Update,
        AisDragAction::Stop,
        AisDragAction::Abort,
    ] {
        flag(scene.object.process_dragging(
            &scene.ctx,
            &Handle::default(),
            &Handle::default(),
            Vec2::new(0, 0),
            Vec2::new(1, 2),
            action,
        ));
    }
    scene.state();
    for _ in 0..r.integer() {
        let (op, value, other) = (r.integer(), r.integer(), r.integer());
        attempt(|| scene.operation(op, value, other));
        scene.state();
    }
    end();
}

fn object_dump(object: &ContextObject, depth: i32) {
    let mut stream = rustycad_tkernel::standard::dump::DumpStream::default();
    object.dump_json(&mut stream, depth).unwrap();
    super::common::text(
        super::frustum_core::named_pointer_identities(
            stream.text().to_owned(),
            &["CTXPtr", "Owner"],
        )
        .as_bytes(),
    );
}
fn object_aspect(object: &ContextObject, kind: i32) -> Result<(), Failure> {
    let (aspect, graphic) = object_aspect_pair(kind);
    let prs = object.presentation()?;
    let groups = prs
        .get()
        .map_or_else(Vec::new, Graphic3dStructureMembers::groups);
    let before: Vec<_> = groups.iter().map(|group| group.aspects()).collect();
    object.set_aspect(&aspect)?;
    int(before.len() as i32);
    for (group, previous) in groups.iter().zip(before) {
        flag(group.aspects() == previous);
        flag(group.aspects() == graphic);
    }
    Ok(())
}
fn object_aspect_pair(
    kind: i32,
) -> (
    Handle<dyn Prs3dBasicAspect>,
    Handle<dyn Graphic3dAspectsSource>,
) {
    use rustycad_tkernel::quantity::{Color, NamedColor};
    use rustycad_tkservice::aspect::{AspectTypeOfLine, AspectTypeOfMarker};
    match kind {
        1 => {
            let a = Handle::new(Prs3dShadingAspect::new().unwrap());
            (a.cast(), a.aspect().cast())
        }
        2 => {
            let a = Handle::new(
                Prs3dLineAspect::new(Color::named(NamedColor::RED), AspectTypeOfLine::Solid, 3.)
                    .unwrap(),
            );
            (a.cast(), a.aspect().cast())
        }
        3 => {
            let a = Handle::new(
                Prs3dPointAspect::new(AspectTypeOfMarker::Plus, Color::named(NamedColor::RED), 2.)
                    .unwrap(),
            );
            (a.cast(), a.aspect().cast())
        }
        4 => {
            let a = Handle::new(Prs3dTextAspect::new());
            (a.cast(), a.aspect().cast())
        }
        5 => (
            Handle::new(Prs3dArrowAspect::new().unwrap()).cast(),
            Handle::default(),
        ),
        _ => (Handle::default(), Handle::default()),
    }
}
struct ObservedSelection {
    transient: StandardTransient,
    base: AisSelectionBase,
    clears: std::sync::atomic::AtomicI32,
    fail_clear: bool,
}
rustycad_tkernel::impl_transient!(ObservedSelection,"ObservedSelection",dyn AisSelectionSource,transient,interfaces:[dyn AisSelectionSource]);
impl AisSelectionSource for ObservedSelection {
    fn selection_base(&self) -> &AisSelectionBase {
        &self.base
    }
    fn clear(&self) -> Result<(), Failure> {
        self.clears.fetch_add(1, Ordering::AcqRel);
        if self.fail_clear {
            return Err(Failure::with_message(
                "Standard_Failure",
                "Clear failed; expected successful fixture callback",
            ));
        }
        Ok(())
    }
}
pub fn context_shutdown_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let registered = r.integer() != 0;
    let selected = r.integer() != 0;
    let custom = r.integer();
    let ContextScene { ctx, object, .. } = ContextScene::new(false);
    object.set_context(&ctx);
    if registered {
        ctx.fields
            .write()
            .objects
            .bind(object.cast(), Handle::new(AisGlobalStatus::new()));
    }
    ctx.selection_manager().load(&object.cast(), 1).unwrap();
    let selection = object.selection(1);
    selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::None);
    let owner = SelectMgrEntityOwner::with_selectable(&object.cast(), 0);
    let original = ctx.selection().clone();
    if selected {
        original.add_select(&owner.cast()).unwrap();
    }
    let replacement = Handle::new(ObservedSelection {
        transient: StandardTransient::default(),
        base: AisSelectionBase::new(),
        clears: std::sync::atomic::AtomicI32::new(0),
        fail_clear: custom == 2,
    });
    if custom != 0 {
        ctx.set_selection(&replacement.cast());
    }
    flag(*ctx.selection() == replacement.cast::<dyn AisSelectionSource>());
    int(replacement.clears.load(Ordering::Acquire));
    drop(ctx);
    flag(owner.is_selected());
    int(original.extent());
    int(replacement.clears.load(Ordering::Acquire));
    flag(object.has_interactive_context());
    int(selection.bvh_update_status() as i32);
    for kind in [
        AisKindOfInteractive::None,
        AisKindOfInteractive::Datum,
        AisKindOfInteractive::Shape,
        AisKindOfInteractive::Object,
        AisKindOfInteractive::Relation,
        AisKindOfInteractive::Dimension,
        AisKindOfInteractive::LightSource,
        AisKindOfInteractive::KOI_NONE,
        AisKindOfInteractive::KOI_DATUM,
        AisKindOfInteractive::KOI_SHAPE,
        AisKindOfInteractive::KOI_OBJECT,
        AisKindOfInteractive::KOI_RELATION,
        AisKindOfInteractive::KOI_DIMENSION,
    ] {
        int(kind as i32);
    }
    for action in [
        AisDragAction::Start,
        AisDragAction::Confirmed,
        AisDragAction::Update,
        AisDragAction::Stop,
        AisDragAction::Abort,
    ] {
        int(action as i32);
    }
    end();
}
