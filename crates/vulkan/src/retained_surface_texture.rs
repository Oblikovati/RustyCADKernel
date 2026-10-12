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
            .all(|draw| draw.topology() == crate::PrimitiveTopology::Points)
    {
        return Ok(());
    }
    let Some(style) = retained_surface_style(primitive, aspects, context)? else {
        return apply_unavailable_surface_texture(draws, primitive, &**aspects);
    };
    for draw in draws {
        style.validate(draw.vertices.len())?;
        draw.surface_texture = Some(style.clone());
        draw.surface_texture_binding_only = draw.topology() == crate::PrimitiveTopology::Lines;
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
        draw.surface_texture_binding_only = draw.topology() == crate::PrimitiveTopology::Lines;
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
    let image = context
        .group
        .aspect_textures
        .snapshot(aspects, &context.group.texture_images)?;
    let Some(image) = image else {
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
    let mut style = extracted_surface_style(image, uv, &parameters, sampler, primitive, &**aspects);
    snapshot_fixed_surface_binding(&mut style, primitive, &parameters, context)?;
    Ok(style)
}
fn snapshot_fixed_surface_binding(
    style: &mut SurfaceTextureStyle,
    primitive: &PrimitiveSubmission,
    parameters: &Graphic3dTextureParams,
    context: &TextSceneContext<'_>,
) -> Result<(), RenderError> {
    if context.group.texture_images.coordinate_pipeline
        == crate::RenderTextureCoordinatePipeline::GeneratedCompatibility
    {
        let (world, view) = crate::persistent_transform::lighting_matrices(context)?;
        style.fixed_environment = true;
        style.pending_texgen = Some(crate::surface_texgen::SurfaceTexgen::snapshot(
            crate::surface_texture_uv::surface_input_uv(primitive)?,
            parameters,
            style.image.top_down,
            world,
            view,
        ));
    }
    Ok(())
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
        pending_texgen: None,
        fixed_environment: false,
        uv,
        sampler,
        back_vertex: aspects.to_use_vertex_color_for_back_faces(),
        vertex_override,
        modulate: parameters.is_modulate(),
        native_gen_mode: parameters.gen_mode(),
    }
}
pub(crate) fn validate_texture_family(
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
    if matches!(
        generation,
        Graphic3dTypeOfTextureMode::Manual
            | Graphic3dTypeOfTextureMode::Object
            | Graphic3dTypeOfTextureMode::Eye
    ) {
        return Ok(());
    }
    Err(rejected(
        generation,
        "Manual/Object/Eye standard2D coordinates; Sphere/Sprite generation remains unsupported",
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
            Graphic3dTypeOfTextureMode::Sphere,
            Vec4::new(1., 0., 0., 0.),
            Vec4::new(0., 1., 0., 0.),
        );
        assert!(validate_texture_family(&*texture, &aspects).is_err());
    }
}
