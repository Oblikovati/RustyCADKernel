//! Native selection retains aliased triangle buffers and appends even when initialization returns false.
use super::AisXrTrackedDeviceSource;
use crate::{select_mgr::*, select3d::*};
use rustycad_tkernel::standard::{Failure, Handle, catch_native_failure};
use rustycad_tkmath::toploc::Location;
/// Appends the native model primitive sensitive entity in mode zero.
/// Example: `ais_xr_tracked_device_compute_selection(&*device,&selection,0)?`.
pub fn ais_xr_tracked_device_compute_selection<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    if mode != 0 {
        return Ok(());
    }
    catch_native_failure(|| append_xr_selection(object, selection))
}
fn append_xr_selection<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
    selection: &Handle<SelectMgrSelection>,
) -> Result<(), Failure> {
    let triangles = object
        .xr_tracked_device_base()
        .parameters
        .read()
        .triangles
        .clone();
    if triangles.is_null() {
        return Ok(());
    }
    let selectable = object.presentable_base().this().cast();
    let owner = SelectMgrEntityOwner::with_selectable(&selectable, 0).cast();
    let sensitive = Handle::new(Select3DSensitivePrimitiveArray::new(&owner)?);
    initialize_xr_sensitive(&sensitive, &triangles)?;
    selection.add(&sensitive.cast())
}
fn initialize_xr_sensitive(
    sensitive: &Handle<Select3DSensitivePrimitiveArray>,
    triangles: &Handle<rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles>,
) -> Result<(), Failure> {
    // Native ignores InitTriangulation's bool result; empty/invalid models still append the entity.
    sensitive.init_triangulation(&Select3DPrimArrayInit::whole(
        &triangles.attributes(),
        &triangles.indices(),
        &Location::default(),
        true,
        1,
    ))?;
    Ok(())
}
