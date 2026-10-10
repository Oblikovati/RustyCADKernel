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
    let model = resolved_model(&**aspects, context.rendering.shading_model);
    if model == Graphic3dTypeOfShadingModel::Unlit {
        return validate_unlit_surface(&**aspects);
    }
    let normals = primitive.normals()?;
    if draws
        .iter()
        .any(|draw| draw.topology() != crate::PrimitiveTopology::Triangles)
    {
        return validate_non_surface_lighting(model, normals.is_some());
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
    let colored = primitive
        .attributes
        .attribute_data(Graphic3dTypeOfAttribute::Color)
        .is_some();
    Ok(RenderLighting::new(
        Vec::new(),
        aspect_material(aspects),
        world,
        view,
        !context.camera.is_orthographic(),
        shading,
    )?
    .with_vertex_colors(colored, !aspects.to_use_vertex_color_for_back_faces()))
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
            "lit line/point normal attributes are not implemented yet",
        ));
    }
    Ok(())
}
pub(crate) fn aspect_material(aspect: &dyn Graphic3dAspectsSource) -> RenderMaterial {
    let front = aspect.front_material();
    let back = aspect.back_material();
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
pub(crate) fn apply_view_lights(
    layers: &mut [RenderLayer],
    view: &dyn Graphic3dCView,
) -> Result<(), RenderError> {
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
    let inverse = Mat4::<f32>::from_other(&view.camera().orientation_matrix())
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
