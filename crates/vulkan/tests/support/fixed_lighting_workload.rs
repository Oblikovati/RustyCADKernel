//! Own real retained fixtures during redraw, mutation and readback workloads.
use super::{
    capping_texture_scene::CappingTextureScene, fixed_cap_lights, fixed_scene, fixed_texture_scene,
    lighting_scene::LightingScene, scene::ViewerScene, texture_scene::SurfaceTextureScene,
};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;

pub enum FixedLightingWorkload {
    Primitives(LightingScene),
    Surface(SurfaceTextureScene),
    Section(CappingTextureScene),
}
impl FixedLightingWorkload {
    /// Retain genuine objects for each fixture; `FixedLightingWorkload::new(&driver,"lit-basic",false)`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str, texture: bool) -> Self {
        if mode.starts_with("cap-") {
            let fixture = CappingTextureScene::new(driver, mode);
            fixed_cap_lights::fixed_cap_light_source(&fixture.scene, mode);
            return Self::Section(fixture);
        }
        if texture {
            return Self::Surface(fixed_texture_scene::fixed_texture_scene(driver, mode));
        }
        Self::Primitives(fixed_scene::fixed_lighting_scene(driver, mode))
    }
    /// Access the actual displayed view; `workload.scene().render_view()`.
    pub fn scene(&self) -> &ViewerScene {
        match self {
            Self::Primitives(fixture) => &fixture.base,
            Self::Surface(fixture) => &fixture.scene,
            Self::Section(fixture) => &fixture.scene,
        }
    }
    /// Apply live source changes after initial resource creation; `workload.mutate("lit-light-updated")`.
    pub fn mutate(&self, mode: &str) {
        match self {
            Self::Primitives(fixture) => fixture.mutate(mode),
            Self::Surface(fixture) => fixture.mutate(mode.strip_prefix("lit-").unwrap_or(mode)),
            Self::Section(fixture) => fixture.mutate(mode),
        }
    }
}
