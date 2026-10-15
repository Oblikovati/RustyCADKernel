//! AIS_ViewController dispatch: dragging an object that accepts it through the queued start,
//! move, abort and stop requests, the virtual callbacks and the inactive XR handlers.
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::{Failure, Handle, HandleReference, StandardTransient};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::select_mgr::*;
use rustycad_tkv3d::select3d::Select3DSensitivePoint;
use rustycad_tkv3d::std_prs::StdPrsPoint;
use rustycad_tkv3d::v3d::{V3dTypeOfOrientation, V3dViewSource, V3dViewer};
use std::sync::{Arc, Mutex};

/// One recorded drag: the action with its start and target pixels.
type DragCall = (AisDragAction, Vec2<i32>, Vec2<i32>);

/// A selectable point accepting every drag action, recording them.
struct Draggable {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    actions: Mutex<Vec<DragCall>>,
}
rustycad_tkernel::impl_transient!(Draggable, "Draggable", dyn AisInteractiveObject, transient,
    interfaces: [dyn AisInteractiveObject, dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);
impl AisInteractiveObject for Draggable {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    fn process_dragging(
        &self,
        _: &Handle<AisInteractiveContext>,
        _: &Handle<dyn V3dViewSource>,
        _: &Handle<dyn SelectMgrEntityOwnerSource>,
        from: Vec2<i32>,
        to: Vec2<i32>,
        action: AisDragAction,
    ) -> bool {
        self.actions.lock().unwrap().push((action, from, to));
        true
    }
}
impl PrsMgrPresentableObject for Draggable {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    /// A point marker: the selectable-object boxes come from the presentation, as natively.
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let point = Handle::new(GeomCartesianPoint::new(0., 0., 0.));
        StdPrsPoint::add(&prs.observe().clone_handle(), &point.cast(), &self.attributes())
    }
}
impl SelectMgrSelectableObject for Draggable {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 5);
        selection.observe().add(
            &Handle::new(Select3DSensitivePoint::new(&owner.cast(), Point3::new(0., 0., 0.)))
                .cast(),
        )
    }
}

struct Scene {
    ctx: Handle<AisInteractiveContext>,
    view: Handle<dyn V3dViewSource>,
    object: Handle<Draggable>,
}
fn scene() -> Scene {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(200, 100);
    view.set_window(&window.cast(), 0).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let object = Handle::new(Draggable {
        transient: StandardTransient::default(),
        base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
        actions: Mutex::new(Vec::new()),
    });
    prs_mgr_presentable_bind(&object.cast());
    ctx.display(&object.clone().into_cast(), false).unwrap();
    // Two outer points give the fitted view an extent around the draggable origin.
    for x in [-10.0, 10.0] {
        let point = AisPoint::new(&Handle::new(GeomCartesianPoint::new(x, 0., 0.)).cast()).unwrap();
        ctx.display(&point.into_cast(), false).unwrap();
    }
    view.set_proj_orientation(V3dTypeOfOrientation::ZUP_TOP, false).unwrap();
    view.fit_all(0.1, false).unwrap();
    Scene { ctx, view, object }
}

fn start_drag(c: &mut AisViewController, s: &Scene, at: Vec2<i32>) {
    let ui = c.change_input_buffer(AisViewInputBufferType::Ui);
    ui.dragging.to_start = true;
    ui.dragging.point_start = at;
    c.flush_view_events(&s.ctx, &s.view, true).unwrap();
}

#[test]
fn dragging_starts_on_the_detected_object_and_moves_until_stopped() {
    let s = scene();
    let (px, py) = s.view.convert_world_to_pixels(0., 0., 0.).unwrap();
    let mut c = AisViewController::new().unwrap();
    start_drag(&mut c, &s, Vec2::new(px, py));
    let dragged: Handle<dyn AisInteractiveObject> = s.object.clone().into_cast();
    assert!(*c.drag_object() == dragged);
    assert!(!c.drag_owner().is_null());
    let ui = c.change_input_buffer(AisViewInputBufferType::Ui);
    ui.dragging.to_move = true;
    ui.dragging.point_to = Vec2::new(px + 5, py);
    c.flush_view_events(&s.ctx, &s.view, true).unwrap();
    c.change_input_buffer(AisViewInputBufferType::Ui).dragging.to_stop = true;
    c.flush_view_events(&s.ctx, &s.view, true).unwrap();
    assert!(c.drag_object().is_null() && c.drag_owner().is_null());
    let actions: Vec<AisDragAction> = s.object.actions.lock().unwrap().iter().map(|a| a.0).collect();
    assert_eq!(
        actions,
        [AisDragAction::Start, AisDragAction::Update, AisDragAction::Stop]
    );
    let moves = s.object.actions.lock().unwrap();
    assert_eq!((moves[1].1, moves[1].2), (Vec2::new(px, py), Vec2::new(px + 5, py)));
}

