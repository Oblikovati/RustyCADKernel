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
    let model = crate::retained_lighting::resolved_model(&**aspects, scene.rendering.shading_model);
    if model == Graphic3dTypeOfShadingModel::Unlit {
        return crate::retained_lighting::validate_unlit_surface(&**aspects);
    }
    draw.lighting = Some(cap_lighting_snapshot(
        draw.vertices.len(),
        &**aspects,
        scene,
        orientation,
        model,
    )?);
    Ok(())
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
    RenderLighting::new(
        vec![[0., -1., 0.]; vertex_count],
        crate::retained_lighting::aspect_material(aspects),
        world,
        view,
        !scene.camera.is_orthographic(),
        crate::retained_lighting::surface_algorithm(model, true)?,
    )
}
