//! Analytic stencil parity, face culling and depth restoration on the actual Vulkan device.
use super::device_options;
use martensite_vulkan::*;
fn quad(z: f32, color: [f32; 4]) -> PrimitiveDraw {
    let vertices = [[-1., -1., z], [1., -1., z], [1., 1., z], [-1., 1., z]]
        .map(|p| RenderVertex::new(p, color))
        .to_vec();
    PrimitiveDraw::new(vertices, vec![0, 1, 2, 0, 2, 3], identity_clip()).unwrap()
}
fn batch(count: usize, color: [f32; 4]) -> Vec<PrimitiveDraw> {
    let clip = RenderClipping::new(vec![vec![[1., 0., 0., 0.]]], identity_clip()).unwrap();
    let mut draws: Vec<_> = (0..count)
        .map(|i| {
            quad(-0.75, [1.; 4])
                .with_clipping(clip.clone())
                .with_capping_pass(if i == 0 {
                    RenderCappingPass::MaskStart
                } else {
                    RenderCappingPass::MaskContinue
                })
                .unwrap()
        })
        .collect();
    draws.push(
        quad(0., color)
            .with_capping_pass(RenderCappingPass::Surface)
            .unwrap(),
    );
    draws
}
fn check_parity(renderer: &mut VulkanRenderer, count: usize) {
    let frame = RenderFrame::new(64, 64, [0., 0., 1., 1.], batch(count, [1., 0., 0., 1.])).unwrap();
    let capture = renderer.render(&frame).unwrap();
    for y in 0..64 {
        for x in 0..64 {
            let visible = x >= 32 && count % 2 == 1;
            assert_eq!(
                capture.pixel(x, y),
                Some(if visible {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                })
            );
            assert_eq!(
                capture.depth[(y * 64 + x) as usize],
                if visible { 0.5 } else { 1. }
            );
        }
    }
}
fn check_restoration(renderer: &mut VulkanRenderer) {
    let mut draws = batch(1, [1., 0., 0., 1.]);
    // A fresh batch must clear the earlier odd mask; its two masks cancel.
    draws.extend(batch(2, [0., 1., 0., 1.]));
    // Equal depth ordinary geometry passes LEQUAL after the cap's LESS pipeline.
    draws.push(quad(0., [1., 1., 0., 1.]));
    draws.push(quad(0.5, [0., 1., 0., 1.]));
    let capture = renderer
        .render(&RenderFrame::new(64, 64, [0., 0., 1., 1.], draws).unwrap())
        .unwrap();
    assert!(
        capture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [255, 255, 0, 255])
    );
    assert!(capture.depth.iter().all(|z| *z == 0.5));
}
fn check_culling(renderer: &mut VulkanRenderer) {
    for (mode, visible) in [
        (RenderFaceCulling::None, true),
        (RenderFaceCulling::Back, true),
        (RenderFaceCulling::Front, false),
    ] {
        let frame = RenderFrame::new(
            16,
            16,
            [0., 0., 1., 1.],
            vec![quad(0., [1., 0., 0., 1.]).with_face_culling(mode)],
        )
        .unwrap();
        let capture = renderer.render(&frame).unwrap();
        assert_eq!(
            capture.pixel(8, 8),
            Some(if visible {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            }),
            "{mode:?}"
        );
    }
}
pub fn check_capping() {
    let mut renderer = VulkanRenderer::new(device_options::validation_options()).unwrap();
    for count in [1, 2, 3, 4] {
        check_parity(&mut renderer, count);
    }
    check_restoration(&mut renderer);
    check_culling(&mut renderer);
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    println!(
        "PASS: stencil parity, batch reset, color/depth isolation, depth restoration and face culling; validation errors: 0"
    );
}
