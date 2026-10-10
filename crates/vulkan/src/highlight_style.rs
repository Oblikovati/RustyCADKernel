//! OpenGL color highlighting overrides vertex/bound color without replacing line width/pattern.
use crate::{PrimitiveDraw, RenderError, error::rejected};
use rustycad_tkservice::{aspect::AspectTypeOfHighlightMethod, graphic3d::*};
pub(crate) fn validate_structure(structure: &Graphic3dCStructureFields) -> Result<(), RenderError> {
    if !structure.clip_planes.is_null() {
        return Err(rejected(
            "structure clipping",
            "an unclipped structure in this renderer slice",
        ));
    }
    let style = &structure.highlight_style;
    if structure.highlight
        && !style.is_null()
        && style.method() != AspectTypeOfHighlightMethod::Color
    {
        return Err(rejected(
            style.method(),
            "color highlighting until bounding-box highlighting is implemented",
        ));
    }
    Ok(())
}
pub(crate) fn apply_highlight(
    draw: &mut PrimitiveDraw,
    structure: &Graphic3dCStructureFields,
) -> Result<(), RenderError> {
    if !structure.highlight || structure.highlight_style.is_null() {
        return Ok(());
    }
    let color = structure.highlight_style.color_rgba();
    let rgba = [
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ];
    let vertices = draw
        .vertices
        .iter()
        .map(|v| crate::RenderVertex::new(v.position, rgba))
        .collect::<Vec<_>>();
    crate::scene::validate_components(&vertices, &draw.clip_transform)?;
    draw.vertices = vertices;
    Ok(())
}
