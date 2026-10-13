//! Measure complete accepted retained OIT draws and live updates; `oit_performance`.
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
#[path = "support/retained_text_performance.rs"]
mod retained_text_performance;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{
    Graphic3dRenderDriver, RenderLightingPipeline, RenderTextureCoordinatePipeline,
};

#[allow(dead_code)]
#[path = "../tests/support/oit_finite_assets.rs"]
mod oit_finite_assets;
#[allow(dead_code)]
#[path = "../tests/support/oit_layer_assets.rs"]
mod oit_layer_assets;
use rustycad_tkernel::standard::Handle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for fixed in [false, true] {
        let driver = performance_driver(fixed)?;
        println!("MARTENSITE_DEVICE_IDENTITY={:?}", driver.driver_identity());
        for mode in ["unordered", "weighted", "peeling"] {
            measure_case(&driver, "two", mode, 0, fixed);
        }
        measure_finite_cases(&driver, fixed);
        measure_layer_cases(&driver, fixed);
        measure_live_parameters(&driver, fixed);
    }
    Ok(())
}
fn performance_driver(
    fixed: bool,
) -> Result<Handle<Graphic3dRenderDriver>, martensite_vulkan::RenderError> {
    let mut options = device_options::validation_options();
    options.validation = false;
    Graphic3dRenderDriver::with_render_pipelines(
        options,
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
    )
}
fn measure_finite_cases(driver: &Handle<Graphic3dRenderDriver>, fixed: bool) {
    for asset in oit_finite_assets::OIT_FINITE_ASSETS
        .iter()
        .filter(|asset| asset.fixed == fixed)
    {
        measure_case(driver, asset.fixture, asset.method, asset.samples, fixed);
    }
}
fn measure_layer_cases(driver: &Handle<Graphic3dRenderDriver>, fixed: bool) {
    for asset in oit_layer_assets::OIT_LAYER_ASSETS
        .iter()
        .filter(|asset| asset.fixed == fixed)
    {
        measure_case(driver, asset.fixture, asset.mode, 0, fixed);
    }
}
fn measure_case(
    driver: &Handle<Graphic3dRenderDriver>,
    scene: &str,
    mode: &str,
    samples: i32,
    fixed: bool,
) {
    let fixture = oit_scene::OitScene::with_scene(driver, false, scene);
    fixture.parameters(mode, samples, 0., 4);
    fixture.scene.render_view().try_redraw().unwrap();
    fixture.mutate_inputs(scene);
    let workload = format!("oit-{scene}-{mode}-samples{samples}-fixed{fixed}");
    retained_text_performance::measure_text_view(&fixture.scene.render_view(), &workload);
    fixture.finish();
}
fn measure_live_parameters(driver: &Handle<Graphic3dRenderDriver>, fixed: bool) {
    let fixture = oit_scene::OitScene::new(driver, false);
    let mut index = 0usize;
    let workload = format!("oit-live-weighted-peeling-fallback-fixed{fixed}");
    retained_text_performance::measure_text_operation(&workload, &mut || {
        fixture.parameters(
            if index.is_multiple_of(3) {
                "peeling"
            } else {
                "weighted"
            },
            4,
            0.,
            4,
        );
        fixture.scene.render_view().try_redraw().unwrap();
        index += 1;
    });
    fixture.finish();
}
