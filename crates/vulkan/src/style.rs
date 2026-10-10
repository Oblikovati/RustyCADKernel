//! Reject style states that the initial opaque pipeline cannot yet represent.
use crate::{RenderError, error::rejected};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectInteriorStyle, graphic3d::*};
pub(crate) fn unlit_color(
    style: &Handle<dyn Graphic3dAspectsSource>,
    closed: bool,
) -> Result<[f32; 4], RenderError> {
    let Some(style) = style.get() else {
        return Err(rejected("null aspects", "explicit unlit solid aspects"));
    };
    validate_shading(style)?;
    validate_raster_style(style, closed)?;
    let color = style.interior_color_rgba();
    if color.alpha() != 1.0 || style.front_material().transparency() != 0.0 {
        return Err(rejected(
            (color.alpha(), style.front_material().transparency()),
            "opaque unlit shading in this renderer slice",
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
    if style.shading_model() != Graphic3dTypeOfShadingModel::Unlit
        || style.interior_style() != AspectInteriorStyle::Solid
    {
        return Err(rejected(
            (style.shading_model(), style.interior_style()),
            "unlit solid shading",
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
    if features != (false, false, false, false, false, false) {
        return Err(rejected(
            features,
            "textures, shaders, edges, silhouettes, material distinction and back-color suppression disabled",
        ));
    }
    Ok(())
}
fn validate_raster_style(
    style: &dyn Graphic3dAspectsSource,
    closed: bool,
) -> Result<(), RenderError> {
    use Graphic3dTypeOfBackfacingModel::{Auto, DoubleSided};
    if style.face_culling() != DoubleSided && !(style.face_culling() == Auto && !closed) {
        return Err(rejected(
            (style.face_culling(), closed),
            "double-sided rasterization",
        ));
    }
    validate_alpha_offset(style)
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
