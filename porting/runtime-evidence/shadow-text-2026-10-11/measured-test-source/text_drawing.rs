//! Shadow shares the glyph atlas and precedes the foreground, as OpenGl_Text at 3d097a0.
use crate::text_transform::TextTransform;
use crate::{GlyphLayout, PrimitiveDraw, RenderError, RenderVertex, TexturedStyle};
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub(crate) fn lower_text_draws(
    text: &Graphic3dText,
    aspect: &dyn Graphic3dAspectsSource,
    layout: &GlyphLayout,
    transform: &TextTransform,
) -> Result<Vec<PrimitiveDraw>, RenderError> {
    let mut draws = Vec::new();
    if aspect.text_display_type() == AspectTypeOfDisplayText::Shadow {
        let clip = transform.clip_offset(text, aspect, layout.point_size, [1., -1.])?;
        let mut shadow = glyph_draw(layout, aspect, clip, aspect.edge_color_rgba())?;
        shadow.text_shadow_bias = !shadow.annotation;
        draws.push(shadow);
    }
    let clip = transform.clip(text, aspect, layout.point_size)?;
    draws.push(glyph_draw(
        layout,
        aspect,
        clip,
        aspect.interior_color_rgba(),
    )?);
    Ok(draws)
}
fn glyph_draw(
    layout: &GlyphLayout,
    aspect: &dyn Graphic3dAspectsSource,
    clip: [f32; 16],
    color: ColorRgba,
) -> Result<PrimitiveDraw, RenderError> {
    let vertices = glyph_vertices(layout, color);
    // Graphic3d_ShaderManager::getStdProgramFont at 3d097a0 hard-codes this cutoff.
    let style = TexturedStyle::new(layout.atlas.clone(), layout.uv.clone(), 0.285)?
        .with_alpha_blending(aspect.alpha_mode() == Graphic3dAlphaMode::MaskBlend);
    let mut draw = PrimitiveDraw::textured(vertices, layout.indices.clone(), clip, style)?;
    let (mode, factor, units) = aspect.polygon_offsets();
    draw = draw.with_polygon_offset(crate::RenderPolygonOffset::new(mode, factor, units)?);
    draw.annotation = aspect.text_style() == AspectTypeOfStyleText::Annotation;
    Ok(draw)
}

fn glyph_vertices(layout: &GlyphLayout, color: ColorRgba) -> Vec<RenderVertex> {
    let rgb = color.rgb();
    let rgba = [
        rgb.red() as f32,
        rgb.green() as f32,
        rgb.blue() as f32,
        color.alpha(),
    ];
    layout
        .positions
        .iter()
        .map(|p| RenderVertex::new([p[0], p[1], 0.], rgba))
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::ncollection::Mat4;
    #[test]
    fn annotation_shadow_has_no_temporary_depth_bias_and_retains_subtitle_alpha() {
        let aspect = Graphic3dAspectText3d::default();
        aspect.set_text_display_type(AspectTypeOfDisplayText::Shadow);
        aspect.set_text_style(AspectTypeOfStyleText::Annotation);
        aspect.set_edge_color_rgba(ColorRgba::new(1., 0., 0., 0.5).unwrap());
        let layout = minimal_glyph_layout();
        let transform = identity_text_transform();
        let draws =
            lower_text_draws(&Graphic3dText::new(16.), &aspect, &layout, &transform).unwrap();
        assert!(
            draws
                .iter()
                .all(|d| d.annotation && !d.text_shadow_bias && !d.surface_blend)
        );
        assert_eq!(draws[0].vertices()[0].color, [1., 0., 0., 0.5]);
        aspect.set_text_style(AspectTypeOfStyleText::Normal);
        let draws =
            lower_text_draws(&Graphic3dText::new(16.), &aspect, &layout, &transform).unwrap();
        assert!(draws[0].text_shadow_bias);
        assert!(!draws[1].text_shadow_bias);
    }
    fn minimal_glyph_layout() -> GlyphLayout {
        GlyphLayout {
            positions: vec![[0.; 2]; 3],
            uv: vec![[0.; 2]; 3],
            indices: vec![0, 1, 2],
            atlas: crate::RasterImage::new([1, 1], vec![[1.; 4]]).unwrap(),
            point_size: 16,
        }
    }
    fn identity_text_transform() -> TextTransform {
        TextTransform {
            projection: Mat4::identity(),
            model_view: Mat4::identity(),
            camera_rotation: Mat4::identity(),
            viewport: [0, 0, 64, 64],
        }
    }
}
