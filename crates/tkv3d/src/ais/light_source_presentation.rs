//! Native presentation dispatch honors virtual branch hooks and live light changes.
use super::*;
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::*};
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::{AsciiString, ExtendedString},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
/// Native light dispatch; `ais_light_source_compute(&object,&presentation,0)?`.
pub fn ais_light_source_compute<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    prs.set_infinite_state(object.is_infinite());
    if mode != 0 && mode != 1 {
        return Ok(());
    }
    if mode == 0 {
        refresh_light_presentation(object)?;
    }
    draw_light_branch(object, prs, mode)?;
    if object.to_display_name() {
        light_name(object, prs)?;
    }
    Ok(())
}
fn refresh_light_presentation<O: AisLightSourceSource + ?Sized>(object: &O) -> Result<(), Failure> {
    object.update_light_aspects()?;
    object.update_light_transform_persistence()?;
    object.update_light_local_transformation()
}
fn light_name<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let name = light_label(object);
    Prs3dText::draw(
        &prs.new_group()?,
        &object.attributes().text_aspect(),
        &name,
        &Point3::default(),
    );
    Ok(())
}

fn draw_light_branch<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    match object.light().light_type() {
        Graphic3dTypeOfLightSource::Ambient => object.compute_ambient(prs, mode)?,
        Graphic3dTypeOfLightSource::Directional => object.compute_directional(prs, mode)?,
        Graphic3dTypeOfLightSource::Positional => object.compute_positional(prs, mode)?,
        Graphic3dTypeOfLightSource::Spot => object.compute_spot(prs, mode)?,
    }
    Ok(())
}

fn light_label<O: AisLightSourceSource + ?Sized>(object: &O) -> ExtendedString {
    let persistence = object.presentable_base().read().transform_persistence;
    let prefix = if persistence
        .get()
        .is_some_and(|pers| pers.is_trihedron_or_2d())
    {
        "\n"
    } else {
        "   "
    };
    let mut name = prefix.as_bytes().to_vec();
    name.extend_from_slice(object.light().name().as_bytes());
    ExtendedString::from_ascii(&AsciiString::from_raw(&name))
}
