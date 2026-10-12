//! Native directional light dragging retains its initial frame and picks on its sensitive sphere.
use super::*;
use crate::v3d::V3dViewSource;
use crate::{prs_mgr::*, select_mgr::*, v3d::V3dView};
use rustycad_tkernel::precision;
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::*;
use rustycad_tkservice::graphic3d::*;
/// Native drag routing with typed pick/aspect failures; `ais_light_source_process_dragging(&light,&context,&view,&owner,from,to,AisDragAction::Start)?`.
pub fn ais_light_source_process_dragging<O: AisLightSourceSource + ?Sized>(
    object: &O,
    context: &Handle<AisInteractiveContext>,
    view: &Handle<dyn V3dViewSource>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    from: Vec2<i32>,
    to: Vec2<i32>,
    action: AisDragAction,
) -> Result<bool, Failure> {
    if object.light().light_type() != Graphic3dTypeOfLightSource::Directional {
        return Ok(false);
    }
    dispatch_light_drag(object, context, view, owner, from, to, action)
}
fn dispatch_light_drag<O: AisLightSourceSource + ?Sized>(
    object: &O,
    context: &Handle<AisInteractiveContext>,
    view: &Handle<dyn V3dViewSource>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    from: Vec2<i32>,
    to: Vec2<i32>,
    action: AisDragAction,
) -> Result<bool, Failure> {
    match action {
        AisDragAction::Start => start_light_drag(object),
        AisDragAction::Confirmed | AisDragAction::Abort => Ok(true),
        AisDragAction::Update => {
            update_light_drag(object, context, view, owner, from, to)?;
            Ok(true)
        }
        AisDragAction::Stop => stop_light_drag(object, context),
    }
}
fn update_light_drag<O: AisLightSourceSource + ?Sized>(
    object: &O,
    context: &AisInteractiveContext,
    view: &V3dView,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    from: Vec2<i32>,
    to: Vec2<i32>,
) -> Result<(), Failure> {
    let sphere = prepare_light_drag(object)?;
    let selector = context.main_selector();
    selector.pick_point(from.x, from.y, view)?;
    let start = sphere.last_detected_point();
    sphere.reset_last_detected_point();
    selector.pick_point(to.x, to.y, view)?;
    let current = sphere.last_detected_point();
    if current.x() != f64::MAX && start.distance(&current) > precision::confusion() {
        apply_light_drag(object, context, owner, start, current)?;
    }
    Ok(())
}
fn apply_light_drag<O: AisLightSourceSource + ?Sized>(
    object: &O,
    context: &AisInteractiveContext,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    start: Point3,
    current: Point3,
) -> Result<(), Failure> {
    let transform = light_drag_rotation(start, current);
    let initial = object.light_source_base().visual.read().start_transform;
    object.set_local_transformation(&initial.multiplied(transform))?;
    let mode = if object.has_hilight_mode() {
        object.hilight_mode()
    } else {
        0
    };
    owner.update_highlight_trsf(&context.current_viewer(), &context.main_prs_mgr(), mode)
}

fn start_light_drag<O: AisLightSourceSource + ?Sized>(object: &O) -> Result<bool, Failure> {
    object.light_source_base().visual.write().start_transform = object.local_transformation();
    Ok(true)
}
fn stop_light_drag<O: AisLightSourceSource + ?Sized>(
    object: &O,
    context: &AisInteractiveContext,
) -> Result<bool, Failure> {
    object
        .get_hilight_presentation(&context.main_prs_mgr())?
        .clear(false);
    Ok(false)
}

fn light_drag_rotation(start: Point3, current: Point3) -> Transform3 {
    let mut rotation = Quaternion::default();
    rotation.set_rotation(
        Vector3::new(start.x(), start.y(), start.z()),
        Vector3::new(current.x(), current.y(), current.z()),
    );
    let mut transform = Transform3::default();
    transform.set_rotation_quaternion(rotation);
    transform
}

fn prepare_light_drag<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Result<Handle<crate::select3d::Select3DSensitiveSphere>, Failure> {
    let sphere = object
        .light_source_base()
        .visual
        .read()
        .sensitive_sphere
        .clone();
    sphere.reset_last_detected_point();
    let initial = object.light_source_base().visual.read().start_transform;
    object.set_local_transformation(&initial)?;
    Ok(sphere)
}
