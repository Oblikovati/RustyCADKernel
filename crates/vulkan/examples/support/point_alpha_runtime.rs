//! Point alpha and coverage preserve native draw order and layer depth writes.
use martensite_vulkan::*;
type AlphaCase = (f32, bool, bool, [u8; 3], f32);
fn point(depth: f32, alpha: f32, blend: bool, color: [f32; 3]) -> PrimitiveDraw {
    let style = PointStyle::new(5., [1, 1], vec![[1.; 4]])
        .unwrap()
        .with_alpha_blending(blend);
    PrimitiveDraw::points(
        vec![RenderVertex::new(
            [0., 0., depth],
            [color[0], color[1], color[2], alpha],
        )],
        vec![0],
        identity_clip(),
        style,
    )
    .unwrap()
}
/// Check point depth/blend/cutoff and native replay; `check_point_alpha()`.
pub fn check_point_alpha() {
    let mut renderer = VulkanRenderer::new(super::device_options::validation_options()).unwrap();
    check_alpha_analytics(&mut renderer);
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    super::point_alpha_goldens::check_point_goldens(&driver);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
}
fn check_alpha_analytics(renderer: &mut VulkanRenderer) {
    for case in [
        (0.25, false, true, [255, 0, 0], 0.25),
        (0.25, true, true, [137, 0, 225], 0.25),
        (0., true, true, [0, 255, 0], 0.75),
        (0.1, true, true, [0, 255, 0], 0.75),
        (0.25, true, false, [0, 255, 0], 1.),
    ] {
        assert_alpha_point(renderer, case);
    }
    check_plain_variants(renderer);
}
fn assert_alpha_point(renderer: &mut VulkanRenderer, case: AlphaCase) {
    let (alpha, blend, write, expected, depth) = case;
    let draws = vec![
        point(-0.5, alpha, blend, [1., 0., 0.]),
        point(0.5, 1., false, [0., 1., 0.]),
    ];
    let frame = RenderFrame::with_layers(
        64,
        64,
        [0., 0., 1., 1.],
        vec![RenderLayer::new(draws, true, write, true)],
    )
    .unwrap();
    assert_point_pixel(&renderer.render(&frame).unwrap(), expected, depth);
}
fn assert_point_pixel(capture: &FrameCapture, expected: [u8; 3], depth: f32) {
    let index = 32 * 64 + 32;
    let pixel: &[u8] = &capture.rgba[4 * index..4 * index + 4];
    for channel in 0..3 {
        assert!(
            pixel[channel].abs_diff(expected[channel]) <= 1,
            "received {pixel:?}; expected {expected:?}"
        );
    }
    assert_eq!(pixel[3], 255);
    assert_eq!(capture.depth[index], depth);
}
fn check_plain_variants(renderer: &mut VulkanRenderer) {
    for flags in 0..2 {
        let mut draw = point(-0.5, 0.25, false, [1., 0., 0.]);
        if flags != 0 {
            draw = draw.with_clipping(
                RenderClipping::new(vec![vec![[1., 0., 0., 1.]]], identity_clip()).unwrap(),
            );
        }
        let frame = RenderFrame::new(64, 64, [0., 0., 1., 1.], vec![draw]).unwrap();
        assert_point_pixel(&renderer.render(&frame).unwrap(), [255, 0, 0], 0.25);
    }
}
