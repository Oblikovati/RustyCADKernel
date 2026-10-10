//! `PrsMgr_PresentableObject::DumpJson`.
use super::PrsMgrPresentableObject;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::graphic3d::{
    Graphic3dPresentationAttributesSource as _, Graphic3dStructureSource as _,
};
use rustycad_tkservice::json_dump::{
    class_begin, field_dumped, field_integer, field_pointer, field_real,
};
use std::io;

/// The native `PrsMgr_PresentableObject::DumpJson` body, for the overrides that dump it.
/// Example: `prs_mgr_presentable_dump_json(&*object, &mut stream, -1)?`.
pub fn prs_mgr_presentable_dump_json<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "PrsMgr_PresentableObject")?;
    let f = object.presentable_base().read();
    field_pointer(stream, "myParent", f.parent.object_address() as usize)?;
    for prs in f.presentations.iter().filter_map(Handle::get) {
        field_dumped(stream, depth, "aPresentation.get()", &|s, d| {
            prs.dump_json(s, d)
        })?;
    }
    if let Some(planes) = f.clip_planes.get() {
        field_dumped(stream, depth, "myClipPlanes.get()", &|s, d| {
            planes.dump_json(s, d)
        })?;
    }
    for (name, drawer) in [
        ("myDrawer.get()", &f.drawer),
        ("myHilightDrawer.get()", &f.hilight_drawer),
        ("myDynHilightDrawer.get()", &f.dyn_hilight_drawer),
    ] {
        if let Some(drawer) = drawer.get() {
            field_dumped(stream, depth, name, &|s, d| drawer.dump_json(s, d))?;
        }
    }
    if let Some(pers) = f.transform_persistence.get() {
        field_dumped(stream, depth, "myTransformPersistence.get()", &|s, d| {
            pers.dump_json(s, d)
        })?;
    }
    for (name, datum) in [
        ("myLocalTransformation.get()", &f.local_transformation),
        ("myTransformation.get()", &f.transformation),
        (
            "myCombinedParentTransform.get()",
            &f.combined_parent_transform,
        ),
    ] {
        if let Some(datum) = datum.get() {
            field_dumped(stream, depth, name, &|s, d| datum.dump_json(s, d))?;
        }
    }
    for child in f.children.iter().filter_map(Handle::get) {
        field_dumped(stream, depth, "aChildObject.get()", &|s, d| {
            child.dump_json(s, d)
        })?;
    }
    let inverse = f.inv_transformation;
    field_dumped(stream, depth, "&myInvTransformation", &|s, d| {
        inverse.dump_json(s, d)
    })?;
    field_integer(
        stream,
        "myTypeOfPresentation3d",
        f.type_of_presentation3d as i32,
    )?;
    field_integer(
        stream,
        "myCurrentFacingModel",
        f.current_facing_model as i32,
    )?;
    field_real(stream, "myOwnWidth", f64::from(f.own_width))?;
    for (name, flag) in [
        ("hasOwnColor", f.has_own_color),
        ("hasOwnMaterial", f.has_own_material),
        ("myInfiniteState", f.infinite_state),
        ("myIsMutable", f.is_mutable),
        ("myHasOwnPresentations", f.has_own_presentations),
        ("myToPropagateVisualState", f.to_propagate_visual_state),
    ] {
        field_integer(stream, name, i32::from(flag))?;
    }
    Ok(())
}
