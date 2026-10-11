//! Preserve OCCT 3d097a0 datum ownership, including its asymmetric unset behavior.
use super::{AisTrihedronSource, trihedron::TRI_AXIS_PARTS};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
/// Copy the linked datum once; `ais_trihedron_own_datum(&*trihedron)?`.
pub fn ais_trihedron_own_datum<O: AisTrihedronSource + ?Sized>(object: &O) -> Result<(), Failure> {
    let drawer = object.attributes();
    if drawer.has_own_datum_aspect() {
        return Ok(());
    }
    let aspect = Handle::new(Prs3dDatumAspect::new()?);
    drawer.set_datum_aspect(aspect.clone());
    if drawer.has_link() {
        aspect.copy_aspects_from(&drawer.link().datum_aspect());
    }
    Ok(())
}
/// Own all axis lengths and schedule geometry/selection; `ais_trihedron_set_size(&*trihedron, 12.)?`.
pub fn ais_trihedron_set_size<O: AisTrihedronSource + ?Sized>(
    object: &O,
    size: f64,
) -> Result<(), Failure> {
    object.trihedron_base().fields.write().has_own_size = true;
    ais_trihedron_own_datum(object)?;
    object
        .attributes()
        .datum_aspect()
        .set_axis_length(size, size, size);
    object.set_to_update_all();
    object.update_selection(-1);
    Ok(())
}
/// Preserve native color-dependent size restoration; `ais_trihedron_unset_size(&*trihedron)?`.
pub fn ais_trihedron_unset_size<O: AisTrihedronSource + ?Sized>(object: &O) -> Result<(), Failure> {
    if !object.has_own_size() {
        return Ok(());
    }
    object.trihedron_base().fields.write().has_own_size = false;
    if object.has_color() {
        let drawer = object.attributes();
        let source = if drawer.has_link() {
            drawer.link().datum_aspect()
        } else {
            Handle::new(Prs3dDatumAspect::new()?)
        };
        drawer.datum_aspect().set_axis_length(
            source.axis_length(TRI_AXIS_PARTS[0]),
            source.axis_length(TRI_AXIS_PARTS[1]),
            source.axis_length(TRI_AXIS_PARTS[2]),
        );
    } else {
        // OCCT 3d097a0 leaves the own datum and its lengths intact in this branch.
        object.set_to_update_all();
    }
    object.update_selection(-1);
    Ok(())
}
fn missing_part(part: Prs3dDatumParts, expected: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!("AIS_Trihedron part {part:?}: expected {expected}"),
        "",
    )
}
/// Assign both native part aspects without scheduling updates; `ais_trihedron_part_color(&*trihedron, Prs3dDatumParts::XAxis, red)?`.
pub fn ais_trihedron_part_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
    part: Prs3dDatumParts,
    color: Color,
) -> Result<(), Failure> {
    ais_trihedron_own_datum(object)?;
    let datum = object.attributes().datum_aspect();
    let shading = datum.shading_aspect(part);
    if shading.is_null() {
        return Err(missing_part(part, "a drawable shading part"));
    }
    shading.set_color(color, AspectTypeOfFacingModel::BothSide)?;
    if part == Prs3dDatumParts::Origin {
        return Ok(());
    }
    let line = datum.line_aspect(part);
    if line.is_null() {
        return Err(missing_part(part, "a drawable line part"));
    }
    line.set_color(color);
    Ok(())
}
/// Read the aspect for the current datum mode; `ais_trihedron_get_part_color(&*trihedron, Prs3dDatumParts::XAxis)?`.
pub fn ais_trihedron_get_part_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
    part: Prs3dDatumParts,
) -> Result<Color, Failure> {
    let datum = object.attributes().datum_aspect();
    if object.datum_display_mode() == Prs3dDatumMode::Shaded {
        let shading = datum.shading_aspect(part);
        if shading.is_null() {
            return Err(missing_part(part, "a drawable shading part"));
        }
        return Ok(shading.color(AspectTypeOfFacingModel::BothSide));
    }
    let line = datum.line_aspect(part);
    if line.is_null() {
        return Err(missing_part(part, "a drawable wire line part"));
    }
    Ok(line.aspect().interior_color())
}
/// Assign one text aspect without setting the native ownership flag; `ais_trihedron_text_part_color(&*trihedron, Prs3dDatumParts::XAxis, red)?`.
pub fn ais_trihedron_text_part_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
    part: Prs3dDatumParts,
    color: Color,
) -> Result<(), Failure> {
    ais_trihedron_own_datum(object)?;
    let text = object.attributes().datum_aspect().text_aspect(part);
    if text.is_null() {
        return Err(missing_part(part, "an axis text part"));
    }
    text.set_color(color);
    Ok(())
}
/// Assign all axis labels; `ais_trihedron_text_color(&*trihedron, red)?`.
pub fn ais_trihedron_text_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
    color: Color,
) -> Result<(), Failure> {
    for part in TRI_AXIS_PARTS {
        ais_trihedron_text_part_color(object, part, color)?;
    }
    Ok(())
}
/// Assign an axis arrow without changing the global arrow aspect; `ais_trihedron_arrow_part_color(&*trihedron, Prs3dDatumParts::XAxis, red)?`.
pub fn ais_trihedron_arrow_part_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
    part: Prs3dDatumParts,
    color: Color,
) -> Result<(), Failure> {
    ais_trihedron_own_datum(object)?;
    object.trihedron_base().fields.write().has_own_arrow_color = true;
    ais_trihedron_part_color(object, Prs3dDatumAspect::arrow_part_for_axis(part), color)
}
/// Assign global and individual arrow aspects; `ais_trihedron_arrow_color(&*trihedron, red)?`.
pub fn ais_trihedron_arrow_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
    color: Color,
) -> Result<(), Failure> {
    ais_trihedron_own_datum(object)?;
    object.trihedron_base().fields.write().has_own_arrow_color = true;
    object
        .attributes()
        .datum_aspect()
        .arrow_aspect()
        .set_color(color);
    for part in TRI_AXIS_PARTS {
        ais_trihedron_part_color(object, Prs3dDatumAspect::arrow_part_for_axis(part), color)?;
    }
    Ok(())
}
/// Assign the native draw flag without scheduling updates; `ais_trihedron_draw_arrows(&*trihedron, false)?`.
pub fn ais_trihedron_draw_arrows<O: AisTrihedronSource + ?Sized>(
    object: &O,
    draw: bool,
) -> Result<(), Failure> {
    ais_trihedron_own_datum(object)?;
    object.attributes().datum_aspect().set_draw_arrows(draw);
    Ok(())
}
/// Color the origin only in shaded mode; `ais_trihedron_origin_color(&*trihedron, red)?`.
pub fn ais_trihedron_origin_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
    color: Color,
) -> Result<(), Failure> {
    if object.datum_display_mode() != Prs3dDatumMode::Shaded {
        return Ok(());
    }
    ais_trihedron_part_color(object, Prs3dDatumParts::Origin, color)
}
/// Own the drawer and origin/axis colors, leaving arrows/text alone; `ais_trihedron_set_color(&*trihedron, red)?`.
pub fn ais_trihedron_set_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
    color: Color,
) -> Result<(), Failure> {
    object.presentable_base().fields.write().has_own_color = true;
    object.attributes().set_color(color);
    ais_trihedron_part_color(object, Prs3dDatumParts::Origin, color)?;
    for part in TRI_AXIS_PARTS {
        ais_trihedron_part_color(object, part, color)?;
    }
    Ok(())
}
/// Restore native defaults and ownership flags; `ais_trihedron_unset_color(&*trihedron)?`.
pub fn ais_trihedron_unset_color<O: AisTrihedronSource + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    object.presentable_base().fields.write().has_own_color = false;
    let color = Color::named(NamedColor::LIGHTSTEELBLUE4);
    // OCCT 3d097a0 UnsetColor calls SetColor, restoring hasOwnColor to true.
    ais_trihedron_set_color(object, color)?;
    if object.has_text_color() {
        ais_trihedron_text_color(object, color)?;
        object.trihedron_base().fields.write().has_own_text_color = false;
    }
    if object.has_arrow_color() {
        ais_trihedron_arrow_color(object, color)?;
        object.trihedron_base().fields.write().has_own_arrow_color = false;
    }
    Ok(())
}
