//! Dynamic highlights use a separate presentation; selected parts change retained groups.
use super::{
    AisInteractiveObjectMembers, AisTrihedronOwnerSource, AisTrihedronSource,
    trihedron::TRI_PLANE_PARTS, trihedron_presentations::trihedron_part_aspect,
};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;
fn trihedron_highlight_aspect<O: AisTrihedronSource + ?Sized>(
    object: &O,
    part: Prs3dDatumParts,
    style: &Handle<Prs3dDrawer>,
) -> Handle<dyn Graphic3dAspectsSource> {
    if TRI_PLANE_PARTS.contains(&part) {
        return style.line_aspect().aspect().cast();
    }
    if object.datum_display_mode() == Prs3dDatumMode::Shaded {
        return style.shading_aspect().aspect().cast();
    }
    if part == Prs3dDatumParts::Origin {
        return style.point_aspect().aspect().cast();
    }
    style.line_aspect().aspect().cast()
}
/// Highlight only the cached owner part, or the whole object for generic owners; `ais_trihedron_hilight_owner(&*trihedron, &manager, &style, &owner)?`.
pub fn ais_trihedron_hilight_owner<O: AisTrihedronSource + ?Sized>(
    object: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<(), Failure> {
    let owner = owner.cast::<dyn AisTrihedronOwnerSource>();
    if owner.is_null() {
        let mode = if object.has_hilight_mode() {
            object.hilight_mode()
        } else {
            0
        };
        return manager.color(
            &object.presentable_base().this(),
            style,
            mode,
            &Handle::default(),
            GRAPHIC3D_ZLAYER_ID_TOP,
        );
    }
    let prs = object.get_hilight_presentation(manager)?;
    if prs.is_null() {
        return Ok(());
    }
    prs.clear(true);
    let group = prs.current_group()?;
    group.set_group_primitives_aspect(&trihedron_highlight_aspect(
        object,
        owner.datum_part(),
        style,
    ));
    group.add_primitive_array(&object.array_of_primitives(owner.datum_part()), true);
    let layer = if style.z_layer() != GRAPHIC3D_ZLAYER_ID_UNKNOWN {
        style.z_layer()
    } else {
        object.attributes().z_layer()
    };
    if prs.get_z_layer() != layer {
        prs.set_z_layer(layer);
    }
    prs.highlight(&style.cast(), true);
    manager.add_to_immediate_list(&prs.cast());
    Ok(())
}
/// Change existing selected-part group aspects once; `ais_trihedron_hilight_selected(&*trihedron, &manager, &owners)?`.
pub fn ais_trihedron_hilight_selected<O: AisTrihedronSource + ?Sized>(
    object: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
) -> Result<(), Failure> {
    if owners.is_empty() || !object.has_interactive_context() {
        return Ok(());
    }
    let own_style = object.hilight_attributes();
    let style = if own_style.is_null() {
        object.get_context().selection_style().clone()
    } else {
        own_style
    };
    for owner in owners {
        select_trihedron_part(object, manager, &style, owner)?;
    }
    Ok(())
}
fn select_trihedron_part<O: AisTrihedronSource + ?Sized>(
    object: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<(), Failure> {
    let owner = owner.cast::<dyn AisTrihedronOwnerSource>();
    if owner.is_null() {
        return manager.color(
            &object.presentable_base().this(),
            style,
            0,
            &Handle::default(),
            GRAPHIC3D_ZLAYER_ID_UNKNOWN,
        );
    }
    let part = owner.datum_part();
    let fields = object.trihedron_base().fields.read();
    let group = fields.part_groups[part as usize].clone();
    if group.is_null() || fields.selected_parts.contains(&part) {
        return Ok(());
    }
    drop(fields);
    group.set_group_primitives_aspect(&trihedron_highlight_aspect(object, part, style));
    object
        .trihedron_base()
        .fields
        .write()
        .selected_parts
        .push(part);
    Ok(())
}
/// Restore current datum aspects and hidden plane lines; `ais_trihedron_clear_selected(&*trihedron)`.
pub fn ais_trihedron_clear_selected<O: AisTrihedronSource + ?Sized>(object: &O) {
    let parts = object.trihedron_base().fields.read().selected_parts.clone();
    for part in parts {
        let group = object.trihedron_base().fields.read().part_groups[part as usize].clone();
        if group.is_null() {
            continue;
        }
        let aspect = if TRI_PLANE_PARTS.contains(&part) {
            object
                .trihedron_base()
                .fields
                .read()
                .hidden_line_aspect
                .cast()
        } else {
            trihedron_part_aspect(object, part)
        };
        group.set_group_primitives_aspect(&aspect);
    }
    object
        .trihedron_base()
        .fields
        .write()
        .selected_parts
        .clear();
}
/// Emit native trihedron flags after the inherited dump; `ais_trihedron_dump_json(&*trihedron, &mut stream, 0)?`.
pub fn ais_trihedron_dump_json<O: AisTrihedronSource + ?Sized>(
    object: &O,
    stream: &mut rustycad_tkernel::standard::dump::DumpStream,
    depth: i32,
) -> std::io::Result<()> {
    use rustycad_tkservice::json_dump::{base_class, class_begin, field_integer};
    class_begin(stream, "AIS_Trihedron")?;
    base_class(stream, depth, &|stream, depth| {
        super::ais_interactive_object_dump_json(object, stream, depth)
    })?;
    let fields = object.trihedron_base().fields.read();
    field_integer(stream, "myHasOwnSize", i32::from(fields.has_own_size))?;
    field_integer(
        stream,
        "myHasOwnTextColor",
        i32::from(fields.has_own_text_color),
    )?;
    field_integer(
        stream,
        "myHasOwnArrowColor",
        i32::from(fields.has_own_arrow_color),
    )?;
    field_integer(stream, "myTrihDispMode", fields.datum_mode as i32)
}
