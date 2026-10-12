//! Alternate real retained light, normal, material, camera, image and sampler state.
use super::fixed_workload::FixedLightingWorkload;
use rustycad_tkservice::graphic3d::*;
pub const LIVE_WORKLOADS: &[(&str, bool)] = &[
    ("lit-normal-updated", false),
    ("lit-light-updated", false),
    ("lit-light-off", false),
    ("lit-alpha-opacity-updated", false),
    ("lit-perspective", false),
    ("lit-image-updated", true),
    ("lit-sampler-updated", true),
    ("lit-mask-blend", true),
];
/// Mutate retained sources before each timed redraw; `refresh_fixed_workload(&workload,"lit-light-off",1)`.
pub fn refresh_fixed_workload(workload: &FixedLightingWorkload, mode: &str, frame: usize) {
    let alternate = frame.is_multiple_of(2);
    match workload {
        FixedLightingWorkload::Primitives(fixture) => {
            refresh_fixed_primitives(fixture, mode, alternate)
        }
        FixedLightingWorkload::Surface(fixture) => refresh_fixed_texture(fixture, mode, alternate),
        FixedLightingWorkload::Section(_) => {}
    }
}
fn refresh_fixed_primitives(
    fixture: &super::lighting_scene::LightingScene,
    mode: &str,
    alternate: bool,
) {
    match mode {
        "lit-light-updated" => {
            fixture
                .light
                .set_direction_coords(if alternate { 1. } else { 0. }, 0., -1.)
                .unwrap();
        }
        "lit-light-off" => fixture.light.set_enabled(alternate),
        "lit-alpha-opacity-updated" => refresh_fixed_opacity(fixture, alternate),
        "lit-normal-updated" => refresh_fixed_normal(fixture, alternate),
        "lit-perspective" => {
            fixture
                .base
                .view
                .camera()
                .set_scale(if alternate { 20. } else { 18. });
        }
        _ => panic!("received live fixed workload={mode}; expected a declared primitive update"),
    }
}
fn refresh_fixed_normal(fixture: &super::lighting_scene::LightingScene, alternate: bool) {
    let normal = if alternate {
        [1., 0., 0.]
    } else {
        [0.6, 0., 0.8]
    };
    fixture
        .vertices
        .set_vertex_normal(2, normal[0], normal[1], normal[2])
        .unwrap();
    fixture.vertices.attributes().invalidate().unwrap();
}
fn refresh_fixed_opacity(fixture: &super::lighting_scene::LightingScene, alternate: bool) {
    let mut material = fixture.aspect.front_material();
    material
        .set_transparency(if alternate { 0.25 } else { 0.75 })
        .unwrap();
    fixture.aspect.set_front_material(material);
    fixture.base.group.synchronize_aspects();
}
fn refresh_fixed_texture(
    fixture: &super::texture_scene::SurfaceTextureScene,
    mode: &str,
    alternate: bool,
) {
    if mode == "lit-image-updated" {
        fixture.mutate("image-updated");
        return;
    }
    configure_fixed_texture_update(fixture, mode, alternate);
    fixture.scene.group.synchronize_aspects();
}
fn configure_fixed_texture_update(
    fixture: &super::texture_scene::SurfaceTextureScene,
    mode: &str,
    alternate: bool,
) {
    match mode {
        "lit-mask-blend" => fixture
            .aspects
            .set_alpha_mode(fixed_live_alpha_mode(alternate), 0.5),
        "lit-sampler-updated" => fixture
            .texture
            .get_params()
            .set_filter(fixed_live_texture_filter(alternate)),
        _ => panic!("received live fixed workload={mode}; expected a declared texture update"),
    }
}
fn fixed_live_alpha_mode(alternate: bool) -> Graphic3dAlphaMode {
    if alternate {
        return Graphic3dAlphaMode::MaskBlend;
    }
    Graphic3dAlphaMode::Blend
}
fn fixed_live_texture_filter(alternate: bool) -> Graphic3dTypeOfTextureFilter {
    if alternate {
        return Graphic3dTypeOfTextureFilter::Nearest;
    }
    Graphic3dTypeOfTextureFilter::Bilinear
}

#[cfg(test)]
mod tests {
    use super::*;
    fn retained_workload(mode: &str, texture: bool) -> FixedLightingWorkload {
        let driver = martensite_vulkan::Graphic3dRenderDriver::with_backend(
            super::super::recording_device::RecordingDevice::default(),
            128,
        )
        .unwrap();
        FixedLightingWorkload::new(&driver, mode, texture)
    }
    #[test]
    fn primitive_updates_change_genuine_sources_on_each_frame() {
        for mode in [
            "lit-normal-updated",
            "lit-light-updated",
            "lit-light-off",
            "lit-alpha-opacity-updated",
            "lit-perspective",
        ] {
            let workload = retained_workload(mode, false);
            refresh_fixed_workload(&workload, mode, 0);
            let before = primitive_update_signature(&workload);
            refresh_fixed_workload(&workload, mode, 1);
            assert_ne!(before, primitive_update_signature(&workload), "{mode}");
        }
    }
    fn primitive_update_signature(
        workload: &FixedLightingWorkload,
    ) -> ([f64; 3], [f64; 3], bool, f32, f64) {
        let FixedLightingWorkload::Primitives(fixture) = workload else {
            panic!("expected primitive fixture");
        };
        (
            fixture.vertices.vertex_normal_xyz(2).unwrap(),
            fixture.light.direction_coords(),
            fixture.light.is_enabled(),
            fixture.aspect.front_material().alpha(),
            fixture.base.view.camera().scale(),
        )
    }
    #[test]
    fn texture_updates_change_real_revision_filter_and_alpha_selection() {
        for mode in ["lit-image-updated", "lit-sampler-updated", "lit-mask-blend"] {
            let workload = retained_workload(mode, true);
            refresh_fixed_workload(&workload, mode, 0);
            let before = texture_update_signature(&workload);
            refresh_fixed_workload(&workload, mode, 1);
            assert_ne!(before, texture_update_signature(&workload), "{mode}");
        }
    }
    fn texture_update_signature(
        workload: &FixedLightingWorkload,
    ) -> (usize, Graphic3dTypeOfTextureFilter, Graphic3dAlphaMode) {
        let FixedLightingWorkload::Surface(fixture) = workload else {
            panic!("expected texture fixture");
        };
        (
            fixture.texture.revision(),
            fixture.texture.get_params().filter(),
            fixture.aspects.alpha_mode(),
        )
    }
}
