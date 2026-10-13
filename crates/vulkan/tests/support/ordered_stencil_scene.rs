//! Actual retained stencil elements mirror martensite_ordered_stencil_capture.cxx.
use super::{oit_scene::OitScene, scene::ViewerScene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::graphic3d::*;
pub const STENCIL_CASES: &[&str] = &[
    "enable",
    "disable",
    "toggle",
    "leading-empty",
    "trailing-empty",
    "clear-executed",
    "remove-executed",
    "cap-enable",
    "cap-disable",
    "cap-transparent",
    "cap-empty-closed",
    "layer-shared",
    "layer-clear",
    "line",
    "point",
    "text",
];
pub struct OrderedStencilScene {
    pub fixture: OitScene,
}
impl OrderedStencilScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str, method: &str) -> Self {
        assert!(
            STENCIL_CASES.contains(&mode),
            "received stencil mode={mode:?}; expected {STENCIL_CASES:?}"
        );
        let fixture = OitScene::with_scene(driver, false, base_fixture(mode));
        fixture.parameters(method, 0, 0., 4);
        apply_mode(&fixture.scene, mode);
        Self { fixture }
    }
    pub fn initial_redraw(&self, mode: &str) -> Result<(), martensite_vulkan::RenderError> {
        self.fixture.scene.render_view().try_redraw()?;
        match mode {
            "clear-executed" => self.fixture.scene.group.clear(false),
            "remove-executed" => self.fixture.scene.group.remove(),
            _ => {}
        }
        Ok(())
    }
    pub fn finish(&self) {
        self.fixture.finish();
    }
}
fn apply_mode(scene: &ViewerScene, mode: &str) {
    if mode.starts_with("cap-") {
        return apply_cap_mode(scene, mode);
    }
    if mode.starts_with("layer-") {
        scene.group.set_stencil_test_options(true);
        return;
    }
    if mode == "trailing-empty" {
        scene
            .structure
            .new_group()
            .unwrap()
            .set_stencil_test_options(true);
        return;
    }
    let group = &scene.group;
    group.clear(false);
    group.set_group_primitives_aspect(&super::oit_scene::oit_fill_aspect(0, "two").cast());
    group.set_stencil_test_options(mode != "disable");
    if mode != "leading-empty" {
        group.add_primitive_array(&super::oit_scene::oit_quad(0, "two"), true);
    }
    if mode == "toggle" {
        group.set_stencil_test_options(false);
    }
}
fn apply_cap_mode(scene: &ViewerScene, mode: &str) {
    let plane = super::capping_scene::configure_capping(scene, "cap-basic");
    scene.group.set_stencil_test_options(mode != "cap-disable");
    if mode == "cap-empty-closed" {
        scene.group.clear(false);
        scene.group.set_closed(true);
    }
    if mode != "cap-transparent" {
        return;
    }
    let aspect = plane.capping_aspect();
    set_cap_alpha(&*aspect);
}
fn set_cap_alpha(aspect: &dyn Graphic3dAspectsSource) {
    aspect.set_interior_color_rgba(ColorRgba::new(0., 1., 0., 0.5).unwrap());
    let mut material = aspect.front_material();
    material.set_transparency(0.5).unwrap();
    aspect.set_front_material(material);
    aspect.set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
}

fn base_fixture(mode: &str) -> &str {
    match mode {
        "line" => "line-mask",
        "point" => "sprite-opaque",
        "text" => "text-colored",
        "layer-clear" | "layer-shared" => mode,
        _ => "two",
    }
}
