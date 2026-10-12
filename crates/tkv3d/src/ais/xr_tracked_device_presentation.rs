//! Native model/axes geometry and nonvirtual laser refresh preserve group and failure ordering.
use super::{AisXrTrackedDeviceMembers, AisXrTrackedDeviceSource};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, catch_native_failure},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
/// Computes native mode-zero geometry through actual parent virtual interfaces.
/// Example: `ais_xr_tracked_device_compute(&*device,&presentation,0)?`.
pub fn ais_xr_tracked_device_compute<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    if mode != 0 {
        return Ok(());
    }
    catch_native_failure(|| compute_xr_geometry(object, prs))
}
fn compute_xr_geometry<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    prs.set_infinite_state(object.presentable_base().read().infinite_state);
    let group = prs.new_group()?;
    let (triangles, axes) = xr_model_parameters(object);
    if !triangles.is_null() {
        group.set_group_primitives_aspect(&object.attributes().shading_aspect().aspect().cast());
        group.add_primitive_array(&triangles.clone().cast(), true);
    }
    if axes || triangles.is_null() {
        add_xr_axes(object, &group)?;
    }
    let ray = prs.new_group()?;
    object.xr_tracked_device_base().parameters.write().ray_group = ray;
    ais_xr_tracked_device_compute_laser_ray(object)
}
fn xr_model_parameters<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
) -> (Handle<Graphic3dArrayOfTriangles>, bool) {
    let state = object.xr_tracked_device_base().parameters.read();
    (state.triangles.clone(), state.show_axes)
}
fn add_xr_axes<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
    group: &Handle<dyn Graphic3dGroup>,
) -> Result<(), Failure> {
    let size = 0.1_f32 * object.unit_factor();
    group.set_group_primitives_aspect(&object.attributes().line_aspect().aspect().cast());
    let lines = xr_axis_segments(size)?;
    group.add_primitive_array(&lines.cast(), true);
    Ok(())
}
fn xr_axis_segments(size: f32) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
    let lines = Handle::new(Graphic3dArrayOfSegments::new(
        6,
        0,
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR,
    )?);
    populate_xr_axes(&lines, size)?;
    Ok(lines)
}
fn populate_xr_axes(lines: &Handle<Graphic3dArrayOfSegments>, size: f32) -> Result<(), Failure> {
    for (axis, color) in [NamedColor::RED, NamedColor::GREEN, NamedColor::BLUE]
        .into_iter()
        .enumerate()
    {
        let mut coordinates = [0.; 3];
        coordinates[axis] = f64::from(size);
        add_xr_axis(
            lines,
            Point3::new(coordinates[0], coordinates[1], coordinates[2]),
            color,
        )?;
    }
    Ok(())
}
fn add_xr_axis(
    lines: &Handle<Graphic3dArrayOfSegments>,
    end: Point3,
    color: NamedColor,
) -> Result<(), Failure> {
    lines.add_vertex_pnt_color(&Point3::default(), Color::named(color))?;
    lines.add_vertex_pnt_color(&end, Color::named(color))?;
    Ok(())
}
/// Executes the native nonvirtual protected laser refresh without invalidating presentation modes.
/// Example: `ais_xr_tracked_device_compute_laser_ray(&*device)?`.
pub fn ais_xr_tracked_device_compute_laser_ray<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    catch_native_failure(|| refresh_xr_laser(object))
}
fn refresh_xr_laser<O: AisXrTrackedDeviceSource + ?Sized>(object: &O) -> Result<(), Failure> {
    let (group, length, color) = xr_laser_parameters(object);
    if group.is_null() {
        return Ok(());
    }
    // OCCT 3d097a0 tests bounds-based IsEmpty: finite excluded-bounds rays can accumulate.
    if !group.is_empty() {
        group.clear(true);
    }
    if length <= 0. {
        return Ok(());
    }
    let lines = xr_laser_segments(length, color)?;
    group.set_group_primitives_aspect(&object.attributes().line_aspect().aspect().cast());
    group.add_primitive_array(&lines.cast(), false);
    Ok(())
}
fn xr_laser_parameters<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
) -> (Handle<dyn Graphic3dGroup>, f32, Color) {
    let state = object.xr_tracked_device_base().parameters.read();
    (
        state.ray_group.clone(),
        state.laser_length,
        state.laser_color,
    )
}
fn xr_laser_segments(
    length: f32,
    color: Color,
) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
    let lines = Handle::new(Graphic3dArrayOfSegments::new(
        2,
        0,
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR,
    )?);
    lines.add_vertex_pnt_color(&Point3::default(), color)?;
    lines.add_vertex_pnt_color(&Point3::new(0., 0., -f64::from(length)), color)?;
    Ok(lines)
}
