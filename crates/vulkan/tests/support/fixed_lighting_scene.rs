//! Genuine retained scene adapter for native GL lighting fixtures.
use super::lighting_scene::LightingScene;
use martensite_vulkan::{
    Graphic3dRenderDriver, RenderError, RenderLightingPipeline, RenderMeshEdgePipeline,
    RenderTextureCoordinatePipeline, VulkanOptions,
};
#[path = "fixed_lighting_arrays.rs"]
mod fixed_arrays;
#[path = "fixed_lighting_binding.rs"]
mod fixed_binding;
#[path = "fixed_lighting_markers.rs"]
mod fixed_markers;
use rustycad_tkernel::standard::Handle;
#[path = "fixed_lighting_transform.rs"]
mod fixed_transform;
use fixed_transform::apply_fixed_fixture_transform;
use rustycad_tkservice::graphic3d::*;

pub fn fixed_lighting_driver(
    options: VulkanOptions,
) -> Result<Handle<Graphic3dRenderDriver>, RenderError> {
    Graphic3dRenderDriver::with_render_pipelines(
        options,
        RenderTextureCoordinatePipeline::GeneratedCompatibility,
        RenderMeshEdgePipeline::Programmable,
        RenderLightingPipeline::FixedFunction,
    )
}
pub fn fixed_lighting_scene(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> LightingScene {
    let canonical = fixed_fixture_mode(mode);
    let mut fixture = LightingScene::new(driver, &canonical);
    configure_fixed_fixture_style(&fixture, mode);
    replace_fixed_fixture_vertices(&mut fixture, mode);
    if mode == "lit-perspective" {
        fixture
            .base
            .view
            .camera()
            .set_projection_type(Graphic3dCameraProjection::Perspective);
    }
    apply_fixed_fixture_transform(&fixture, mode);
    fixed_binding::fixed_adjacent_groups(&fixture, mode);
    fixture
}

fn fixed_fixture_mode(mode: &str) -> String {
    if let Some(suffix) = mode.strip_prefix("lit-unlit-") {
        return format!("lit-{suffix}");
    }
    if let Some(suffix) = mode.strip_prefix("lit-adjacent-") {
        return format!("lit-{suffix}");
    }
    if let Some(suffix) = mode.strip_prefix("lit-persist-") {
        return format!("lit-{suffix}");
    }
    if mode.starts_with("lit-facet-") {
        return "lit-facet".into();
    }
    if let Some(suffix) = mode.strip_prefix("lit-strip-") {
        return format!("lit-line-{suffix}");
    }
    mode.into()
}

fn configure_fixed_fixture_style(fixture: &LightingScene, mode: &str) {
    let shading = if mode.contains("pbr") {
        if mode.contains("facet") {
            Graphic3dTypeOfShadingModel::PbrFacet
        } else {
            Graphic3dTypeOfShadingModel::Pbr
        }
    } else if mode.starts_with("lit-unlit") || mode.contains("marker-") && mode.contains("unlit") {
        Graphic3dTypeOfShadingModel::Unlit
    } else if mode.starts_with("lit-marker-") && mode.contains("facet") {
        Graphic3dTypeOfShadingModel::PhongFacet
    } else {
        return;
    };
    fixture.aspect.set_shading_model(shading);
}
fn replace_fixed_fixture_vertices(fixture: &mut LightingScene, mode: &str) {
    let vertices = if mode.starts_with("lit-marker-") {
        fixed_markers::fixed_marker_aspect(&fixture.aspect, mode);
        Some(fixed_markers::fixed_marker_array(mode))
    } else {
        fixed_arrays::fixed_array_override(mode)
    };
    let Some(vertices) = vertices else {
        return;
    };
    fixture.base.group.clear(false);
    fixture
        .base
        .group
        .set_group_primitives_aspect(&fixture.aspect.cast());
    fixture.base.group.add_primitive_array(&vertices, true);
    fixture.vertices = vertices;
}
