//! Native cap normals and materials use the section plane's replacement ModelWorld.
use crate::{PrimitiveDraw, RenderError, RenderLighting, retained_capping::CappingScene};
use rustycad_tkernel::{ncollection::Mat4, standard::Handle};
use rustycad_tkservice::graphic3d::*;

pub(crate) fn apply_cap_lighting(
    draw: &mut PrimitiveDraw,
    aspects: &Handle<dyn Graphic3dAspectsSource>,
    scene: &CappingScene<'_>,
    orientation: Mat4<f32>,
) -> Result<(), RenderError> {
    let model = cap_shading_model(draw, &**aspects, scene.rendering.shading_model);
    if model == Graphic3dTypeOfShadingModel::Unlit {
        return validate_unlit_cap(draw, &**aspects);
    }
    let lighting =
        cap_lighting_snapshot(draw.vertices.len(), &**aspects, scene, orientation, model)?;
    draw.lighting = Some(lighting);
    Ok(())
}
fn cap_shading_model(
    draw: &PrimitiveDraw,
    aspect: &dyn Graphic3dAspectsSource,
    view: Graphic3dTypeOfShadingModel,
) -> Graphic3dTypeOfShadingModel {
    if crate::retained_lighting::texture_replaces_lighting(draw) {
        return Graphic3dTypeOfShadingModel::Unlit;
    }
    crate::retained_lighting::resolved_model(aspect, view)
}
fn validate_unlit_cap(
    draw: &PrimitiveDraw,
    aspect: &dyn Graphic3dAspectsSource,
) -> Result<(), RenderError> {
    if draw.surface_texture.is_some() || draw.surface_texture_unavailable.is_some() {
        return Ok(());
    }
    crate::retained_lighting::validate_unlit_surface(aspect)
}

fn cap_lighting_snapshot(
    vertex_count: usize,
    aspects: &dyn Graphic3dAspectsSource,
    scene: &CappingScene<'_>,
    orientation: Mat4<f32>,
    model: Graphic3dTypeOfShadingModel,
) -> Result<RenderLighting, RenderError> {
    let (world, view) = crate::persistent_transform::cap_lighting_matrices(
        scene.camera,
        orientation,
        scene.fields,
        scene.extent,
    )?;
    let mut snapshot = RenderLighting::new(
        vec![[0., -1., 0.]; vertex_count],
        crate::retained_lighting::aspect_material(aspects),
        world,
        view,
        !scene.camera.is_orthographic(),
        crate::retained_lighting::surface_algorithm(model, true)?,
    )?;
    snapshot.normalize_fixed =
        crate::fixed_normalization::native_normalize_state(scene.camera, scene.fields, view, false);
    Ok(snapshot)
}
