//! Capture persistent actual-view fallback; `oit_transition /tmp/oit-live`.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/oit_scene.rs"]
mod oit_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod surface_texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use surface_texture_scene as texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
use martensite_vulkan::Graphic3dRenderDriver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = transition_prefix()?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = oit_scene::OitScene::new(&driver, false);
    capture_live_stages(&fixture, &prefix)?;
    fixture.scene.view.remove();
    require_clean_transition(&driver)?;
    eprintln!("MARTENSITE_DEVICE={}", driver.device_name());
    Ok(())
}
fn capture_live_stages(
    fixture: &oit_scene::OitScene,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for samples in [0, 1, 2, 3, 4, 6, 8] {
        capture_stage(
            fixture,
            "weighted",
            samples,
            &format!("{prefix}-weighted-{samples}"),
        )?;
    }
    capture_stage(fixture, "peeling", 4, &format!("{prefix}-peeling-4"))?;
    capture_stage(
        fixture,
        "weighted",
        4,
        &format!("{prefix}-after-weighted-4"),
    )?;
    Ok(())
}
fn require_clean_transition(
    driver: &Graphic3dRenderDriver,
) -> Result<(), Box<dyn std::error::Error>> {
    if !driver.validation_errors().is_empty() {
        return Err(format!(
            "received errors={:?}; expected validation-clean OIT transitions",
            driver.validation_errors()
        )
        .into());
    }
    Ok(())
}
fn transition_prefix() -> Result<String, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err(
            format!("received args={args:?}; expected executable and output prefix").into(),
        );
    }
    Ok(args[1].clone())
}
fn capture_stage(
    fixture: &oit_scene::OitScene,
    mode: &str,
    samples: i32,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fixture.parameters(mode, samples, 0., 4);
    fixture.scene.render_view().try_redraw()?;
    view_capture::capture_view(&fixture.scene.view, 64, 64, prefix, false)
}
