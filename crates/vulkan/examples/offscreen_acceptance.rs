//! Explicit Vulkan runtime acceptance checks, separate from device-independent workspace tests.
use martensite_vulkan::*;
fn triangle(depth: f32, color: [f32; 4]) -> TriangleDraw {
    let vertices = [[-0.8, -0.8, depth], [0.8, -0.8, depth], [0., 0.8, depth]]
        .map(|p| RenderVertex::new(p, color));
    TriangleDraw::new(vertices.to_vec(), vec![0, 1, 2], identity_clip()).unwrap()
}
fn vulkan_offscreen_depth_orientation_and_repeated_resize() {
    let mut renderer = VulkanRenderer::new(VulkanOptions {
        validation: true,
        ..Default::default()
    })
    .unwrap();
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
    println!("PASS: 2 Vulkan runtime checks; validation errors: 0");
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
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
}
