//! GPU depth readback proves that dynamic offset changes apply and reset between draws.
use martensite_vulkan::*;
fn triangle(units: f32) -> PrimitiveDraw {
    let vertices = [[-0.8, -0.8, 0.], [0.8, -0.8, 0.], [0., 0.8, 0.]]
        .map(|position| RenderVertex::new(position, [1., 0., 0., 1.]));
    PrimitiveDraw::new(vertices.to_vec(), vec![0, 1, 2], identity_clip())
        .unwrap()
        .with_polygon_offset(RenderPolygonOffset::new(1, 0., units).unwrap())
}
fn capture(renderer: &mut VulkanRenderer, draws: Vec<PrimitiveDraw>) -> FrameCapture {
    renderer
        .render(&RenderFrame::new(32, 32, [0., 0., 1., 1.], draws).unwrap())
        .unwrap()
}
pub fn check_polygon_offsets() {
    let mut renderer = VulkanRenderer::new(super::device_options::validation_options()).unwrap();
    let plain = capture(&mut renderer, vec![triangle(0.)]);
    let forward = capture(&mut renderer, vec![triangle(4.)]);
    let backward = capture(&mut renderer, vec![triangle(-4.)]);
    let reset = capture(&mut renderer, vec![triangle(4.), triangle(0.)]);
    let center = 16 * 32 + 16;
    assert!(forward.depth[center] > plain.depth[center]);
    assert!(backward.depth[center] < plain.depth[center]);
    assert_eq!(reset.depth[center].to_bits(), plain.depth[center].to_bits());
    assert_eq!(forward.rgba, plain.rgba);
    assert_eq!(backward.rgba, plain.rgba);
    assert!(renderer.validation_errors().is_empty());
}
