//! Pinned native texgen and cap buffers replay with Rust-only retained presentations.
#[allow(dead_code)]
#[path = "support/capping_texture_scene.rs"]
mod capping_scene;
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "support/texture_generation_scene.rs"]
mod generation_scene;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "support/surface_texture_scene.rs"]
mod texture_scene;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
macro_rules! dependency_asset {
    ($family:literal, $mode:literal) => {
        (
            $family,
            $mode,
            include_bytes!(concat!(
                "fixtures/texture-texgen-capping/replay/host/",
                $family,
                "-",
                $mode,
                ".rgba"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "fixtures/texture-texgen-capping/replay/host/",
                $family,
                "-",
                $mode,
                ".depth"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "fixtures/texture-texgen-capping/replay/lavapipe/",
                $family,
                "-",
                $mode,
                ".rgba"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "fixtures/texture-texgen-capping/replay/lavapipe/",
                $family,
                "-",
                $mode,
                ".depth"
            ))
            .as_slice(),
        )
    };
}
type DependencyNativeAsset = (
    &'static str,
    &'static str,
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [u8],
);
const DEPENDENCY_NATIVE_ASSETS: &[DependencyNativeAsset] = &[
    dependency_asset!("fixed", "object-no-uv"),
    dependency_asset!("fixed", "eye-no-uv"),
    dependency_asset!("fixed", "object-q"),
    dependency_asset!("fixed", "eye-q"),
    dependency_asset!("fixed", "eye-moved"),
    dependency_asset!("fixed", "eye-camera"),
    dependency_asset!("fixed", "eye-updated"),
    dependency_asset!("fixed", "eye-shared-no-uv"),
    dependency_asset!("fixed", "eye-shared-reset-no-uv"),
    dependency_asset!("programmable", "eye-shared-revision-no-uv"),
    dependency_asset!("programmable", "object-shared-revision-no-uv"),
    dependency_asset!("fixed", "eye-shared-revision-no-uv"),
    dependency_asset!("fixed", "object-shared-revision-no-uv"),
    dependency_asset!("programmable", "object-no-uv"),
    dependency_asset!("programmable", "eye-q"),
    dependency_asset!("programmable", "eye-shared-no-uv"),
    dependency_asset!("cap", "oblique-image-updated"),
    dependency_asset!("cap", "oblique-image-refreshed"),
    dependency_asset!("cap", "oblique-mask-exact"),
    dependency_asset!("cap", "oblique-bc1-rgb"),
    dependency_asset!("cap", "oblique-bc1-rgba"),
    dependency_asset!("cap", "oblique-bc2"),
    dependency_asset!("cap", "oblique-bc3"),
    dependency_asset!("cap", "oblique-gouraud"),
    dependency_asset!("cap", "oblique-facet"),
    dependency_asset!("cap", "oblique-phong"),
    dependency_asset!("cap", "oblique-gouraud-mask-blend"),
    dependency_asset!("cap", "oblique-facet-mask-blend"),
    dependency_asset!("cap", "oblique-phong-blend"),
    dependency_asset!("cap", "oblique-phong-no-modulate"),
    dependency_asset!("fixedcap", "oblique-image-updated"),
    dependency_asset!("fixedcap", "oblique-image-refreshed"),
    dependency_asset!("fixedcap", "oblique-mask-exact"),
    dependency_asset!("fixedcap", "oblique-bc1-rgb"),
    dependency_asset!("fixedcap", "oblique-bc1-rgba"),
    dependency_asset!("fixedcap", "oblique-bc2"),
    dependency_asset!("fixedcap", "oblique-bc3"),
    dependency_asset!("fixedcap", "oblique-phong-no-modulate"),
    dependency_asset!("cap", "oblique-mask"),
    dependency_asset!("cap", "oblique-mask-blend"),
    dependency_asset!("cap", "oblique-blend"),
    dependency_asset!("cap", "oblique-opaque-alpha"),
    dependency_asset!("fixedcap", "oblique-mask"),
    dependency_asset!("fixedcap", "oblique-opaque-alpha"),
];
#[test]
fn native_coordinate_and_cap_assets_replay_without_occt() {
    for policy in [
        RenderTextureCoordinatePipeline::Programmable,
        RenderTextureCoordinatePipeline::GeneratedCompatibility,
    ] {
        let driver = Graphic3dRenderDriver::with_texture_coordinate_pipeline(
            device_options::validation_options(),
            policy,
        )
        .unwrap();
        replay_dependency_assets(&driver, policy);
        assert!(
            driver.validation_errors().is_empty(),
            "{:?}",
            driver.validation_errors()
        );
    }
}
fn replay_dependency_assets(
    driver: &Handle<Graphic3dRenderDriver>,
    policy: RenderTextureCoordinatePipeline,
) {
    for (family, mode, host_rgba, host_depth, software_rgba, software_depth) in
        DEPENDENCY_NATIVE_ASSETS
    {
        let (rgba, depth) = if driver.device_name().contains("llvmpipe") {
            (*software_rgba, *software_depth)
        } else {
            (*host_rgba, *host_depth)
        };
        let fixed = policy == RenderTextureCoordinatePipeline::GeneratedCompatibility;
        if matches!(*family, "fixed" | "fixedcap") != fixed {
            continue;
        }
        let capture = if matches!(*family, "cap" | "fixedcap") {
            capture_dependency_cap(driver, mode)
        } else {
            capture_dependency_coordinates(driver, mode)
        };
        native_golden::assert_capture(&capture, rgba, depth, mode);
    }
}
fn capture_dependency_coordinates(
    driver: &Handle<Graphic3dRenderDriver>,
    mode: &str,
) -> FrameCapture {
    let fixture = generation_scene::TextureGenerationScene::new(driver, mode);
    fixture.scene.render_view().try_redraw().unwrap();
    fixture.mutate(mode);
    let capture = redraw_dependency_capture(&fixture.scene);
    fixture.scene.view.remove();
    capture
}
fn capture_dependency_cap(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> FrameCapture {
    let fixture = capping_scene::CappingTextureScene::new(driver, mode);
    fixture.scene.render_view().try_redraw().unwrap();
    fixture.mutate(mode);
    let capture = redraw_dependency_capture(&fixture.scene);
    fixture.scene.view.remove();
    capture
}
fn redraw_dependency_capture(scene: &scene::ViewerScene) -> FrameCapture {
    use rustycad_tkservice::graphic3d::*;
    scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    scene.cview().set_z_layer_redraw_mode(true);
    let render = scene.render_view();
    render.try_redraw().unwrap();
    render.capture().unwrap()
}
