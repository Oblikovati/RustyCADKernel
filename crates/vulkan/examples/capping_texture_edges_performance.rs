//! Complete retained cap redraw timings include real image, sampler and plane updates.
#[path = "../tests/support/capping_texture_policy.rs"]
mod cap_policy;
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/retained_redraw_samples.rs"]
mod redraw_samples;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkservice::graphic3d::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for mode in cap_policy::CAP_TEXTURE_EDGE_MODES {
        measure_cap_workload(mode)?;
    }
    Ok(())
}
fn measure_cap_workload(mode: &str) -> Result<(), martensite_vulkan::RenderError> {
    let (texture, edges) = cap_policy::cap_fixture_pipelines(mode);
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::with_surface_pipelines(options, texture, edges)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    let fixture = capping_texture_scene::CappingTextureScene::new(&driver, mode);
    let samples = redraw_samples::measure_redraws(&fixture.scene, |frame| {
        refresh_cap_workload(&fixture, mode, frame)
    });
    redraw_samples::report_samples(&format!("texture-section-edge-{mode}"), samples);
    fixture.scene.view.remove();
    Ok(())
}
fn refresh_cap_workload(
    fixture: &capping_texture_scene::CappingTextureScene,
    mode: &str,
    frame: usize,
) {
    fixture.mutate(mode);
    let alternate = frame.is_multiple_of(2);
    if mode.contains("sampler-updated") {
        fixture.texture.get_params().set_filter(if alternate {
            Graphic3dTypeOfTextureFilter::Nearest
        } else {
            Graphic3dTypeOfTextureFilter::Bilinear
        });
    }
    refresh_cap_plane(fixture, mode, alternate);
    refresh_cap_alpha(fixture, mode, alternate);
}
fn refresh_cap_plane(
    fixture: &capping_texture_scene::CappingTextureScene,
    mode: &str,
    alternate: bool,
) {
    if mode.contains("plane-updated") {
        fixture
            .plane
            .set_equation(Vec4::new(0.6, 0., -0.8, if alternate { -2. } else { -1. }))
            .unwrap();
    }
}
fn refresh_cap_alpha(
    fixture: &capping_texture_scene::CappingTextureScene,
    mode: &str,
    alternate: bool,
) {
    if mode.contains("alpha-updated") {
        fixture.plane.capping_aspect().set_alpha_mode(
            if alternate {
                Graphic3dAlphaMode::Opaque
            } else {
                Graphic3dAlphaMode::Blend
            },
            0.5,
        );
    }
}
#[cfg(test)]
#[allow(dead_code)]
#[path = "../tests/support/recording_device.rs"]
mod recording_device;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_cap_update_workloads_alternate_real_sampler_plane_and_alpha_state() {
        let recorder = recording_device::RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend(recorder, 128).unwrap();
        let mode = "cap-oblique-sampler-updated-plane-updated-alpha-updated";
        let fixture = capping_texture_scene::CappingTextureScene::new(&driver, mode);
        refresh_cap_workload(&fixture, mode, 0);
        assert_eq!(
            fixture.texture.get_params().filter(),
            Graphic3dTypeOfTextureFilter::Nearest
        );
        assert_eq!(fixture.plane.get_equation().w(), -2.);
        assert_eq!(
            fixture.plane.capping_aspect().alpha_mode(),
            Graphic3dAlphaMode::Opaque
        );
        refresh_cap_workload(&fixture, mode, 1);
        assert_eq!(
            fixture.texture.get_params().filter(),
            Graphic3dTypeOfTextureFilter::Bilinear
        );
        assert_eq!(fixture.plane.get_equation().w(), -1.);
        assert_eq!(
            fixture.plane.capping_aspect().alpha_mode(),
            Graphic3dAlphaMode::Blend
        );
        fixture.scene.view.remove();
    }
}
