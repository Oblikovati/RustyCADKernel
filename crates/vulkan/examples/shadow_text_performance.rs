//! Optimized completed-frame Shadow text and named AIS light timings, without validation overhead.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/light_source_scene.rs"]
mod light_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
use light_scene::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkv3d::{ais::*, v3d::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    let text = text_scene::TextScene::new(&driver);
    text.text
        .set_text_str(&vec!["AV".repeat(16); 32].join("\n"));
    text.text.set_height(8.);
    text.scene.window.set_size(512, 512);
    text.scene.view.must_be_resized();
    text.apply("text-shadow-basic");
    measure(&text.scene.view, "shadow-text-1024-glyphs");
    text.scene.view.remove();
    measure_named_lights(&driver)?;
    Ok(())
}
fn measure_named_lights(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
) -> Result<(), Box<dyn std::error::Error>> {
    for kind in 0..4 {
        let (viewer, view) = light_scene::light_view(driver);
        let light = light_scene::rendered_light(kind)?;
        light.set_display_name(true);
        light.light().set_name(&"Light".into());
        let context = AisInteractiveContext::new(&viewer)?;
        context.display(&light.cast(), false)?;
        measure(&view, &format!("named-light-{kind}"));
        view.remove();
    }
    Ok(())
}
fn measure(view: &V3dView, name: &str) {
    let render = view.view().cast::<Graphic3dRenderView>();
    for _ in 0..20 {
        render.try_redraw().unwrap();
    }
    let samples: Vec<f64> = (0..100)
        .map(|_| {
            let start = std::time::Instant::now();
            render.try_redraw().unwrap();
            start.elapsed().as_secs_f64() * 1000.
        })
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{name}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    assert!(render.capture().is_ok());
}
