//! Part groups preserve native allocation order and omit arrows from the selection map.
use super::{
    AisTrihedron, AisTrihedronSource,
    trihedron::{TRI_AXIS_PARTS, TRI_PLANE_PARTS},
};
use crate::{prs_mgr::*, prs3d::*};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::*;
/// Compute cached geometry and groups for display mode zero; `ais_trihedron_compute(&*trihedron, &prs, 0)?`.
pub fn ais_trihedron_compute<O: AisTrihedronSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    if mode != 0 {
        return Ok(());
    }
    prs.set_infinite_state(true);
    let frame = object.component().ax2()?;
    super::ais_trihedron_update_primitives(
        object,
        &object.attributes().datum_aspect(),
        object.datum_display_mode(),
        frame.location(),
        [frame.x_direction(), frame.y_direction(), frame.direction()],
    )?;
    ais_trihedron_compute_presentation(object, prs)
}
pub(super) fn trihedron_part_aspect<O: AisTrihedronSource + ?Sized>(
    object: &O,
    part: Prs3dDatumParts,
) -> Handle<dyn Graphic3dAspectsSource> {
    let datum = object.attributes().datum_aspect();
    if object.datum_display_mode() == Prs3dDatumMode::Shaded {
        return datum.shading_aspect(part).aspect().cast();
    }
    if part == Prs3dDatumParts::Origin {
        return datum.point_aspect().aspect().cast();
    }
    datum.line_aspect(part).aspect().cast()
}
fn add_trihedron_group<O: AisTrihedronSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    part: Prs3dDatumParts,
    retain: bool,
) -> Result<(), Failure> {
    let group = prs.new_group()?;
    group.set_group_primitives_aspect(&trihedron_part_aspect(object, part));
    group.add_primitive_array(&object.array_of_primitives(part), true);
    if retain {
        object.trihedron_base().fields.write().part_groups[part as usize] = group;
    }
    Ok(())
}
/// Populate groups from the existing cache; `ais_trihedron_compute_presentation(&*trihedron, &prs)?`.
pub fn ais_trihedron_compute_presentation<O: AisTrihedronSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    object.trihedron_base().fields.write().part_groups = std::array::from_fn(|_| Handle::default());
    let datum = object.attributes().datum_aspect();
    if datum.draw_datum_part(Prs3dDatumParts::Origin) {
        add_trihedron_group(object, prs, Prs3dDatumParts::Origin, true)?;
    } else {
        prs.new_group()?;
    }
    for part in TRI_AXIS_PARTS {
        add_trihedron_axis_groups(object, prs, part)?;
    }
    add_trihedron_labels(object, prs)?;
    for part in TRI_PLANE_PARTS {
        add_trihedron_hidden_plane(object, prs, part)?;
    }
    Ok(())
}
fn add_trihedron_axis_groups<O: AisTrihedronSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    part: Prs3dDatumParts,
) -> Result<(), Failure> {
    let datum = object.attributes().datum_aspect();
    if !datum.draw_datum_part(part) {
        return Ok(());
    }
    add_trihedron_group(object, prs, part, true)?;
    let arrow = Prs3dDatumAspect::arrow_part_for_axis(part);
    if datum.draw_datum_part(arrow) {
        add_trihedron_group(object, prs, arrow, false)?;
    }
    Ok(())
}
fn add_trihedron_labels<O: AisTrihedronSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let datum = object.attributes().datum_aspect();
    if !datum.to_draw_labels() {
        return Ok(());
    }
    let frame = object.component().ax2()?;
    let directions = [frame.x_direction(), frame.y_direction(), frame.direction()];
    for (part, direction) in TRI_AXIS_PARTS.into_iter().zip(directions) {
        if !datum.draw_datum_part(part) {
            continue;
        }
        let tip = rustycad_tkmath::gp::Point3::from(
            frame.location().xyz() + direction.xyz() * datum.axis_length(part),
        );
        Prs3dText::draw(
            &prs.new_group()?,
            &datum.text_aspect(part),
            &object.label(part),
            &tip,
        );
    }
    Ok(())
}
fn add_trihedron_hidden_plane<O: AisTrihedronSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    part: Prs3dDatumParts,
) -> Result<(), Failure> {
    if !object.attributes().datum_aspect().draw_datum_part(part) {
        return Ok(());
    }
    let group = prs.new_group()?;
    // OCCT adds geometry first, then applies the invisible aspect to the whole group.
    group.add_primitive_array(&object.array_of_primitives(part), true);
    group.set_group_primitives_aspect(
        &object
            .trihedron_base()
            .fields
            .read()
            .hidden_line_aspect
            .cast(),
    );
    object.trihedron_base().fields.write().part_groups[part as usize] = group;
    Ok(())
}
impl PrsMgrPresentableObject for AisTrihedron {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.trihedron_base().interactive.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_trihedron_compute(self, prs, mode)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_set_color(self, color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        super::ais_trihedron_unset_color(self)
    }
    crate::select_mgr_presentable_overrides!(dump = super::ais_trihedron_dump_json);
}
