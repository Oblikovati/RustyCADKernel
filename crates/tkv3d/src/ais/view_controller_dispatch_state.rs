//! The rubber band, dragged object and XR state of `AIS_ViewController`, with the overridable
//! selection, dragging and subview callbacks.
use super::{
    AisDragAction, AisInteractiveContext, AisInteractiveObject, AisRubberBand, AisViewController,
    AisXrTrackedDevice,
};
use crate::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::aspect::{
    AspectTypeOfLine, AspectTypeOfTriedronPosition, AspectXRHapticActionData,
    AspectXRTrackedDeviceRole,
};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_TOP_OSD, Graphic3dTransModeFlags, Graphic3dTransformPers,
};

/// The virtual callbacks of `AIS_ViewController` (`OnSelectionChanged`, `OnObjectDragged`,
/// `OnSubviewChanged`); an application installs them with
/// `controller.set_hooks(Box::new(MyHooks))`. Each receives the controller itself.
pub trait AisViewControllerHooks: Send + Sync {
    /// `OnSelectionChanged`: nothing by default; `hooks.on_selection_changed(c, &ctx, &view)`.
    fn on_selection_changed(
        &mut self,
        controller: &mut AisViewController,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let _ = (controller, context, view);
        Ok(())
    }
    /// `OnObjectDragged`: the native dragging of the detected object by default;
    /// `hooks.on_object_dragged(c, &ctx, &view, AisDragAction::Start)`.
    fn on_object_dragged(
        &mut self,
        controller: &mut AisViewController,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        action: AisDragAction,
    ) -> Result<(), Failure> {
        controller.object_dragged_base(context, view, action)
    }
    /// `OnSubviewChanged`: nothing by default; `hooks.on_subview_changed(c, &ctx, &old, &new)`.
    fn on_subview_changed(
        &mut self,
        controller: &mut AisViewController,
        context: &Handle<AisInteractiveContext>,
        old_view: &Handle<dyn V3dViewSource>,
        new_view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let _ = (controller, context, old_view, new_view);
        Ok(())
    }
}

/// `myRubberBand`, `myDragObject`, `myDragOwner` and the installed callbacks.
pub(super) struct ViewControllerDispatchState {
    pub(super) rubber_band: Handle<AisRubberBand>,
    pub(super) drag_object: Handle<dyn AisInteractiveObject>,
    pub(super) drag_owner: Handle<dyn SelectMgrEntityOwnerSource>,
    pub(super) hooks: Option<Box<dyn AisViewControllerHooks>>,
}
impl ViewControllerDispatchState {
    /// The light blue half transparent rubber band in the upper-left 2D overlay.
    pub(super) fn new() -> Result<Self, Failure> {
        let rubber_band = AisRubberBand::with_filling(
            Color::named(NamedColor::LIGHTBLUE),
            AspectTypeOfLine::Solid,
            Color::named(NamedColor::LIGHTBLUE4),
            0.5,
            1.0,
            true,
        )?;
        rubber_band.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP_OSD);
        let persistence = Handle::new(Graphic3dTransformPers::with_corner(
            Graphic3dTransModeFlags::TMF_2D,
            AspectTypeOfTriedronPosition::LEFT_UPPER,
            Vec2::new(0, 0),
        )?);
        rubber_band.set_transform_persistence(&persistence.cast());
        rubber_band.set_display_mode(0);
        rubber_band.set_mutable(true);
        Ok(Self {
            rubber_band,
            drag_object: Handle::default(),
            drag_owner: Handle::default(),
            hooks: None,
        })
    }
}

/// The XR input state (`myXRPrsDevices` .. `myXRTurnAngle`).
pub(super) struct ViewControllerXrState {
    pub(super) prs_devices_lower: i32,
    pub(super) prs_devices: Vec<Handle<AisXrTrackedDevice>>,
    pub(super) laser_tele_color: Color,
    pub(super) laser_pick_color: Color,
    pub(super) last_teleport_hand: AspectXRTrackedDeviceRole,
    pub(super) last_picking_hand: AspectXRTrackedDeviceRole,
    pub(super) teleport_haptic: AspectXRHapticActionData,
    pub(super) picking_haptic: AspectXRHapticActionData,
    pub(super) select_haptic: AspectXRHapticActionData,
    pub(super) last_pick_depth_left: f64,
    pub(super) last_pick_depth_right: f64,
    pub(super) turn_angle: f64,
}
impl Default for ViewControllerXrState {
    fn default() -> Self {
        Self {
            prs_devices_lower: 0,
            prs_devices: vec![Handle::default()],
            laser_tele_color: Color::named(NamedColor::GREEN),
            laser_pick_color: Color::named(NamedColor::BLUE),
            last_teleport_hand: AspectXRTrackedDeviceRole::Other,
            last_picking_hand: AspectXRTrackedDeviceRole::Other,
            teleport_haptic: AspectXRHapticActionData::default(),
            picking_haptic: AspectXRHapticActionData::default(),
            select_haptic: AspectXRHapticActionData::default(),
            last_pick_depth_left: rustycad_tkernel::precision::infinite(),
            last_pick_depth_right: rustycad_tkernel::precision::infinite(),
            turn_angle: std::f64::consts::FRAC_PI_4,
        }
    }
}

