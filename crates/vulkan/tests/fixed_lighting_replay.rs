//! Replay genuine fixed-lighting native assets without OCCT; retain precision failures separately.
#[path = "support/fixed_caps_assets_0.rs"]
mod cap_assets;
#[allow(dead_code)]
#[path = "support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "support/ffp_oracle_profile.rs"]
mod ffp_oracle_profile;
#[path = "support/fixed_cap_lights.rs"]
mod fixed_cap_lights;
#[allow(dead_code)]
#[path = "support/fixed_lighting_scene.rs"]
mod fixed_scene;
#[path = "support/fixed_texture_scene.rs"]
mod fixed_texture_scene;
#[path = "support/fixed_lighting_assets_0.rs"]
mod lighting_assets_0;
#[path = "support/fixed_lighting_assets_1.rs"]
mod lighting_assets_1;
#[path = "support/fixed_lighting_assets_2.rs"]
mod lighting_assets_2;
#[allow(dead_code)]
#[path = "support/lighting_scene.rs"]
mod lighting_scene;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/fixed_textures_assets_0.rs"]
mod texture_assets;
#[allow(dead_code)]
#[path = "support/surface_texture_scene.rs"]
mod texture_scene;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;

type NativeFixedLightingAsset = (
    &'static str,
    bool,
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [u8],
);
const HOST_PRECISION_DIAGNOSTICS: &[&str] = &[
    "lit-colored",
    "lit-rotation",
    "lit-normal-updated",
    "lit-light-updated",
    "lit-alpha-opacity-updated",
    "lit-adjacent-basic",
    "lit-adjacent-point",
    "lit-adjacent-spot",
];

#[test]
fn fixed_lighting_native_assets_replay_without_occt() {
    replay_fixed_assets(false);
}
#[test]
fn fixed_lighting_manual_coordinates_keep_encoded_texture_environment() {
    let driver = Graphic3dRenderDriver::with_render_pipelines(
        device_options::validation_options(),
        martensite_vulkan::RenderTextureCoordinatePipeline::Programmable,
        martensite_vulkan::RenderMeshEdgePipeline::Programmable,
        martensite_vulkan::RenderLightingPipeline::FixedFunction,
    )
    .unwrap();
    let asset = texture_assets::ASSETS
        .iter()
        .find(|row| row.0 == "lit-nearest")
        .unwrap();
    let (_, _, host_rgba, host_depth, software_rgba, software_depth) = *asset;
    let (rgba, depth) = if driver.device_name().contains("llvmpipe") {
        (software_rgba, software_depth)
    } else {
        (host_rgba, host_depth)
    };
    native_golden::assert_capture(
        &capture_fixed_texture(&driver, "lit-nearest"),
        rgba,
        depth,
        "lit-nearest-manual-coordinates",
    );
}
#[test]
#[ignore = "GL2.1 lighting is undefined for eye-space w=0 section vertices; retain configured-driver diagnostic"]
fn fixed_cap_point_source_preserves_homogeneous_lighting() {
    let driver = fixed_scene::fixed_lighting_driver(device_options::validation_options()).unwrap();
    let mode = "cap-oblique-fixed-phong-source-point";
    let capture = capture_fixed_cap(&driver, mode);
    let (rgba, depth) = fixed_cap_point_asset(driver.device_name().contains("llvmpipe"));
    native_golden::assert_capture(&capture, rgba, depth, mode);
}
fn fixed_cap_point_asset(software: bool) -> (&'static [u8], &'static [u8]) {
    if software {
        return (
            include_bytes!(
                "fixtures/fixed-lighting/software/cap-oblique-fixed-phong-source-point.rgba"
            ),
            include_bytes!(
                "fixtures/fixed-lighting/software/cap-oblique-fixed-phong-source-point.depth"
            ),
        );
    }
    (
        include_bytes!("fixtures/fixed-lighting/host/cap-oblique-fixed-phong-source-point.rgba"),
        include_bytes!("fixtures/fixed-lighting/host/cap-oblique-fixed-phong-source-point.depth"),
    )
}
fn capture_fixed_cap(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> FrameCapture {
    let fixture = capping_texture_scene::CappingTextureScene::new(driver, mode);
    fixed_cap_lights::fixed_cap_light_source(&fixture.scene, mode);
    fixture.scene.render_view().try_redraw().unwrap();
    fixture.mutate(mode);
    capture_fixed_view(&fixture.scene)
}
#[test]
#[ignore = "retains native AMD one-byte and rotated-depth strict diagnostics; no gate relaxation"]
fn complete_fixed_lighting_strict_native_diagnostic() {
    replay_fixed_assets(true);
}
fn replay_fixed_assets(include_diagnostics: bool) {
    let driver = fixed_scene::fixed_lighting_driver(device_options::validation_options()).unwrap();
    let software = driver.device_name().contains("llvmpipe");
    for &(mode, texture, host_rgba, host_depth, software_rgba, software_depth) in fixed_assets() {
        if !include_diagnostics && !software && HOST_PRECISION_DIAGNOSTICS.contains(&mode) {
            continue;
        }
        let capture = capture_fixed_asset(&driver, mode, texture);
        let (rgba, depth) = if software {
            (software_rgba, software_depth)
        } else {
            (host_rgba, host_depth)
        };
        let (rgba, depth) = ffp_oracle_profile::profiled_driver_asset(
            &driver,
            mode,
            (host_rgba, host_depth),
            (software_rgba, software_depth),
        )
        .unwrap_or((rgba, depth));
        native_golden::assert_capture(&capture, rgba, depth, mode);
    }
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn fixed_assets() -> impl Iterator<Item = &'static NativeFixedLightingAsset> {
    lighting_assets_0::ASSETS
        .iter()
        .chain(lighting_assets_1::ASSETS)
        .chain(lighting_assets_2::ASSETS)
        .chain(texture_assets::ASSETS)
        .chain(cap_assets::ASSETS)
}
fn capture_fixed_asset(
    driver: &Handle<Graphic3dRenderDriver>,
    mode: &str,
    texture: bool,
) -> FrameCapture {
    if mode.starts_with("cap-") {
        return capture_fixed_cap(driver, mode);
    }
    if texture {
        return capture_fixed_texture(driver, mode);
    }
    let fixture = fixed_scene::fixed_lighting_scene(driver, mode);
    fixture.base.render_view().try_redraw().unwrap();
    fixture.mutate(mode);
    capture_fixed_view(&fixture.base)
}
fn capture_fixed_texture(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> FrameCapture {
    let fixture = fixed_texture_scene::fixed_texture_scene(driver, mode);
    fixture.scene.render_view().try_redraw().unwrap();
    fixture.mutate(mode.strip_prefix("lit-").unwrap_or(mode));
    capture_fixed_view(&fixture.scene)
}
fn capture_fixed_view(scene: &scene::ViewerScene) -> FrameCapture {
    let view = scene.cview();
    view.set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    view.set_z_layer_redraw_mode(true);
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    scene.view.remove();
    capture
}
