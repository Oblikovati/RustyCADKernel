//! Validate solid surface styles before opaque or unordered blend submission.
use crate::{RenderError, error::rejected};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectInteriorStyle, graphic3d::*};
pub(crate) fn surface_color(
    style: &Handle<dyn Graphic3dAspectsSource>,
    _closed: bool,
) -> Result<[f32; 4], RenderError> {
    surface_color_in_pipeline(style, crate::RenderLightingPipeline::Programmable)
}
pub(crate) fn surface_color_in_pipeline(
    style: &Handle<dyn Graphic3dAspectsSource>,
    pipeline: crate::RenderLightingPipeline,
) -> Result<[f32; 4], RenderError> {
    surface_color_with_policies(style, pipeline, Default::default())
}
pub(crate) fn surface_color_with_policies(
    style: &Handle<dyn Graphic3dAspectsSource>,
    pipeline: crate::RenderLightingPipeline,
    unlit: crate::RenderUnlitFaceColorPolicy,
) -> Result<[f32; 4], RenderError> {
    let style = style
        .get()
        .ok_or_else(|| rejected("null aspects", "explicit solid aspects"))?;
    validated_surface_color_in_pipeline(style, pipeline, unlit)
}
#[cfg(test)]
fn validated_surface_color(style: &dyn Graphic3dAspectsSource) -> Result<[f32; 4], RenderError> {
    validated_surface_color_in_pipeline(
        style,
        crate::RenderLightingPipeline::Programmable,
        Default::default(),
    )
}
fn validated_surface_color_in_pipeline(
    style: &dyn Graphic3dAspectsSource,
    pipeline: crate::RenderLightingPipeline,
    unlit_policy: crate::RenderUnlitFaceColorPolicy,
) -> Result<[f32; 4], RenderError> {
    validate_shading(style, pipeline, unlit_policy)?;
    validate_alpha_offset(style)?;
    let color = style.interior_color_rgba();
    Ok([
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ])
}
fn validate_shading(
    style: &dyn Graphic3dAspectsSource,
    pipeline: crate::RenderLightingPipeline,
    unlit_policy: crate::RenderUnlitFaceColorPolicy,
) -> Result<(), RenderError> {
    let pbr_fallback = pipeline == crate::RenderLightingPipeline::FixedFunction
        && matches!(
            style.shading_model(),
            Graphic3dTypeOfShadingModel::Pbr | Graphic3dTypeOfShadingModel::PbrFacet
        );
    if (!pbr_fallback
        && !matches!(
            style.shading_model(),
            Graphic3dTypeOfShadingModel::Default
                | Graphic3dTypeOfShadingModel::Unlit
                | Graphic3dTypeOfShadingModel::PhongFacet
                | Graphic3dTypeOfShadingModel::Gouraud
                | Graphic3dTypeOfShadingModel::Phong
        ))
        || style.interior_style() != AspectInteriorStyle::Solid
    {
        return Err(rejected(
            (style.shading_model(), style.interior_style()),
            "solid Unlit, PhongFacet, Gouraud or Phong shading",
        ));
    }
    validate_style_features(style, pipeline, unlit_policy)
}
fn validate_style_features(
    style: &dyn Graphic3dAspectsSource,
    pipeline: crate::RenderLightingPipeline,
    unlit_policy: crate::RenderUnlitFaceColorPolicy,
) -> Result<(), RenderError> {
    let features = (
        style.to_map_texture(),
        !style.shader_program().is_null(),
        style.to_draw_edges(),
        style.to_draw_silhouette(),
        style.distinguish(),
        !style.to_use_vertex_color_for_back_faces(),
    );
    let unlit = style.shading_model() == Graphic3dTypeOfShadingModel::Unlit;
    let programmable = pipeline == crate::RenderLightingPipeline::Programmable
        && unlit_policy == crate::RenderUnlitFaceColorPolicy::LegacyV1;
    if features.1
        || features.3
        || (programmable && unlit && !features.0 && (features.4 || features.5))
    {
        return Err(rejected(
            features,
            "built-in solid surface shading, silhouettes disabled, valid unlit material/back-color policy",
        ));
    }
    Ok(())
}
fn validate_alpha_offset(style: &dyn Graphic3dAspectsSource) -> Result<(), RenderError> {
    if !matches!(
        style.alpha_mode(),
        Graphic3dAlphaMode::Opaque
            | Graphic3dAlphaMode::BlendAuto
            | Graphic3dAlphaMode::Blend
            | Graphic3dAlphaMode::Mask
            | Graphic3dAlphaMode::MaskBlend
    ) {
        return Err(rejected(
            style.alpha_mode(),
            "opaque, automatic or explicit unordered blend alpha mode",
        ));
    }
    let (mode, factor, units) = style.polygon_offsets();
    crate::RenderPolygonOffset::new(mode, factor, units)?;
    Ok(())
}

pub(crate) fn surface_uses_blending(style: &dyn Graphic3dAspectsSource) -> bool {
    style.alpha_mode() == Graphic3dAlphaMode::Blend
        || (style.alpha_mode() == Graphic3dAlphaMode::BlendAuto
            && (style.front_material().transparency() != 0.
                || (style.distinguish() && style.back_material().transparency() != 0.)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unlit_color_alpha_is_independent_of_material_and_blend_policy() {
        let style = Graphic3dAspects::default();
        style.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        style.set_polygon_offsets(0, 0., 0.);
        let mut material = style.front_material();
        material.set_transparency(0.8).unwrap();
        style.set_front_material(material);
        assert!(surface_uses_blending(&style));
        assert_eq!(validated_surface_color(&style).unwrap()[3], 1.);
        style.set_interior_color_rgba(rustycad_tkernel::quantity::ColorRgba::from_rgb(
            style.interior_color(),
            0.2,
        ));
        style.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
        assert!(!surface_uses_blending(&style));
        assert_eq!(validated_surface_color(&style).unwrap()[3], 0.2);
        style.set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
        assert!(surface_uses_blending(&style));
    }
}
