//! Surface-only lighting and culling do not reject native line presentations.
use crate::{LineStyle, RenderError, error::rejected};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
pub(crate) fn primitive_style(
    kind: Graphic3dTypeOfPrimitiveArray,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    closed: bool,
) -> Result<([f32; 4], Option<LineStyle>), RenderError> {
    if kind == Graphic3dTypeOfPrimitiveArray::Points {
        let style = aspects
            .get()
            .ok_or_else(|| rejected("null point aspects", "explicit Graphic3d aspects"))?;
        validate_line_shading(style, false)?;
        return Ok((rgba_color(style), None));
    }
    if !matches!(
        kind,
        Graphic3dTypeOfPrimitiveArray::Segments | Graphic3dTypeOfPrimitiveArray::Polylines
    ) {
        return Ok((crate::style::surface_color(aspects, closed)?, None));
    }
    let style = aspects
        .get()
        .ok_or_else(|| rejected("null line aspects", "explicit Graphic3d aspects"))?;
    validate_line_shading(style, true)?;
    line_parameters(style)
}

pub(crate) fn primitive_style_in_pipeline(
    kind: Graphic3dTypeOfPrimitiveArray,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    closed: bool,
    pipeline: crate::RenderLightingPipeline,
    unlit: crate::RenderUnlitFaceColorPolicy,
) -> Result<([f32; 4], Option<LineStyle>), RenderError> {
    let nodal = matches!(
        kind,
        Graphic3dTypeOfPrimitiveArray::Points
            | Graphic3dTypeOfPrimitiveArray::Segments
            | Graphic3dTypeOfPrimitiveArray::Polylines
    );
    if !nodal {
        return Ok((
            crate::style::surface_color_with_policies(aspects, pipeline, unlit)?,
            None,
        ));
    }
    primitive_style(kind, aspects, closed)
}

fn line_parameters(
    style: &dyn Graphic3dAspectsSource,
) -> Result<([f32; 4], Option<LineStyle>), RenderError> {
    let rgba = rgba_color(style);
    let line = LineStyle::new(
        style.line_width(),
        style.line_pattern(),
        style.line_stipple_factor(),
    )?
    .with_alpha_blending(style.alpha_mode() == Graphic3dAlphaMode::MaskBlend);
    Ok((rgba, Some(line)))
}

fn validate_line_shading(
    style: &dyn Graphic3dAspectsSource,
    ignores_surface_sampler: bool,
) -> Result<(), RenderError> {
    if (style.to_map_texture() && !ignores_surface_sampler) || !style.shader_program().is_null() {
        return Err(rejected(
            (style.to_map_texture(), !style.shader_program().is_null()),
            "built-in line shader without sampling surface textures; unmapped point aspect",
        ));
    }
    Ok(())
}

fn rgba_color(style: &dyn Graphic3dAspectsSource) -> [f32; 4] {
    let color = style.interior_color_rgba();
    [
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ]
}
