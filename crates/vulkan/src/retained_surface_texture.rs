//! Surface extraction snapshots content by revision while reading UV/sampler policy live.
use crate::{
    PrimitiveDraw, RenderError, error::rejected, primitive::PrimitiveSubmission,
    surface_texture::*, text_submission::TextSceneContext,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
/// Share the driver's image cache across groups; `driver_texture_images(&structure)`.
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
/// Resolve actual retained surface textures; `apply_surface_texture(draws, primitive, aspects, context)?`.
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
    let Some(style) = retained_surface_style(primitive, aspects, context)? else {
        return apply_unavailable_surface_texture(draws, primitive, &**aspects);
    };
    for draw in draws {
        style.validate(draw.vertices.len())?;
        draw.surface_texture = Some(style.clone());
    }
    Ok(())
}
fn apply_unavailable_surface_texture(
    draws: &mut [PrimitiveDraw],
    primitive: &PrimitiveSubmission,
    aspects: &dyn Graphic3dAspectsSource,
) -> Result<(), RenderError> {
    let handle = aspects.texture_map();
    let Some(texture) = handle.get() else {
        return Ok(());
    };
    let vertex_override = primitive
        .attributes
        .attribute_data(Graphic3dTypeOfAttribute::Color)
        .is_some();
    let unavailable = SurfaceTextureUnavailable {
        resource_id: texture.get_id().as_bytes().to_vec(),
        modulate: texture.get_params().is_modulate(),
        vertex_override,
        back_vertex: aspects.to_use_vertex_color_for_back_faces(),
    };
    for draw in draws {
        draw.surface_texture_unavailable = Some(unavailable.clone());
    }
    Ok(())
}
fn retained_surface_style(
    primitive: &PrimitiveSubmission,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    context: &TextSceneContext<'_>,
) -> Result<Option<SurfaceTextureStyle>, RenderError> {
    let handle = aspects.texture_map();
    let Some(texture) = handle.get() else {
        return Ok(None);
    };
    validate_texture_family(texture, &**aspects)?;
    let Some(image) = context.group.texture_images.snapshot(&handle)? else {
        return Ok(None);
    };
    complete_surface_style(image, primitive, aspects, context).map(Some)
}
fn complete_surface_style(
    image: Arc<SurfaceTextureImage>,
    primitive: &PrimitiveSubmission,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    context: &TextSceneContext<'_>,
) -> Result<SurfaceTextureStyle, RenderError> {
    let parameters = aspects.texture_map().get_params();
    let uv = crate::surface_texture_uv::surface_texture_coordinates(
        primitive,
        &parameters,
        image.top_down,
    )?;
    let sampler = context
        .group
        .texture_images
        .samplers
        .policy(&image, &parameters)?;
    Ok(extracted_surface_style(
        image,
        uv,
        &parameters,
        sampler,
        primitive,
        &**aspects,
    ))
}
fn extracted_surface_style(
    image: Arc<SurfaceTextureImage>,
    uv: Vec<[f32; 4]>,
    parameters: &Graphic3dTextureParams,
    sampler: SurfaceSamplerPolicy,
    primitive: &PrimitiveSubmission,
    aspects: &dyn Graphic3dAspectsSource,
) -> SurfaceTextureStyle {
    let vertex_override = primitive
        .attributes
        .attribute_data(Graphic3dTypeOfAttribute::Color)
        .is_some();
    SurfaceTextureStyle {
        image,
        uv,
        sampler,
        back_vertex: aspects.to_use_vertex_color_for_back_faces(),
        vertex_override,
        modulate: parameters.is_modulate(),
    }
}
fn validate_texture_family(
    texture: &dyn Graphic3dTextureMap,
    aspects: &dyn Graphic3dAspectsSource,
) -> Result<(), RenderError> {
    validate_surface_generation(texture.get_params().gen_mode())?;
    let shape = (
        texture.texture_type(),
        aspects.texture_set().size(),
        texture.get_params().texture_unit(),
    );
    let expected = (
        Graphic3dTypeOfTexture::TOT_2D,
        1,
        Graphic3dTextureUnit::BASE_COLOR,
    );
    if shape == expected {
        return Ok(());
    }
    Err(rejected(
        shape,
        "one retained standard 2D base-color texture",
    ))
}
fn validate_surface_generation(generation: Graphic3dTypeOfTextureMode) -> Result<(), RenderError> {
    if generation == Graphic3dTypeOfTextureMode::Manual {
        return Ok(());
    }
    Err(rejected(
        generation,
        "manual standard2D texture coordinates; legacy texgen is not implemented yet",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::ncollection::Vec4;
    #[test]
    fn unsupported_texture_generation_is_explicit_instead_of_fabricated_uvs() {
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(Handle::default()));
        let aspects = Graphic3dAspects::default();
        aspects.set_texture_map(texture.cast());
        assert!(validate_texture_family(&*texture, &aspects).is_ok());
        texture.get_params().set_gen_mode(
            Graphic3dTypeOfTextureMode::Object,
            Vec4::new(1., 0., 0., 0.),
            Vec4::new(0., 1., 0., 0.),
        );
        assert!(validate_texture_family(&*texture, &aspects).is_err());
    }
}