impl AisViewController {
    /// Install the virtual callbacks (replacing previous ones); `controller.set_hooks(Box::new(h))`.
    pub fn set_hooks(&mut self, hooks: Box<dyn AisViewControllerHooks>) {
        self.dispatch.hooks = Some(hooks);
    }
    /// Remove the installed callbacks, restoring the native defaults; `controller.take_hooks()`.
    pub fn take_hooks(&mut self) -> Option<Box<dyn AisViewControllerHooks>> {
        self.dispatch.hooks.take()
    }
    /// `myRubberBand`: the selection rubber band presentation; `controller.rubber_band()`.
    pub fn rubber_band(&self) -> &Handle<AisRubberBand> {
        &self.dispatch.rubber_band
    }
    /// `myDragObject`: the object being dragged (null when none); `controller.drag_object()`.
    pub fn drag_object(&self) -> &Handle<dyn AisInteractiveObject> {
        &self.dispatch.drag_object
    }
    /// `myDragOwner`: the detected owner of the dragged object; `controller.drag_owner()`.
    pub fn drag_owner(&self) -> &Handle<dyn SelectMgrEntityOwnerSource> {
        &self.dispatch.drag_owner
    }
    /// `OnSelectionChanged` (virtual): the installed callback or nothing.
    /// Example: `controller.on_selection_changed(&ctx, &view)?`.
    pub fn on_selection_changed(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let Some(mut hooks) = self.dispatch.hooks.take() else {
            return Ok(());
        };
        let result = hooks.on_selection_changed(self, context, view);
        self.dispatch.hooks.get_or_insert(hooks);
        result
    }
    /// `OnObjectDragged` (virtual): the installed callback or the native dragging.
    /// Example: `controller.on_object_dragged(&ctx, &view, AisDragAction::Start)?`.
    pub fn on_object_dragged(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        action: AisDragAction,
    ) -> Result<(), Failure> {
        let Some(mut hooks) = self.dispatch.hooks.take() else {
            return self.object_dragged_base(context, view, action);
        };
        let result = hooks.on_object_dragged(self, context, view, action);
        self.dispatch.hooks.get_or_insert(hooks);
        result
    }
    /// `OnSubviewChanged` (virtual): the installed callback or nothing.
    /// Example: `controller.on_subview_changed(&ctx, &old, &new)?`.
    pub fn on_subview_changed(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        old_view: &Handle<dyn V3dViewSource>,
        new_view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let Some(mut hooks) = self.dispatch.hooks.take() else {
            return Ok(());
        };
        let result = hooks.on_subview_changed(self, context, old_view, new_view);
        self.dispatch.hooks.get_or_insert(hooks);
        result
    }
    /// The native `AIS_ViewController::OnObjectDragged` body: start dragging the detected
    /// object when it accepts, forward confirm/update/abort/stop to it (selecting its global
    /// owner while it moves) and forget it on stop or abort.
    /// Example: `controller.object_dragged_base(&ctx, &view, AisDragAction::Stop)?`.
    pub fn object_dragged_base(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        action: AisDragAction,
    ) -> Result<(), Failure> {
        let (from, to) = (
            self.render_input.dragging.point_start,
            self.render_input.dragging.point_to,
        );
        match action {
            AisDragAction::Start => {
                self.dispatch.drag_object = Handle::default();
                self.dispatch.drag_owner = Handle::default();
                if !context.has_detected() {
                    return Ok(());
                }
                let owner = context.detected_owner().clone();
                let detected: Handle<dyn AisInteractiveObject> = owner
                    .get()
                    .map_or_else(Handle::default, |o| o.selectable().cast());
                let object = detected.get().ok_or_else(|| {
                    Failure::with_message(
                        "Standard_NullObject",
                        "AIS_ViewController::OnObjectDragged: detected object is not interactive",
                    )
                })?;
                if object.process_dragging(context, view, &owner, from, to, action) {
                    self.dispatch.drag_object = detected.clone();
                    self.dispatch.drag_owner = owner;
                }
                Ok(())
            }
            AisDragAction::Confirmed => {
                if let Some(object) = self.dispatch.drag_object.clone().get() {
                    object.process_dragging(
                        context,
                        view,
                        &self.dispatch.drag_owner,
                        from,
                        to,
                        action,
                    );
                }
                Ok(())
            }
            AisDragAction::Update => {
                let object = self.dispatch.drag_object.clone();
                let Some(object) = object.get() else {
                    return Ok(());
                };
                let global = object.global_sel_owner();
                if !global.is_null() {
                    context.set_selected_state(&global, true)?;
                }
                object.process_dragging(context, view, &self.dispatch.drag_owner, from, to, action);
                view.invalidate();
                Ok(())
            }
            AisDragAction::Abort | AisDragAction::Stop => {
                if self.dispatch.drag_object.is_null() {
                    return Ok(());
                }
                if action == AisDragAction::Abort {
                    self.render_input.dragging.point_to = self.render_input.dragging.point_start;
                    self.on_object_dragged(context, view, AisDragAction::Update)?;
                    let object = self.dispatch.drag_object.clone();
                    if let Some(object) = object.get() {
                        let start = self.render_input.dragging.point_start;
                        object.process_dragging(
                            context,
                            view,
                            &self.dispatch.drag_owner,
                            start,
                            start,
                            action,
                        );
                    }
                }
                let object = self.dispatch.drag_object.clone();
                let Some(object) = object.get() else {
                    return Ok(());
                };
                let global = object.global_sel_owner();
                if !global.is_null() {
                    context.set_selected_state(&global, false)?;
                }
                let (from, to) = (
                    self.render_input.dragging.point_start,
                    self.render_input.dragging.point_to,
                );
                object.process_dragging(context, view, &self.dispatch.drag_owner, from, to, action);
                view.invalidate();
                self.dispatch.drag_object = Handle::default();
                self.dispatch.drag_owner = Handle::default();
                Ok(())
            }
        }
    }
}
