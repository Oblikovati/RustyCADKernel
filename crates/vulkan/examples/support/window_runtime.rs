//! Presentation, replacement, dumps, multiple live windows and repeated swapchain reconstruction.
use crate::{device_options, native_window, window_scene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::v3d::V3dImageDumpOptions;
pub fn check_windows() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for mode in window_scene::MODES {
        let mut window = window_scene::WindowScene::new(&driver, 64, 64);
        window.redraw();
        window.apply(mode, &driver).unwrap();
        window.redraw();
        window.verify_pixels();
        check_dump(&window);
        window.scene.view.remove();
        assert!(window.scene.viewer.active_views().is_empty());
    }
    check_repeated_resize(&driver);
    check_multiple_windows(&driver);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
}
fn check_dump(window: &window_scene::WindowScene) {
    let before = window.native.snapshot();
    let image = rustycad_tkservice::image::ImagePixMap::default();
    let options = V3dImageDumpOptions {
        width: 32,
        height: 48,
        ..Default::default()
    };
    assert!(window.scene.view.to_pix_map(&image, &options).unwrap());
    assert_eq!(window.native.snapshot(), before);
    window.redraw();
    assert_eq!(window.verify_pixels(), before);
}
fn check_repeated_resize(driver: &Handle<Graphic3dRenderDriver>) {
    let window = window_scene::WindowScene::new(driver, 64, 64);
    for index in 0..24 {
        let size = if index % 2 == 0 { [80, 48] } else { [48, 80] };
        driver.set_vertical_sync(index % 3 == 0);
        window.resize(size[0], size[1]);
        window.redraw();
        window.verify_pixels();
    }
    driver.set_vertical_sync(false);
    window.scene.view.remove();
}
fn check_multiple_windows(driver: &Handle<Graphic3dRenderDriver>) {
    let window = window_scene::WindowScene::new(driver, 64, 64);
    let second = native_window::NativeWindow::new(64, 64);
    second.move_to(300, 100);
    let view = window.scene.viewer.create_view().unwrap();
    crate::view_setup::configure_view(&view);
    view.set_window(&second.window.cast(), 0).unwrap();
    let renderer = view
        .get()
        .unwrap()
        .view()
        .cast::<martensite_vulkan::Graphic3dRenderView>();
    for _ in 0..3 {
        window.redraw();
        window.verify_pixels();
        renderer.try_redraw().unwrap();
        second.wait_for_pixels(&renderer.capture().unwrap().rgba);
    }
    window.scene.view.remove();
    renderer.try_redraw().unwrap();
    second.wait_for_pixels(&renderer.capture().unwrap().rgba);
    view.remove();
    assert!(window.scene.viewer.active_views().is_empty());
}
