//! Native MaskBlend is an in-place blend with the layer's existing depth writes.
use martensite_vulkan::*;
fn segment(depth: f32, color: [f32; 4], blend: bool) -> PrimitiveDraw {
    PrimitiveDraw::segments(
        vec![
            RenderVertex::new([-0.8, 0., depth], color),
            RenderVertex::new([0.8, 0., depth], color),
        ],
        vec![0, 1],
        identity_clip(),
        LineStyle::new(1., 0xffff, 1)
            .unwrap()
            .with_alpha_blending(blend),
    )
    .unwrap()
}
/// Verify forced blending and depth policy on the selected GPU; `check_line_alpha()`.
pub fn check_line_alpha() {
    let mut renderer = VulkanRenderer::new(super::device_options::validation_options()).unwrap();
    for (alpha, blend, write, expected) in [
        (0.25, false, true, [255, 0, 0]),
        (0.25, true, true, [137, 0, 225]),
        (0., true, true, [0, 0, 255]),
        (0.25, true, false, [0, 255, 0]),
    ] {
        assert_alpha_capture(&mut renderer, alpha, blend, write, expected);
    }
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
}
fn assert_alpha_capture(
    renderer: &mut VulkanRenderer,
    alpha: f32,
    blend: bool,
    write: bool,
    expected: [u8; 3],
) {
    let draws = vec![
        segment(-0.5, [1., 0., 0., alpha], blend),
        segment(0.5, [0., 1., 0., 1.], false),
    ];
    let frame = RenderFrame::with_layers(
        64,
        64,
        [0., 0., 1., 1.],
        vec![RenderLayer::new(draws, true, write, true)],
    )
    .unwrap();
    let capture = renderer.render(&frame).unwrap();
    assert_line_pixels(&capture, write, expected);
}
fn assert_line_pixels(capture: &FrameCapture, write: bool, expected: [u8; 3]) {
    let mut painted = 0;
    for (index, pixel) in capture.rgba.as_chunks::<4>().0.iter().enumerate() {
        if capture.depth[index] == 1. && *pixel == [0, 0, 255, 255] {
            continue;
        }
        painted += 1;
        assert_line_pixel(pixel, capture.depth[index], write, expected);
    }
    assert!(
        painted > 0,
        "received no fragments; expected visible color or written depth"
    );
}
fn assert_line_pixel(pixel: &[u8; 4], depth: f32, write: bool, expected: [u8; 3]) {
    for channel in 0..3 {
        assert!(
            pixel[channel].abs_diff(expected[channel]) <= 1,
            "received {pixel:?}; expected {expected:?}"
        );
    }
    assert_eq!(pixel[3], 255);
    assert_eq!(depth, if write { 0.25 } else { 1. });
}
