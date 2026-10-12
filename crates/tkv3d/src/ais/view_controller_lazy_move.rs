//! Native detection temporarily fits depth and restores it only after successful MoveTo.
use super::{AisInteractiveContext, AisViewController};
use crate::v3d::V3dViewSource;
use crate::{select_mgr::SelectMgrEntityOwnerSource, v3d::V3dView};
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle, exceptions::null_object},
};
type ActionOwner = Handle<dyn SelectMgrEntityOwnerSource>;
impl AisViewController {
    /// Detect a changed cursor and refresh actual grid echo; `controller.context_lazy_move_to(&context,&view,&pixel)?`.
    pub fn context_lazy_move_to(
        &mut self,
        context: &AisInteractiveContext,
        view: &Handle<dyn V3dViewSource>,
        pixel: &Vec2<i32>,
    ) -> Result<(), Failure> {
        rustycad_tkernel::standard::catch_native_failure(|| {
            self.dispatch_lazy_move_to(context, view, pixel)
        })
    }
    fn dispatch_lazy_move_to(
        &mut self,
        context: &AisInteractiveContext,
        view: &Handle<dyn V3dViewSource>,
        pixel: &Vec2<i32>,
    ) -> Result<(), Failure> {
        if self.previous_move_to == *pixel || self.camera_action_state.had_computed_hlr {
            return Ok(());
        }
        self.previous_move_to = *pixel;
        let (last, picked) = execute_lazy_detection(context, view, pixel)?;
        refresh_lazy_grid_echo(view, pixel, &picked)?;
        invalidate_lazy_detection_views(view, &last, &picked)?;
        Ok(())
    }
}
pub(super) fn camera_action_view(
    view: &Handle<dyn V3dViewSource>,
) -> Result<&dyn V3dViewSource, Failure> {
    view.get().ok_or_else(|| {
        null_object("camera action view=null; expected V3dView for camera/detection operations")
    })
}
fn refresh_lazy_grid_echo(
    view: &Handle<dyn V3dViewSource>,
    pixel: &Vec2<i32>,
    picked: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<(), Failure> {
    let actual = camera_action_view(view)?;
    let viewer = actual.viewer();
    if !actual.is_grid_active()? || !viewer.grid_echo() {
        return Ok(());
    }
    update_lazy_grid_echo(actual, view, pixel, picked.is_null())?;
    actual.invalidate_immediate();
    Ok(())
}

fn execute_lazy_detection(
    context: &AisInteractiveContext,
    view: &Handle<dyn V3dViewSource>,
    pixel: &Vec2<i32>,
) -> Result<(ActionOwner, ActionOwner), Failure> {
    let last = context.detected_owner().clone();
    let actual = camera_action_view(view)?;
    let camera = actual.camera();
    let depth = (camera.z_near(), camera.z_far());
    actual.auto_z_fit()?;
    context.move_to(pixel.x, pixel.y, view, false)?;
    camera.set_z_range(depth.0, depth.1)?;
    let picked = context.detected_owner().clone();
    Ok((last, picked))
}
fn invalidate_lazy_detection_views(
    view: &Handle<dyn V3dViewSource>,
    last: &Handle<dyn SelectMgrEntityOwnerSource>,
    picked: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<(), Failure> {
    if last == picked && (picked.is_null() || !picked.is_forced_hilight()) {
        return Ok(());
    }
    let viewer = camera_action_view(view)?.viewer();
    let mut cursor = viewer.active_view_iterator();
    while cursor.more() {
        let other = rustycad_tkernel::standard::HandleReference::observe(&cursor.value_ref());
        other.invalidate_immediate();
        cursor.next();
    }
    Ok(())
}

fn update_lazy_grid_echo(
    actual: &V3dView,
    view: &Handle<dyn V3dViewSource>,
    pixel: &Vec2<i32>,
    missed: bool,
) -> Result<(), Failure> {
    let viewer = actual.viewer();
    if missed {
        if let Some((_, echo)) = actual.convert_to_grid_echo(pixel.x, pixel.y)? {
            viewer.show_grid_echo(view, &echo)?;
        } else {
            viewer.hide_grid_echo(view);
        }
    } else {
        viewer.hide_grid_echo(view);
    }
    Ok(())
}
