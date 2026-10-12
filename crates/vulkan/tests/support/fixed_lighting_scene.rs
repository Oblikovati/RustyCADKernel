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
use rustycad_tkmath::gp::{Axis1, Direction3};
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Datum3;
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
    if mode == "lit-unlit" {
        fixture
            .aspect
            .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    }
    let override_vertices = if mode.starts_with("lit-marker-") {
        fixed_markers::fixed_marker_aspect(&fixture.aspect, mode);
        Some(fixed_markers::fixed_marker_array(mode))
    } else {
        fixed_arrays::fixed_array_override(mode)
    };
    if let Some(vertices) = override_vertices {
        fixture.base.group.clear(false);
        fixture
            .base
            .group
            .set_group_primitives_aspect(&fixture.aspect.cast());
        fixture.base.group.add_primitive_array(&vertices, true);
        fixture.vertices = vertices;
    }
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
fn apply_fixed_fixture_transform(fixture: &LightingScene, mode: &str) {
    if mode.starts_with("lit-persist-") {
        fixture.base.structure.set_transform_persistence(
            &Handle::new(
                Graphic3dTransformPers::with_point(
                    Graphic3dTransModeFlags::ZOOM_PERS,
                    &Point3::default(),
                )
                .unwrap(),
            )
            .cast(),
        );
        return;
    }
    let mut transform = Transform3::default();
    if mode == "lit-group-scale" {
        transform.set_scale(Point3::default(), 0.75).unwrap();
        fixture.base.group.set_transformation(&transform);
        return;
    }
    if mode == "lit-scale" {
        transform.set_scale(Point3::default(), 0.75).unwrap();
    } else if mode == "lit-rotation" {
        transform
            .set_rotation(
                Axis1::new(Point3::default(), Direction3::new(0., 1., 0.).unwrap()),
                0.35,
            )
            .unwrap();
    } else {
        return;
    }
    fixture
        .base
        .structure
        .set_transformation(&Handle::new(Datum3::new(transform)));
}

fn fixed_fixture_mode(mode: &str) -> String {
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
