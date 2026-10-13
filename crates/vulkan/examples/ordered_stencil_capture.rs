//! Capture real retained ordered stencil elements; `ordered_stencil_capture enable weighted /tmp/stencil`.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/oit_scene.rs"]
mod oit_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/ordered_stencil_scene.rs"]
mod stencil_scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod surface_texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use capping_texture_scene::section_geometry as capping_scene;
use surface_texture_scene as texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "../tests/support/seeded_stencil_backend.rs"]
mod seeded_backend;
#[path = "support/view_capture.rs"]
mod view_capture;
use martensite_vulkan::Graphic3dRenderDriver;
struct StencilCaptureArgs {
    mode: String,
    method: String,
    prefix: String,
    seed: u8,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = stencil_capture_args(std::env::args().collect())?;
    capture_ordered_stencil(&args)
}
fn stencil_capture_args(
    args: Vec<String>,
) -> Result<StencilCaptureArgs, Box<dyn std::error::Error>> {
    if args.len() != 4 && args.len() != 5 {
        return Err(format!(
            "received args={args:?}; expected mode method prefix and optional seed0..255"
        )
        .into());
    }
    let seed = args.get(4).map_or(Ok(0), |value| value.parse::<u8>())?;
    Ok(StencilCaptureArgs {
        mode: args[1].clone(),
        method: args[2].clone(),
        prefix: args[3].clone(),
        seed,
    })
}
fn capture_ordered_stencil(args: &StencilCaptureArgs) -> Result<(), Box<dyn std::error::Error>> {
    let backend =
        seeded_backend::SeededStencilBackend::new(device_options::validation_options(), args.seed)?;
    let identity = backend.driver_identity();
    let driver = Graphic3dRenderDriver::with_backend(backend, 128)?;
    let fixture = stencil_scene::OrderedStencilScene::new(&driver, &args.mode, &args.method);
    fixture.initial_redraw(&args.mode)?;
    dump_stencil_fixture(&fixture, args)?;
    eprintln!("MARTENSITE_DEVICE_IDENTITY={identity:?}");
    fixture.finish();
    Ok(())
}
fn dump_stencil_fixture(
    fixture: &stencil_scene::OrderedStencilScene,
    args: &StencilCaptureArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    view_capture::capture_view(
        &fixture.fixture.scene.view,
        64,
        64,
        &args.prefix,
        args.mode.starts_with("layer-"),
    )?;
    dump_stencil_state(&fixture.fixture.scene.render_view(), &args.prefix)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_arguments_preserve_defaults_and_the_full_stencil_byte() {
        let args = vec!["probe", "enable", "weighted", "/tmp/view"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(stencil_capture_args(args).unwrap().seed, 0);
        let args = vec!["probe", "enable", "peeling", "/tmp/view", "255"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(stencil_capture_args(args).unwrap().seed, 255);
    }
    #[test]
    fn malformed_capture_arguments_and_out_of_range_seeds_are_rejected() {
        assert!(stencil_capture_args(vec![]).is_err());
        let args = vec!["probe", "enable", "weighted", "/tmp/view", "256"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert!(stencil_capture_args(args).is_err());
    }
}

fn dump_stencil_state(
    view: &martensite_vulkan::Graphic3dRenderView,
    prefix: &str,
) -> Result<(), std::io::Error> {
    let fields = view.stencil_context_state().native_state_fields();
    let state = fields
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(" ")
        + "\n";
    std::fs::write(format!("{prefix}.stencil-state"), state)?;
    Ok(())
}
