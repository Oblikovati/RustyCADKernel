#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/examples/support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/examples/support/view_capture.rs"]
mod native_capture;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/view_setup.rs"]
mod view_setup;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/lighting_scene.rs"]
mod lighting_scene;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/fixed_lighting_scene.rs"]
mod fixed_scene;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/fixed_texture_scene.rs"]
mod fixed_texture_scene;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/fixed_cap_lights.rs"]
mod fixed_cap_lights;
#[allow(dead_code)]
#[path = "/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/release-0.2.3-runner-replay/crates/vulkan/tests/support/capping_texture_policy.rs"]
mod cap_policy;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 { return Err(format!("received args={args:?}; expected family mode capture-prefix").into()); }
    let driver = capture_driver(&args[1], &args[2])?;
    match args[1].as_str() {
        "lighting" => capture_lighting(&driver, &args[2], &args[3])?,
        "textures" => capture_texture(&driver, &args[2], &args[3])?,
        "caps" | "cap-filter" => capture_cap(&driver, &args[2], &args[3], args[1] == "caps")?,
        family => return Err(format!("received family={family:?}; expected lighting/textures/caps/cap-filter").into()),
    }
    assert!(driver.validation_errors().is_empty(), "{:?}", driver.validation_errors());
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    Ok(())
}
fn capture_driver(family: &str, mode: &str) -> Result<Handle<Graphic3dRenderDriver>, martensite_vulkan::RenderError> {
    if family != "cap-filter" { return fixed_scene::fixed_lighting_driver(device_options::validation_options()); }
    let (coordinates, edges) = cap_policy::cap_fixture_pipelines(mode);
    Graphic3dRenderDriver::with_surface_pipelines(device_options::validation_options(), coordinates, edges)
}
fn capture_lighting(driver: &Handle<Graphic3dRenderDriver>, mode: &str, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = fixed_scene::fixed_lighting_scene(driver, mode);
    fixture.base.render_view().try_redraw()?;
    fixture.mutate(mode);
    capture_scene(&fixture.base, prefix)
}
fn capture_texture(driver: &Handle<Graphic3dRenderDriver>, mode: &str, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = fixed_texture_scene::fixed_texture_scene(driver, mode);
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(mode.strip_prefix("lit-").unwrap_or(mode));
    capture_scene(&fixture.scene, prefix)
}
fn capture_cap(driver: &Handle<Graphic3dRenderDriver>, mode: &str, prefix: &str, lighting: bool) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = capping_texture_scene::CappingTextureScene::new(driver, mode);
    if lighting { fixed_cap_lights::fixed_cap_light_source(&fixture.scene, mode); }
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(mode);
    capture_scene(&fixture.scene, prefix)
}
fn capture_scene(scene: &scene::ViewerScene, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    native_capture::capture_view(&scene.view, 64, 64, prefix, false)?;
    scene.view.remove();
    Ok(())
}
