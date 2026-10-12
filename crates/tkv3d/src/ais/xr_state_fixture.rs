//! Actual empty/null/valid/indexed/incomplete model states and native mutation schedule.
use super::xr_recording_group::XrRecordingEvents;
use super::*;
use crate::{prs_mgr::*, v3d::V3dViewer};
use rustycad_tkernel::{
    quantity::{Color, ColorSpace, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{
    aspect::AspectXRTrackedDeviceRole, graphic3d::*, null_driver::NullGraphicDriver,
};
pub(super) fn xr_state_model(kind: i32) -> Handle<Graphic3dArrayOfTriangles> {
    if kind <= 1 {
        return Handle::default();
    }
    let model =
        Handle::new(Graphic3dArrayOfTriangles::new(3, if kind == 3 { 3 } else { 0 }, 0).unwrap());
    if kind == 4 {
        return model;
    }
    append_xr_model_vertices(&model, kind);
    model
}
fn append_xr_model_vertices(model: &Handle<Graphic3dArrayOfTriangles>, kind: i32) {
    use rustycad_tkmath::gp::Point3;
    model.add_vertex_pnt(&Point3::new(-1., -2., 0.)).unwrap();
    model.add_vertex_pnt(&Point3::new(2., -2., 0.)).unwrap();
    if kind != 5 {
        model.add_vertex_pnt(&Point3::new(0., 3., 0.)).unwrap();
    }
    if kind == 3 {
        for edge in 1..=3 {
            model.add_edge(edge).unwrap();
        }
    }
}
pub(super) fn xr_state_device(
    ordinal: i32,
) -> (Handle<AisXrTrackedDevice>, Handle<AisInteractiveContext>) {
    let kind = ordinal / 8;
    let model = xr_state_model(kind);
    let device = if kind == 0 {
        AisXrTrackedDevice::new().unwrap()
    } else {
        AisXrTrackedDevice::with_render_model(&model, &Handle::default()).unwrap()
    };
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    device.set_context(&context);
    device.set_infinite_state(ordinal / 4 % 2 != 0);
    device.set_unit_factor([1., -2., 0., f32::NAN][ordinal as usize % 4]);
    (device, context)
}
pub(super) fn xr_state_operation(
    device: &Handle<AisXrTrackedDevice>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    events: &XrRecordingEvents,
    step: i32,
) -> Result<(), Failure> {
    if step <= 6 {
        return xr_state_early_operation(device, prs, step);
    }
    if step <= 10 {
        return xr_state_color_role_operation(device, step);
    }
    xr_state_late_operation(device, prs, events, step)
}
fn xr_state_early_operation(
    device: &Handle<AisXrTrackedDevice>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    step: i32,
) -> Result<(), Failure> {
    match step {
        0 => device.compute(&Handle::default(), prs, 1),
        1 => xr_first_state_render(device, prs),
        2 | 3 => device.set_laser_length(2.),
        4 => device.set_laser_length(-1.),
        _ => device.set_laser_length(f32::NAN),
    }
}
fn xr_first_state_render(
    device: &Handle<AisXrTrackedDevice>,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    device.set_laser_length(1.)?;
    device.compute(&Handle::default(), prs, 0)
}
fn xr_state_color_role_operation(
    device: &Handle<AisXrTrackedDevice>,
    step: i32,
) -> Result<(), Failure> {
    match step {
        7 => device.set_laser_color(Color::new(0.00001, 0., 1., ColorSpace::Rgb).unwrap()),
        8 | 9 => device.set_laser_color(Color::named(NamedColor::RED)),
        _ => {
            device.set_role(AspectXRTrackedDeviceRole::LeftHand);
            device.set_unit_factor(7.);
            Ok(())
        }
    }
}
fn xr_state_late_operation(
    device: &Handle<AisXrTrackedDevice>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    events: &XrRecordingEvents,
    step: i32,
) -> Result<(), Failure> {
    match step {
        11 | 19 => device.compute(&Handle::default(), prs, 0),
        12 | 13 => xr_faulted_state_length(device, events, step == 12),
        14 => device.set_laser_length(4.),
        15 => device.set_laser_length(0.),
        16 => device.set_laser_length(5.),
        _ => xr_state_allocation_color_operation(device, prs, events, step),
    }
}
fn xr_faulted_state_length(
    device: &Handle<AisXrTrackedDevice>,
    events: &XrRecordingEvents,
    fail: bool,
) -> Result<(), Failure> {
    *events.fail_aspect.write() = fail;
    device.set_laser_length(3.)
}
fn xr_state_allocation_color_operation(
    device: &Handle<AisXrTrackedDevice>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    events: &XrRecordingEvents,
    step: i32,
) -> Result<(), Failure> {
    if step == 17 {
        *events.fail_new_group.write() = prs.number_of_groups() + 1;
        return device.compute(&Handle::default(), prs, 0);
    }
    *events.fail_new_group.write() = -1;
    device.set_laser_color(Color::named(NamedColor::GREEN))
}
