//! Resolve native material and normal policy at live Graphic3d extraction, not presentation creation.
use crate::{
    PrimitiveDraw, RenderError, RenderLayer, RenderLighting, RenderMaterial,
    RenderMaterialColorSpace, RenderShading, error::rejected,
};
use crate::{primitive::PrimitiveSubmission, text_submission::TextSceneContext};
use rustycad_tkernel::{ncollection::Mat4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
pub(crate) fn apply_surface_lighting(
    draws: &mut [PrimitiveDraw],
    primitive: &PrimitiveSubmission,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    context: &TextSceneContext<'_>,
) -> Result<(), RenderError> {
    let pipeline =
        crate::lighting_pipeline::structure_lighting_pipeline(&context.group.structure());
    let model =
        primitive_shading_model(draws, &**aspects, context.rendering.shading_model, pipeline);
    if model == Graphic3dTypeOfShadingModel::Unlit {
        if pipeline == crate::RenderLightingPipeline::FixedFunction {
            return Ok(());
        }
        let policy =
            crate::lighting_pipeline::structure_unlit_face_policy(&context.group.structure());
        return crate::unlit_face_color::apply_unlit_face_policy(draws, &**aspects, policy);
    }
    apply_resolved_lighting(draws, primitive, aspects, context, model)
}
fn primitive_shading_model(
    draws: &[PrimitiveDraw],
    aspect: &dyn Graphic3dAspectsSource,
    view: Graphic3dTypeOfShadingModel,
    pipeline: crate::RenderLightingPipeline,
) -> Graphic3dTypeOfShadingModel {
    let model = fixed_compatible_shading(resolved_model(aspect, view), pipeline);
    if draws.iter().any(texture_replaces_lighting) {
        return Graphic3dTypeOfShadingModel::Unlit;
    }
    if model == Graphic3dTypeOfShadingModel::PhongFacet
        && draws
            .iter()
            .all(|draw| draw.topology() != crate::PrimitiveTopology::Triangles)
    {
        return Graphic3dTypeOfShadingModel::Unlit;
    }
    model
}
fn fixed_compatible_shading(
    model: Graphic3dTypeOfShadingModel,
    pipeline: crate::RenderLightingPipeline,
) -> Graphic3dTypeOfShadingModel {
    if pipeline != crate::RenderLightingPipeline::FixedFunction {
        return model;
    }
    // FFP binds common materials; accepting a PBR request here does not implement a PBR shader.
    match model {
        Graphic3dTypeOfShadingModel::Pbr => Graphic3dTypeOfShadingModel::Phong,
        Graphic3dTypeOfShadingModel::PbrFacet => Graphic3dTypeOfShadingModel::PhongFacet,
        _ => model,
    }
}
pub(crate) fn validate_unlit_draws(
    draws: &[PrimitiveDraw],
    aspect: &dyn Graphic3dAspectsSource,
) -> Result<(), RenderError> {
    if draws
        .iter()
        .all(|draw| draw.topology() == crate::PrimitiveTopology::Lines)
        || draws.iter().any(|draw| {
            draw.surface_texture.is_some() || draw.surface_texture_unavailable.is_some()
        })
    {
        return Ok(());
    }
    validate_unlit_surface(aspect)
}
fn apply_resolved_lighting(
    draws: &mut [PrimitiveDraw],
    primitive: &PrimitiveSubmission,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    context: &TextSceneContext<'_>,
    model: Graphic3dTypeOfShadingModel,
) -> Result<(), RenderError> {
    let normals = primitive.normals()?;
    if draws
        .iter()
        .any(|draw| draw.topology() != crate::PrimitiveTopology::Triangles)
    {
        return lower_nodal_lighting(draws, primitive, aspects, context, model, normals);
    }
    let lighting = surface_lighting(primitive, &**aspects, context, model, normals.as_deref())?;
    for draw in draws {
        let mut snapshot = lighting.clone();
        snapshot.normals = normals
            .clone()
            .unwrap_or_else(|| vec![[0., 0., 1.]; draw.vertices.len()]);
        draw.lighting = Some(snapshot);
    }
    Ok(())
}
fn lower_nodal_lighting(
    draws: &mut [PrimitiveDraw],
    primitive: &PrimitiveSubmission,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    context: &TextSceneContext<'_>,
    model: Graphic3dTypeOfShadingModel,
    normals: Option<Vec<[f32; 3]>>,
) -> Result<(), RenderError> {
    if draws
        .iter()
        .all(|draw| draw.point_style.is_some() || draw.line_style.is_some())
    {
        return apply_nodal_lighting(draws, primitive, aspects, context, model, normals);
    }
    validate_non_surface_lighting(model, normals.is_some())
}

pub(crate) fn texture_replaces_lighting(draw: &PrimitiveDraw) -> bool {
    !draw.surface_texture_binding_only
        && (draw
            .surface_texture
            .as_ref()
            .is_some_and(|texture| !texture.modulate)
            || draw
                .surface_texture_unavailable
                .as_ref()
                .is_some_and(|texture| !texture.modulate))
}

fn apply_nodal_lighting(
    draws: &mut [PrimitiveDraw],
    primitive: &PrimitiveSubmission,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    context: &TextSceneContext<'_>,
    model: Graphic3dTypeOfShadingModel,
    normals: Option<Vec<[f32; 3]>>,
) -> Result<(), RenderError> {
    let Some(normals) = normals else {
        return Ok(());
    };
    if !matches!(
        model,
        Graphic3dTypeOfShadingModel::Gouraud | Graphic3dTypeOfShadingModel::Phong
    ) {
        return validate_non_surface_lighting(model, true);
    }
    let mut lighting = surface_lighting(primitive, &**aspects, context, model, Some(&normals))?;
    if draws
        .iter()
        .any(|draw| draw.topology() == crate::PrimitiveTopology::Points)
        && crate::lighting_pipeline::structure_lighting_pipeline(&context.group.structure())
            != crate::RenderLightingPipeline::FixedFunction
    {
        lighting.validate_opaque_marker_lighting()?;
    }
    lighting.normals = normals;
    for draw in draws {
        draw.lighting = Some(lighting.clone());
    }
    Ok(())
}

pub(crate) fn validate_unlit_surface(
    aspects: &dyn Graphic3dAspectsSource,
) -> Result<(), RenderError> {
    if aspects.distinguish() || !aspects.to_use_vertex_color_for_back_faces() {
        return Err(rejected(
            (
                aspects.distinguish(),
                aspects.to_use_vertex_color_for_back_faces(),
            ),
            "unlit distinction/back-color policy is not implemented yet",
        ));
    }
    Ok(())
}
fn surface_lighting(
    primitive: &PrimitiveSubmission,
    aspects: &dyn Graphic3dAspectsSource,
    context: &TextSceneContext<'_>,
    model: Graphic3dTypeOfShadingModel,
    normals: Option<&[[f32; 3]]>,
) -> Result<RenderLighting, RenderError> {
    let shading = surface_algorithm(model, normals.is_some())?;
    let (world, view) = crate::persistent_transform::lighting_matrices(context)?;
    let mut snapshot = RenderLighting::new(
        Vec::new(),
        aspect_material(aspects),
        world,
        view,
        !context.camera.is_orthographic(),
        shading,
    )?
    .with_vertex_colors(
        primitive_has_lighting_colors(primitive),
        !aspects.to_use_vertex_color_for_back_faces(),
    );
    snapshot.normalize_fixed = crate::fixed_normalization::native_normalize_state(
        context.camera,
        context.structure,
        view,
        !context.group.transform_persistence().is_null(),
    );
    crate::fixed_lighting::snapshot_fixed_material_opacity(&mut snapshot, aspects);
    Ok(snapshot)
}

fn primitive_has_lighting_colors(primitive: &PrimitiveSubmission) -> bool {
    primitive
        .attributes
        .attribute_data(Graphic3dTypeOfAttribute::Color)
        .is_some()
}

pub(crate) fn resolved_model(
    aspect: &dyn Graphic3dAspectsSource,
    view: Graphic3dTypeOfShadingModel,
) -> Graphic3dTypeOfShadingModel {
    if aspect.shading_model() == Graphic3dTypeOfShadingModel::Default {
        return view;
    }
    aspect.shading_model()
}
pub(crate) fn surface_algorithm(
    model: Graphic3dTypeOfShadingModel,
    normals: bool,
) -> Result<RenderShading, RenderError> {
    use Graphic3dTypeOfShadingModel as Model;
    match model {
        Model::PhongFacet => Ok(RenderShading::PhongFacet),
        Model::Gouraud | Model::Phong if !normals => Ok(RenderShading::PhongFacet),
        Model::Gouraud => Ok(RenderShading::Gouraud),
        Model::Phong => Ok(RenderShading::Phong),
        _ => Err(rejected(
            model,
            "PhongFacet, Gouraud or Phong surface lighting",
        )),
    }
}
fn validate_non_surface_lighting(
    model: Graphic3dTypeOfShadingModel,
    normals: bool,
) -> Result<(), RenderError> {
    if normals
        && matches!(
            model,
            Graphic3dTypeOfShadingModel::Gouraud
                | Graphic3dTypeOfShadingModel::Phong
                | Graphic3dTypeOfShadingModel::Pbr
        )
    {
        return Err(rejected(
            (model, normals),
            "unlit non-surface geometry or Gouraud/Phong nodal lighting; PBR remains unsupported",
        ));
    }
    Ok(())
}
pub(crate) fn aspect_material(aspect: &dyn Graphic3dAspectsSource) -> RenderMaterial {
    let (front, back) = surface_material_pair(aspect);
    let mut material = RenderMaterial::default();
    // The target is an sRGB attachment; coefficients stay linear, matching active framebuffer sRGB.
    material.init(
        RenderMaterialColorSpace::Linear,
        &front,
        *aspect.interior_color_rgba().rgb(),
        if aspect.distinguish() { &back } else { &front },
        *aspect.back_interior_color_rgba().rgb(),
    );
    material
}

fn surface_material_pair(
    aspect: &dyn Graphic3dAspectsSource,
) -> (Graphic3dMaterialAspect, Graphic3dMaterialAspect) {
    let mut front = aspect.front_material();
    let mut back = aspect.back_material();
    if !crate::style::surface_uses_blending(aspect) {
        front.set_transparency(0.).expect("valid opacity");
        back.set_transparency(0.).expect("valid opacity");
    }
    (front, back)
}
pub(crate) fn apply_view_lights(
    layers: &mut [RenderLayer],
    view: &dyn Graphic3dCView,
) -> Result<(), RenderError> {
    if crate::fixed_view_lighting::uses_fixed_view_lighting(view) {
        return crate::fixed_view_lighting::apply_fixed_view_lighting(layers, view);
    }
    if !layers
        .iter()
        .flat_map(|layer| &layer.draws)
        .any(|draw| draw.lighting.is_some())
    {
        return Ok(());
    }
    if view.rendering_params().shading_model != Graphic3dTypeOfShadingModel::Unlit {
        validate_shadow_requests(view)?;
    }
    let inverse = Mat4::<f32>::from_other(
        &crate::view_render_camera::owned_render_camera(view).orientation_matrix(),
    )
    .inverted()
    .map_err(|e| rejected(e, "an invertible camera world-view matrix"))?;
    let lights = crate::RenderLightSet::for_view(
        view.rendering_params().shading_model,
        &view.lights(),
        inverse.data(),
    )?;
    for lighting in layers
        .iter_mut()
        .flat_map(|layer| &mut layer.draws)
        .filter_map(|draw| draw.lighting.as_mut())
    {
        lighting.lights = lights.clone();
    }
    Ok(())
}
fn validate_shadow_requests(view: &dyn Graphic3dCView) -> Result<(), RenderError> {
    let params = view.rendering_params();
    let lights = view.lights();
    if lights.is_null()
        || !params.ray_tracing.is_shadow_enabled
        || params.shadow_map_resolution <= 0
    {
        return Ok(());
    }
    lights.update_revision();
    if lights.nb_cast_shadows() > 0 {
        return Err(rejected(
            lights.nb_cast_shadows(),
            "zero enabled shadow casters until shadow maps are implemented",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_surface_fallbacks_and_unsupported_models_are_explicit() {
        use Graphic3dTypeOfShadingModel as M;
        assert_eq!(
            surface_algorithm(M::Gouraud, false).unwrap(),
            RenderShading::PhongFacet
        );
        assert_eq!(
            surface_algorithm(M::Phong, false).unwrap(),
            RenderShading::PhongFacet
        );
        assert_eq!(
            surface_algorithm(M::Gouraud, true).unwrap(),
            RenderShading::Gouraud
        );
        assert_eq!(
            surface_algorithm(M::Phong, true).unwrap(),
            RenderShading::Phong
        );
        assert!(surface_algorithm(M::Pbr, true).is_err());
        assert!(validate_non_surface_lighting(M::Phong, true).is_err());
        assert!(validate_non_surface_lighting(M::Phong, false).is_ok());
    }
}

#[cfg(test)]
mod opacity_tests {
    use super::*;
    #[test]
    fn opaque_lighting_overrides_transparency_without_mutating_aspects() {
        let aspect = Graphic3dAspects::default();
        let mut source = aspect.front_material();
        source.set_transparency(0.8).unwrap();
        aspect.set_front_material(source);
        assert!((aspect_material(&aspect).common[0].diffuse[3] - 0.2).abs() < 1e-6);
        aspect.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
        assert_eq!(aspect_material(&aspect).common[0].diffuse[3], 1.);
        assert_eq!(aspect.front_material().transparency(), 0.8);
    }
}
