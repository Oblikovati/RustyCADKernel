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
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 8 && args.len() != 9 {
        return Err(format!(
            "received args={args:?}; expected mode prefix samples factor layers reverse fixed"
        )
        .into());
    }
    let fixed = args[7].parse::<u8>()? != 0;
    let driver = Graphic3dRenderDriver::with_render_pipelines(
        device_options::validation_options(),
        if fixed {
            RenderTextureCoordinatePipeline::GeneratedCompatibility
        } else {
            RenderTextureCoordinatePipeline::Programmable
        },
        Default::default(),
        if fixed {
            RenderLightingPipeline::FixedFunction
        } else {
            RenderLightingPipeline::Programmable
        },
    )?;
    let fixture = oit_scene::OitScene::with_scene(
        &driver,
        args[6].parse::<u8>()? != 0,
        args.get(8).map_or("two", String::as_str),
    );
    fixture.parameters(
        &args[1],
        args[3].parse()?,
        args[4].parse()?,
        args[5].parse()?,
    );
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate_inputs(args.get(8).map_or("two", String::as_str));
    view_capture::capture_view(&fixture.scene.view, 64, 64, &args[2], args.get(8).is_some_and(|name| name.starts_with("layer-")))?;
    if !driver.validation_errors().is_empty() {
        return Err(format!(
            "received validation errors={:?}; expected none",
            driver.validation_errors()
        )
        .into());
    }
    eprintln!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.scene.view.remove();
    Ok(())
}
