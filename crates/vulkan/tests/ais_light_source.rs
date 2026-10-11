//! All four concrete light presentations submit visible geometry through Vulkan.
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "support/light_source_scene.rs"]
mod light_scene;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkv3d::ais::AisInteractiveContext;
#[test]
fn all_light_types_render_visible_native_geometry_without_validation_errors() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for kind in 0..4 {
        render_light_kind(&driver, kind);
    }
}

fn render_light_kind(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    kind: usize,
) {
    let (viewer, view) = light_scene::light_view(driver);
    let context = AisInteractiveContext::new(&viewer).unwrap();
    context
        .display(&light_scene::rendered_light(kind).unwrap().cast(), false)
        .unwrap();
    let render = rustycad_tkv3d::v3d::V3dView::view(&view).cast::<Graphic3dRenderView>();
    render.try_redraw().unwrap();
    let capture = render.capture().unwrap();
    assert_visible_light(&capture, kind);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    view.remove();
}

fn assert_visible_light(capture: &martensite_vulkan::FrameCapture, kind: usize) {
    let yellow = capture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[0] > 100 && pixel[1] > 100 && pixel[2] < 200)
        .count();
    assert!(
        yellow > 5,
        "light kind={kind} has yellow_pixels={yellow}; expected visible native geometry"
    );
}
