//! Reject style states that the initial opaque pipeline cannot yet represent.
use crate::{RenderError, error::rejected};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectInteriorStyle, graphic3d::*};
pub(crate) fn unlit_color(
    style: &Handle<dyn Graphic3dAspectsSource>,
    _closed: bool,
) -> Result<[f32; 4], RenderError> {
    let Some(style) = style.get() else {
        return Err(rejected("null aspects", "explicit unlit solid aspects"));
    };
    if style.shading_model() != Graphic3dTypeOfShadingModel::Unlit {
        return Err(rejected(
            style.shading_model(),
            "explicit unlit solid aspects",
        ));
    }
    validated_surface_color(style)
}
pub(crate) fn surface_color(
    style: &Handle<dyn Graphic3dAspectsSource>,
    _closed: bool,
) -> Result<[f32; 4], RenderError> {
    let style = style
        .get()
        .ok_or_else(|| rejected("null aspects", "explicit solid aspects"))?;
    validated_surface_color(style)
}
fn validated_surface_color(style: &dyn Graphic3dAspectsSource) -> Result<[f32; 4], RenderError> {
    validate_shading(style)?;
    validate_alpha_offset(style)?;
    let color = style.interior_color_rgba();
    if color.alpha() != 1.0 || style.front_material().transparency() != 0.0 {
        return Err(rejected(
            (color.alpha(), style.front_material().transparency()),
            "opaque surface shading in this renderer slice",
        ));
    }
    Ok([
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ])
}
fn validate_shading(style: &dyn Graphic3dAspectsSource) -> Result<(), RenderError> {
    if !matches!(
        style.shading_model(),
        Graphic3dTypeOfShadingModel::Default
            | Graphic3dTypeOfShadingModel::Unlit
            | Graphic3dTypeOfShadingModel::PhongFacet
            | Graphic3dTypeOfShadingModel::Gouraud
            | Graphic3dTypeOfShadingModel::Phong
    ) || style.interior_style() != AspectInteriorStyle::Solid
    {
        return Err(rejected(
            (style.shading_model(), style.interior_style()),
            "solid Unlit, PhongFacet, Gouraud or Phong shading",
        ));
    }
    validate_style_features(style)
}
fn validate_style_features(style: &dyn Graphic3dAspectsSource) -> Result<(), RenderError> {
    let features = (
        style.to_map_texture(),
        !style.shader_program().is_null(),
        style.to_draw_edges(),
        style.to_draw_silhouette(),
        style.distinguish(),
        !style.to_use_vertex_color_for_back_faces(),
    );
    let unlit = style.shading_model() == Graphic3dTypeOfShadingModel::Unlit;
    if features.0 || features.1 || features.2 || features.3 || (unlit && (features.4 || features.5))
    {
        return Err(rejected(
            features,
            "textures, shaders, edges, silhouettes, material distinction and back-color suppression disabled",
        ));
    }
    if style.distinguish() && style.back_material().transparency() != 0.0 {
        return Err(rejected(
            style.back_material().transparency(),
            "opaque back-face material",
        ));
    }
    Ok(())
}
fn validate_alpha_offset(style: &dyn Graphic3dAspectsSource) -> Result<(), RenderError> {
    if !matches!(
        style.alpha_mode(),
        Graphic3dAlphaMode::Opaque | Graphic3dAlphaMode::BlendAuto
    ) {
        return Err(rejected(
            style.alpha_mode(),
            "opaque or automatic opaque alpha mode",
        ));
    }
    if style.polygon_offsets().0 != 0 {
        return Err(rejected(
            style.polygon_offsets(),
            "polygon offset disabled in this renderer slice",
        ));
    }
    Ok(())
}
