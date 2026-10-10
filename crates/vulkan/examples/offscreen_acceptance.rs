//! Explicit Vulkan runtime acceptance checks, separate from device-independent workspace tests.
#[path = "support/ais_removal_runtime.rs"]
mod ais_removal_runtime;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
#[path = "support/ais_runtime.rs"]
mod ais_runtime;
#[path = "support/viewer_runtime.rs"]
mod viewer_runtime;
fn triangle(depth: f32, color: [f32; 4]) -> TriangleDraw {
    let vertices = [[-0.8, -0.8, depth], [0.8, -0.8, depth], [0., 0.8, depth]]
        .map(|p| RenderVertex::new(p, color));
    TriangleDraw::new(vertices.to_vec(), vec![0, 1, 2], identity_clip()).unwrap()
}
fn vulkan_offscreen_depth_orientation_and_repeated_resize() {
    let mut renderer = VulkanRenderer::new(device_options::validation_options()).unwrap();
    let front = triangle(-0.5, [1., 0., 0., 1.]);
    let back = triangle(0.5, [0., 1., 0., 1.]);
    for (width, height) in [(64, 64), (96, 48), (64, 64)] {
        let frame = RenderFrame::new(
            width,
            height,
            [0., 0., 1., 1.],
            vec![front.clone(), back.clone()],
        )
        .unwrap();
        check_capture(&renderer.render(&frame).unwrap());
    }
    check_validation(&renderer);
}
fn vulkan_rejects_unavailable_device() {
    let result = VulkanRenderer::new(VulkanOptions {
        validation: true,
        device_index: Some(usize::MAX),
    });
    assert!(result.is_err());
}

fn main() {
    vulkan_offscreen_depth_orientation_and_repeated_resize();
    vulkan_rejects_unavailable_device();
    vulkan_layer_depth_policies();
    viewer_runtime::check_viewer_lifecycle();
    ais_runtime::check_ais_lifecycle();
    ais_removal_runtime::check_ais_removal();
    surface_runtime::check_surface_lifecycle();
    persistence_runtime::check_persistence_lifecycle();
    line_runtime::check_line_lifecycle();
    color_runtime::check_linear_to_srgb_capture();
    stdprs_runtime::check_curve_lifecycle();
    ais_line_runtime::check_line_datums();
    ais_point_runtime::check_point_datums();
    text_runtime::check_text_lifecycle();
    ais_axis_runtime::check_axis_datums();
    plane_trihedron_runtime::check_plane_datums();
    println!("PASS: 16 Vulkan runtime checks; validation errors: 0");
}

fn check_capture(capture: &FrameCapture) {
    let (width, height) = (capture.width, capture.height);
    assert_eq!(capture.pixel(width / 2, height / 2), Some([255, 0, 0, 255]));
    assert_eq!(capture.pixel(0, 0), Some([0, 0, 255, 255]));
    assert!((capture.depth[(height / 2 * width + width / 2) as usize] - 0.25).abs() < 1e-6);
    assert_eq!(capture.depth[0], 1.0);
    assert_eq!(capture.pixel(width / 4, height / 4), Some([0, 0, 255, 255]));
    assert_eq!(
        capture.pixel(width / 4, height * 3 / 4),
        Some([255, 0, 0, 255])
    );
}

fn check_validation(renderer: &VulkanRenderer) {
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
}

fn vulkan_layer_depth_policies() {
    let mut renderer = VulkanRenderer::new(device_options::validation_options()).unwrap();
    for (test, write, clear, color, depth) in [
        (true, true, false, [255, 0, 0, 255], 0.25),
        (false, true, false, [0, 255, 0, 255], 0.25),
        (true, false, false, [0, 255, 0, 255], 0.75),
        (true, true, true, [0, 255, 0, 255], 0.75),
    ] {
        let layers = vec![
            RenderLayer::new(vec![triangle(-0.5, [1., 0., 0., 1.])], true, write, false),
            RenderLayer::new(vec![triangle(0.5, [0., 1., 0., 1.])], test, true, clear),
        ];
        check_layer_capture(&mut renderer, layers, color, depth);
    }
    check_empty_depth_clear(&mut renderer);
    check_validation(&renderer);
}
fn check_layer_capture(
    renderer: &mut VulkanRenderer,
    layers: Vec<RenderLayer>,
    color: [u8; 4],
    depth: f32,
) {
    let frame = RenderFrame::with_layers(64, 64, [0., 0., 1., 1.], layers).unwrap();
    let capture = renderer.render(&frame).unwrap();
    assert_eq!(capture.pixel(32, 32), Some(color));
    assert!((capture.depth[32 * 64 + 32] - depth).abs() < 1e-6);
}

fn check_empty_depth_clear(renderer: &mut VulkanRenderer) {
    let layers = vec![
        RenderLayer::new(vec![triangle(-0.5, [1., 0., 0., 1.])], true, true, false),
        RenderLayer::new(vec![], true, true, true),
    ];
    check_layer_capture(renderer, layers, [255, 0, 0, 255], 1.);
}

#[path = "support/device_options.rs"]
mod device_options;

#[path = "support/surface_runtime.rs"]
mod surface_runtime;

#[path = "support/persistence_runtime.rs"]
mod persistence_runtime;

#[path = "support/line_runtime.rs"]
mod line_runtime;

#[path = "support/color_runtime.rs"]
mod color_runtime;

#[path = "support/stdprs_runtime.rs"]
mod stdprs_runtime;

#[path = "../tests/support/ais_line_scene.rs"]
mod line_scene;

#[path = "support/ais_line_runtime.rs"]
mod ais_line_runtime;

#[path = "support/ais_point_runtime.rs"]
mod ais_point_runtime;
#[path = "../tests/support/ais_point_scene.rs"]
mod point_scene;

use viewer_runtime::scene;
#[path = "support/text_runtime.rs"]
mod text_runtime;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;

#[path = "support/ais_axis_runtime.rs"]
mod ais_axis_runtime;
#[path = "../tests/support/ais_axis_scene.rs"]
mod axis_scene;

#[path = "../tests/support/plane_trihedron_scene.rs"]
mod plane_scene;
#[path = "support/plane_trihedron_runtime.rs"]
mod plane_trihedron_runtime;
