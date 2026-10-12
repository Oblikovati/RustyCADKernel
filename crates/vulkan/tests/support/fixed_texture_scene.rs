//! Actual fixed-lighting material/texture interactions retain independent sampler revisions.
use super::{lighting_scene, texture_scene::SurfaceTextureScene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;

pub fn fixed_texture_scene(
    driver: &Handle<Graphic3dRenderDriver>,
    mode: &str,
) -> SurfaceTextureScene {
    let surface_mode = mode.strip_prefix("lit-").unwrap_or(mode);
    let fixture = SurfaceTextureScene::with_mode(driver, surface_mode);
    if mode.starts_with("lit-") {
        fixture
            .aspects
            .set_shading_model(Graphic3dTypeOfShadingModel::Phong);
        lighting_scene::configure_lights(&fixture.scene, "lit-basic");
    }
    fixture
}
