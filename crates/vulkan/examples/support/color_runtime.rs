//! A midtone exercises sRGB encoding that primary-color captures cannot detect.
use martensite_vulkan::{RenderBackend, RenderFrame, VulkanRenderer};

pub fn check_linear_to_srgb_capture() {
    let mut renderer = VulkanRenderer::new(super::device_options::validation_options()).unwrap();
    let frame = RenderFrame::new(
        32,
        32,
        [0.125, 0.25, 0.5, 1.],
        vec![super::triangle(0., [0.5, 0.25, 0.125, 1.])],
    )
    .unwrap();
    let capture = renderer.render(&frame).unwrap();
    assert_srgb_pixel(capture.pixel(16, 16).unwrap(), [188, 137, 99, 255]);
    assert_srgb_pixel(capture.pixel(0, 0).unwrap(), [99, 137, 188, 255]);
    super::check_validation(&renderer);
}

fn assert_srgb_pixel(actual: [u8; 4], expected: [u8; 4]) {
    // llvmpipe encodes linear 0.5 as 187; RADV uses 188. Native comparison stays exact.
    assert_eq!(actual[3], expected[3]);
    for (value, reference) in actual[..3].iter().zip(&expected[..3]) {
        assert!(
            value.abs_diff(*reference) <= 1,
            "received {actual:?}; expected sRGB {expected:?} within one quantization byte"
        );
    }
}
