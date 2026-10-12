//! Capture genuine retained edge presentations; `triangle_edges_capture solid /tmp/edges`.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/triangle_edge_scene.rs"]
mod edge_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(format!("received args={args:?}; expected mode capture-prefix").into());
    }
    capture_edge_mode(&args[1], &args[2])
}
fn capture_edge_mode(mode: &str, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pipeline = if mode.contains("polygon") {
        martensite_vulkan::RenderMeshEdgePipeline::PolygonCompatibility
    } else {
        martensite_vulkan::RenderMeshEdgePipeline::Programmable
    };
    let driver = Graphic3dRenderDriver::with_mesh_edge_pipeline(
        device_options::validation_options(),
        pipeline,
    )?;
    let fixture = edge_scene::TriangleEdgeScene::new(&driver, mode);
    fixture.fixture.scene.render_view().try_redraw()?;
    view_capture::capture_view(&fixture.fixture.scene.view, 64, 64, prefix, false)?;
    validate_edge_capture(&driver)?;
    eprintln!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.fixture.scene.view.remove();
    Ok(())
}

fn validate_edge_capture(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !driver.validation_errors().is_empty() {
        return Err(format!(
            "received validation errors={:?}; expected none",
            driver.validation_errors()
        )
        .into());
    }
    Ok(())
}
