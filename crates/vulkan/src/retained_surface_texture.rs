//! Surface extraction snapshots content by revision while reading UV/sampler policy live.
use crate::{
    PrimitiveDraw, RenderError, error::rejected, primitive::PrimitiveSubmission,
    surface_texture::*, text_submission::TextSceneContext,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
pub(crate) fn driver_texture_images(
    structure: &Handle<dyn Graphic3dStructureSource>,
) -> Arc<crate::retained_texture_images::RetainedTextureImages> {
    let driver = structure
        .structure_manager()
        .graphic_driver()
        .cast::<crate::Graphic3dRenderDriver>();
    driver
        .get()
        .map_or_else(Arc::default, |driver| driver.texture_images.clone())
}
pub(crate) fn apply_surface_texture(
    draws: &mut [PrimitiveDraw],
    primitive: &PrimitiveSubmission,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    context: &TextSceneContext<'_>,
) -> Result<(), RenderError> {
    if !aspects.to_map_texture()
        || draws
            .iter()
            .all(|draw| draw.topology() != crate::PrimitiveTopology::Triangles)
    {
        return Ok(());
    }
    let handle = aspects.texture_map();
    let Some(texture) = handle.get() else {
        return Ok(());
    };
    validate_texture_family(texture, &**aspects)?;
    let Some(image) = context.group.texture_images.snapshot(texture)? else {
        return Ok(());
    };
    let parameters = texture.get_params();
    let uv = crate::surface_texture_uv::surface_texture_coordinates(
        primitive,
        &parameters,
        image.top_down,
    )?;
    for draw in draws {
        draw.surface_texture = Some(SurfaceTextureStyle {
            image: image.clone(),
            sampler: SurfaceSamplerPolicy::read(&parameters),
            uv: uv.clone(),
            back_vertex: aspects.to_use_vertex_color_for_back_faces(),
            vertex_override: primitive
                .attributes
                .attribute_data(Graphic3dTypeOfAttribute::Color)
                .is_some(),
            modulate: parameters.is_modulate(),
        });
        draw.surface_texture
            .as_ref()
            .unwrap()
            .validate(draw.vertices.len())?;
    }
    Ok(())
}
fn validate_texture_family(
    texture: &dyn Graphic3dTextureMap,
    aspects: &dyn Graphic3dAspectsSource,
) -> Result<(), RenderError> {
    let set = aspects.texture_set();
    let parameters = texture.get_params();
    if texture.texture_type() != Graphic3dTypeOfTexture::TOT_2D
        || set.size() != 1
        || parameters.texture_unit() != Graphic3dTextureUnit::BASE_COLOR
    {
        return Err(rejected(
            (
                texture.texture_type(),
                set.size(),
                parameters.texture_unit(),
            ),
            "one retained standard 2D base-color texture",
        ));
    }
    Ok(())
}
