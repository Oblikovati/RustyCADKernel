//! Retained texgen and cap redraw/readback timings include real per-frame state updates.
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/texture_generation_scene.rs"]
mod generation_scene;
#[path = "support/retained_redraw_samples.rs"]
mod redraw_samples;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, RenderTextureCoordinatePipeline};
use redraw_samples::{measure_redraws, report_samples};
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for policy in [
        RenderTextureCoordinatePipeline::Programmable,
        RenderTextureCoordinatePipeline::GeneratedCompatibility,
    ] {
        let mut options = device_options::validation_options();
        options.validation = false;
        let driver = Graphic3dRenderDriver::with_texture_coordinate_pipeline(options, policy)?;
        println!("MARTENSITE_DEVICE={}", driver.device_name());
        generation_workloads(&driver, policy)?;
        cap_workloads(&driver, policy)?;
    }
    Ok(())
}
fn generation_workloads(
    driver: &Handle<Graphic3dRenderDriver>,
    policy: RenderTextureCoordinatePipeline,
) -> Result<(), Box<dyn std::error::Error>> {
    static_generation_workloads(driver, policy);
    for mode in ["object-live-planes", "eye-live-planes"] {
        let fixture = generation_scene::TextureGenerationScene::new(driver, mode);
        let samples = measure_redraws(&fixture.scene, |frame| {
            refresh_generation_planes(&fixture, mode, frame)
        });
        report_samples(&format!("texgen-{policy:?}-{mode}"), samples);
        fixture.scene.view.remove();
    }
    Ok(())
}
fn static_generation_workloads(
    driver: &Handle<Graphic3dRenderDriver>,
    policy: RenderTextureCoordinatePipeline,
) {
    for mode in [
        "object-no-uv",
        "eye-no-uv",
        "eye-shared-no-uv",
        "eye-shared-revision-no-uv",
        "eye-shared-reset-no-uv",
        "object-q",
        "eye-q",
    ] {
        let fixture = generation_scene::TextureGenerationScene::new(driver, mode);
        let samples = measure_redraws(&fixture.scene, |_| {});
        report_samples(&format!("texgen-{policy:?}-{mode}"), samples);
        fixture.scene.view.remove();
    }
}
fn refresh_generation_planes(
    fixture: &generation_scene::TextureGenerationScene,
    mode: &str,
    frame: usize,
) {
    let generation = if mode.starts_with("eye") {
        Graphic3dTypeOfTextureMode::Eye
    } else {
        Graphic3dTypeOfTextureMode::Object
    };
    let scale = if frame.is_multiple_of(2) {
        0.0625
    } else {
        0.03125
    };
    fixture.texture.get_params().set_gen_mode(
        generation,
        Vec4::new(scale, 0., 0., 0.5),
        Vec4::new(0., scale, 0., 0.5),
    );
    fixture.scene.group.synchronize_aspects();
}
fn cap_workloads(
    driver: &Handle<Graphic3dRenderDriver>,
    policy: RenderTextureCoordinatePipeline,
) -> Result<(), Box<dyn std::error::Error>> {
    static_cap_workloads(driver, policy);
    for update in ["image-stale", "image-refresh", "sampler", "plane", "alpha"] {
        let fixture = capping_scene::CappingTextureScene::new(driver, "oblique-mask-blend");
        let samples = measure_redraws(&fixture.scene, |frame| {
            refresh_cap_state(&fixture, update, frame)
        });
        report_samples(&format!("cap-{policy:?}-live-{update}"), samples);
        fixture.scene.view.remove();
    }
    Ok(())
}
const STATIC_CAP_WORKLOADS: &[&str] = &[
    "oblique",
    "oblique-mask",
    "oblique-mask-blend",
    "oblique-blend",
    "object-texture",
    "object-material-blend",
    "phong",
    "phong-mask-blend",
    "phong-blend",
    "object-generated-oblique",
    "eye-generated-oblique",
    "oblique-bc3",
];
fn static_cap_workloads(
    driver: &Handle<Graphic3dRenderDriver>,
    policy: RenderTextureCoordinatePipeline,
) {
    for mode in STATIC_CAP_WORKLOADS {
        let fixture = capping_scene::CappingTextureScene::new(driver, mode);
        report_samples(
            &format!("cap-{policy:?}-{mode}"),
            measure_redraws(&fixture.scene, |_| {}),
        );
        fixture.scene.view.remove();
    }
}
fn refresh_cap_state(fixture: &capping_scene::CappingTextureScene, update: &str, frame: usize) {
    match update {
        "image-stale" => refresh_cap_content(fixture, false),
        "image-refresh" => refresh_cap_content(fixture, true),
        "sampler" => refresh_cap_filter(fixture, frame),
        "plane" => refresh_cap_plane(fixture, frame),
        "alpha" => refresh_cap_alpha(fixture, frame),
        other => panic!(
            "received update={other}; expected image-stale/image-refresh/sampler/plane/alpha"
        ),
    }
}
fn refresh_cap_content(fixture: &capping_scene::CappingTextureScene, refresh: bool) {
    fixture.mutate("image-updated");
    if refresh {
        fixture
            .plane
            .set_capping_texture(fixture.texture.cast())
            .unwrap();
    }
}
fn refresh_cap_filter(fixture: &capping_scene::CappingTextureScene, frame: usize) {
    let filter = if frame.is_multiple_of(2) {
        Graphic3dTypeOfTextureFilter::Nearest
    } else {
        Graphic3dTypeOfTextureFilter::Bilinear
    };
    fixture.texture.get_params().set_filter(filter);
}
fn refresh_cap_plane(fixture: &capping_scene::CappingTextureScene, frame: usize) {
    let w = if frame.is_multiple_of(2) { -2. } else { -1. };
    fixture
        .plane
        .set_equation(Vec4::new(0.6, 0., -0.8, w))
        .unwrap();
}
fn refresh_cap_alpha(fixture: &capping_scene::CappingTextureScene, frame: usize) {
    let alpha = if frame.is_multiple_of(2) {
        Graphic3dAlphaMode::MaskBlend
    } else {
        Graphic3dAlphaMode::Blend
    };
    fixture.plane.capping_aspect().set_alpha_mode(alpha, 0.5);
}
#[cfg(test)]
#[path = "../tests/support/recording_device.rs"]
mod recording_device;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cap_benchmark_updates_exercise_content_sampler_geometry_and_ordering() {
        for update in ["image-stale", "image-refresh", "sampler", "plane", "alpha"] {
            assert_live_cap_command(update, update != "image-stale");
        }
    }
    fn assert_live_cap_command(update: &str, changes: bool) {
        let recorder = recording_device::RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend(recorder.clone(), 128).unwrap();
        let fixture = capping_scene::CappingTextureScene::new(&driver, "oblique-mask-blend");
        fixture.scene.render_view().try_redraw().unwrap();
        let before = recorder.last_frame();
        refresh_cap_state(&fixture, update, 1);
        fixture.scene.render_view().try_redraw().unwrap();
        assert_eq!(
            before.draws() != recorder.last_frame().draws(),
            changes,
            "{update}"
        );
        fixture.scene.view.remove();
    }
    #[test]
    fn generated_plane_benchmark_updates_keep_prior_frame_coordinates() {
        let recorder = recording_device::RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend_and_texture_coordinates(
            recorder.clone(),
            128,
            std::sync::Arc::new(martensite_vulkan::FreeTypeTextSource::default()),
            RenderTextureCoordinatePipeline::GeneratedCompatibility,
        )
        .unwrap();
        for mode in ["object-live-planes", "eye-live-planes"] {
            let fixture = generation_scene::TextureGenerationScene::new(&driver, mode);
            fixture.scene.render_view().try_redraw().unwrap();
            let before = recorder.last_frame();
            refresh_generation_planes(&fixture, mode, 1);
            fixture.scene.render_view().try_redraw().unwrap();
            assert_ne!(before.draws(), recorder.last_frame().draws(), "{mode}");
            fixture.scene.view.remove();
        }
    }
}