#[test]
fn aborted_drag_moves_back_to_the_start_then_aborts_twice() {
    let s = scene();
    let (px, py) = s.view.convert_world_to_pixels(0., 0., 0.).unwrap();
    let mut c = AisViewController::new().unwrap();
    start_drag(&mut c, &s, Vec2::new(px, py));
    let ui = c.change_input_buffer(AisViewInputBufferType::Ui);
    ui.dragging.to_abort = true;
    c.flush_view_events(&s.ctx, &s.view, true).unwrap();
    let actions = s.object.actions.lock().unwrap().clone();
    let kinds: Vec<AisDragAction> = actions.iter().map(|a| a.0).collect();
    // OCCT falls through from Abort into Stop with the Abort action still set.
    assert_eq!(
        kinds,
        [
            AisDragAction::Start,
            AisDragAction::Update,
            AisDragAction::Abort,
            AisDragAction::Abort
        ]
    );
    assert!(actions[1..].iter().all(|a| a.1 == a.2));
    assert!(c.drag_object().is_null());
}

struct Hooks(Arc<Mutex<Vec<String>>>);
impl AisViewControllerHooks for Hooks {
    fn on_selection_changed(
        &mut self,
        controller: &mut AisViewController,
        _: &Handle<AisInteractiveContext>,
        _: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        // The controller stays usable from the callback.
        let pending = controller.input_buffer(AisViewInputBufferType::Gl).selection.points.len();
        self.0.lock().unwrap().push(format!("selection {pending}"));
        Ok(())
    }
    fn on_object_dragged(
        &mut self,
        _: &mut AisViewController,
        _: &Handle<AisInteractiveContext>,
        _: &Handle<dyn V3dViewSource>,
        action: AisDragAction,
    ) -> Result<(), Failure> {
        // Overriding without the base call never starts dragging.
        self.0.lock().unwrap().push(format!("drag {action:?}"));
        Ok(())
    }
}

#[test]
fn installed_hooks_replace_the_virtual_callbacks() {
    let s = scene();
    let (px, py) = s.view.convert_world_to_pixels(0., 0., 0.).unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut c = AisViewController::new().unwrap();
    c.set_hooks(Box::new(Hooks(log.clone())));
    c.select_in_viewer(Vec2::new(px, py), AisSelectionScheme::Replace);
    c.flush_view_events(&s.ctx, &s.view, true).unwrap();
    assert_eq!(s.ctx.nb_selected(), 1);
    start_drag(&mut c, &s, Vec2::new(px, py));
    assert!(c.drag_object().is_null());
    assert_eq!(*log.lock().unwrap(), ["selection 1", "drag Start"]);
    assert!(c.take_hooks().is_some());
    start_drag(&mut c, &s, Vec2::new(px, py));
    assert!(!c.drag_object().is_null());
}

#[test]
fn protected_state_starts_with_the_native_constructor_values() {
    let c = AisViewController::new().unwrap();
    assert_eq!(c.scroll_zoom_ratio(), 15.0);
    assert_eq!(c.mouse_click_threshold(), 3.0);
    assert_eq!(
        (
            c.touch_rotation_threshold_px(),
            c.touch_pan_threshold_px(),
            c.touch_zoom_threshold_px(),
            c.touch_dragging_threshold_px()
        ),
        (6.0, 4.0, 6.0, 6.0)
    );
    assert_eq!(c.touch_zoom_ratio(), 0.13);
    assert_eq!(c.touch_click_threshold_px(), 3.0);
    assert_eq!(c.touch_z_rotation_threshold(), (2.0 * std::f64::consts::PI / 180.0) as f32);
    assert_eq!((c.mouse_single_button(), c.mouse_click_counter()), (-1, 0));
    assert!(c.update_start_point_pan() && c.update_start_point_rot() && c.update_start_point_z_rot());
    assert!(!c.has_thrust() && !c.has_hlr_on_before_rotation());
    assert_eq!(c.nb_touches_last(), 0);
    assert_eq!(c.anchor_point_prs1().z_layer(), rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOP);
    assert_eq!(c.anchor_point_prs2().z_layer(), rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOPMOST);
    assert_eq!(c.rubber_band().z_layer(), rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOP_OSD);
}

#[test]
fn inactive_xr_handlers_keep_the_default_state() {
    let s = scene();
    let mut c = AisViewController::new().unwrap();
    let walk = AisWalkDelta::default();
    c.handle_xr_input(&s.ctx, &s.view, &walk).unwrap();
    c.handle_xr_presentations(&s.ctx, &s.view).unwrap();
    assert_eq!(c.xr_prs_devices().len(), 1);
    assert!(c.xr_prs_devices()[0].is_null());
    assert_eq!(c.xr_turn_angle(), std::f64::consts::FRAC_PI_4);
    let left = rustycad_tkservice::aspect::AspectXRTrackedDeviceRole::LeftHand;
    assert!(rustycad_tkernel::precision::is_infinite(c.xr_last_pick_depth(left)));
}
