//! Compare real presented X11 pixels with GPU readback, then dump an offscreen target.
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/native_window.rs"]
mod native_window;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[path = "support/window_scene.rs"]
mod window_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, "window-base")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let mut window = window_scene::WindowScene::new(&driver, request.width, request.height);
    request.configure(&window.scene.window, &window.scene.view);
    window
        .scene
        .object
        .set_local_transformation(&Transform3::translation_vector(Vector3::new(
            request.offset,
            0.,
            0.,
        )))?;
    window.redraw();
    window.apply(&request.mode, &driver)?;
    window.redraw();
    let pixels = window.verify_pixels();
    std::fs::write(format!("{}.window.rgba", request.prefix), &pixels)?;
    view_capture::capture_view(
        &window.scene.view,
        request.width,
        request.height,
        &request.prefix,
        false,
    )?;
    assert_eq!(
        window.native.snapshot(),
        pixels,
        "offscreen dump changed the native window"
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    window.scene.view.remove();
    assert!(window.scene.viewer.active_views().is_empty());
    drop(window);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    Ok(())
}

#[path = "../tests/support/ais_scene.rs"]
mod ais_scene;
