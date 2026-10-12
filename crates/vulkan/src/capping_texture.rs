//! Section textures use the native infinite-plane UVs and the ordinary retained resource cache.
use crate::{
    PrimitiveDraw, RenderError,
    retained_capping::CappingScene,
    retained_texture_images::RetainedTextureImages,
    surface_texture::{SurfaceSamplerPolicy, SurfaceTextureStyle, SurfaceTextureUnavailable},
};
use rustycad_tkernel::{ncollection::Mat4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
pub(crate) fn apply_cap_texture(
    draw: &mut PrimitiveDraw,
    aspect: &Handle<dyn Graphic3dAspectsSource>,
    cache: &RetainedTextureImages,
    scene: &CappingScene<'_>,
    orientation: Mat4<f32>,
    plane: &Handle<Graphic3dClipPlane>,
) -> Result<(), RenderError> {
    let Some(image) = prepare_cap_image(draw, &**aspect, cache, plane)? else {
        return Ok(());
    };
    let parameters = image.parameters.clone();
    let mut style = cap_image_style(draw, image, &**aspect, cache)?;
    if cache.coordinate_pipeline == crate::RenderTextureCoordinatePipeline::GeneratedCompatibility {
        style.fixed_environment = true;
        style.pending_texgen = Some(cap_generation_snapshot(
            draw,
            &style,
            &parameters,
            scene,
            orientation,
        )?);
    }
    style.validate(draw.vertices.len())?;
    draw.surface_texture = Some(style);
    Ok(())
}
fn prepare_cap_image(
    draw: &mut PrimitiveDraw,
    aspect: &dyn Graphic3dAspectsSource,
    cache: &RetainedTextureImages,
    plane: &Handle<Graphic3dClipPlane>,
) -> Result<Option<crate::retained_cap_textures::CapTextureSnapshot>, RenderError> {
    if !aspect.to_map_texture() {
        return Ok(None);
    }
    let texture = aspect.texture_map();
    let Some(texture_ref) = texture.get() else {
        return Ok(None);
    };
    crate::retained_surface_texture::validate_texture_family(texture_ref, aspect)?;
    let image = cache.cap_textures.snapshot(plane, &texture, cache)?;
    if image.image.is_none() {
        draw.surface_texture_unavailable = Some(unavailable_cap_texture(
            aspect,
            &image,
            cache.coordinate_pipeline
                == crate::RenderTextureCoordinatePipeline::GeneratedCompatibility,
        ));
    }
    Ok(image.image.is_some().then_some(image))
}
fn cap_generation_snapshot(
    draw: &PrimitiveDraw,
    style: &SurfaceTextureStyle,
    parameters: &Graphic3dTextureParams,
    scene: &CappingScene<'_>,
    orientation: Mat4<f32>,
) -> Result<Box<crate::surface_texgen::SurfaceTexgen>, RenderError> {
    let (world, view) = crate::persistent_transform::cap_lighting_matrices(
        scene.camera,
        orientation,
        scene.fields,
        scene.extent,
    )?;
    Ok(crate::surface_texgen::SurfaceTexgen::snapshot(
        crate::capping_geometry::cap_texture_uv(&draw.vertices),
        parameters,
        style.image.top_down,
        world,
        view,
    ))
}

fn cap_image_style(
    draw: &PrimitiveDraw,
    snapshot: crate::retained_cap_textures::CapTextureSnapshot,
    aspect: &dyn Graphic3dAspectsSource,
    cache: &RetainedTextureImages,
) -> Result<SurfaceTextureStyle, RenderError> {
    let image = snapshot.image.expect("validated loaded cap image");
    let parameters = snapshot.parameters;
    let uv = programmable_cap_uv(draw, &parameters, image.top_down);
    let sampler: SurfaceSamplerPolicy = cache.samplers.policy(&image, &parameters)?;
    Ok(SurfaceTextureStyle {
        image,
        pending_texgen: None,
        fixed_environment: false,
        sampler,
        uv,
        back_vertex: aspect.to_use_vertex_color_for_back_faces(),
        vertex_override: false,
        modulate: parameters.is_modulate(),
        native_gen_mode: parameters.gen_mode(),
    })
}
fn programmable_cap_uv(
    draw: &PrimitiveDraw,
    parameters: &Graphic3dTextureParams,
    top_down: bool,
) -> Vec<[f32; 4]> {
    let transform = crate::surface_texture_uv::SurfaceUvTransform::read(parameters, top_down);
    crate::capping_geometry::cap_texture_uv(&draw.vertices)
        .into_iter()
        .map(|uv| transform.programmable(uv))
        .collect()
}
fn unavailable_cap_texture(
    aspect: &dyn Graphic3dAspectsSource,
    snapshot: &crate::retained_cap_textures::CapTextureSnapshot,
    fixed_environment: bool,
) -> SurfaceTextureUnavailable {
    SurfaceTextureUnavailable {
        resource_id: snapshot.resource_id.clone(),
        fixed_environment,
        modulate: snapshot.parameters.is_modulate(),
        vertex_override: false,
        back_vertex: aspect.to_use_vertex_color_for_back_faces(),
    }
}
