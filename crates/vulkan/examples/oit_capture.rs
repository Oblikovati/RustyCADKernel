//! Capture real retained transparency; `oit_capture weighted /tmp/oit 0 0 4 0 0`.
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
use surface_texture_scene as texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{
    Graphic3dRenderDriver, RenderLightingPipeline, RenderTextureCoordinatePipeline,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = capture_arguments()?;
    let fixed = args[7].parse::<u8>()? != 0;
    let driver = capture_driver(fixed)?;
    let fixture = capture_fixture(&driver, &args)?;
    if !driver.validation_errors().is_empty() {
        return Err(format!(
            "received validation errors={:?}; expected none",
            driver.validation_errors()
        )
        .into());
    }
    eprintln!("MARTENSITE_DEVICE_IDENTITY={:?}", driver.driver_identity());
    fixture.scene.view.remove();
    Ok(())
}

fn capture_arguments() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 8 && args.len() != 9 {
        return Err(format!(
            "received args={args:?}; expected mode prefix samples factor layers reverse fixed"
        )
        .into());
    }
    Ok(args)
}
fn capture_driver(
    fixed: bool,
) -> Result<rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>, martensite_vulkan::RenderError>
{
    Graphic3dRenderDriver::with_render_pipelines(
        device_options::validation_options(),
        coordinate_policy(fixed),
        Default::default(),
        lighting_policy(fixed),
    )
}
fn capture_fixture(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    args: &[String],
) -> Result<oit_scene::OitScene, Box<dyn std::error::Error>> {
    let fixture = oit_scene::OitScene::with_scene(
        driver,
        args[6].parse::<u8>()? != 0,
        args.get(8).map_or("two", String::as_str),
    );
    configure_capture(&fixture, args)?;
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate_inputs(args.get(8).map_or("two", String::as_str));
    write_capture(&fixture, args)?;
    Ok(fixture)
}

fn coordinate_policy(fixed: bool) -> RenderTextureCoordinatePipeline {
    if fixed {
        return RenderTextureCoordinatePipeline::GeneratedCompatibility;
    }
    RenderTextureCoordinatePipeline::Programmable
}
fn lighting_policy(fixed: bool) -> RenderLightingPipeline {
    if fixed {
        return RenderLightingPipeline::FixedFunction;
    }
    RenderLightingPipeline::Programmable
}
fn configure_capture(
    fixture: &oit_scene::OitScene,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    fixture.parameters(
        &args[1],
        args[3].parse()?,
        args[4].parse()?,
        args[5].parse()?,
    );
    Ok(())
}
fn write_capture(
    fixture: &oit_scene::OitScene,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let whole = args.get(8).is_some_and(|name| name.starts_with("layer-"));
    view_capture::capture_view(&fixture.scene.view, 64, 64, &args[2], whole)
}
