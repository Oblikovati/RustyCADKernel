//! Analytic fragment-plane acceptance independent of the legacy reference captures.
use martensite_vulkan::*;
#[path = "support/device_options.rs"]
mod device_options;
fn quad(clipping: RenderClipping) -> PrimitiveDraw {
    let vertices = [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]]
        .map(|position| RenderVertex::new(position, [1., 0., 0., 1.]));
    PrimitiveDraw::new(vertices.to_vec(), vec![0, 1, 2, 0, 2, 3], identity_clip())
        .unwrap()
        .with_clipping(clipping)
}
fn check_volume(renderer: &mut VulkanRenderer, chains: Vec<Vec<[f32; 4]>>, translate: f32) {
    let mut model = identity_clip();
    model[12] = translate;
    let clipping = RenderClipping::new(chains.clone(), model).unwrap();
    let capture = renderer
        .render(&RenderFrame::new(64, 64, [0., 0., 1., 1.], vec![quad(clipping)]).unwrap())
        .unwrap();
    for y in 0..64 {
        for x in 0..64 {
            let point = [
                (x as f32 + 0.5) / 32. - 1. + translate,
                1. - (y as f32 + 0.5) / 32.,
                0.,
                1.,
            ];
            check_pixel(&capture, x, y, &chains, point);
        }
    }
}
fn check_pixel(capture: &FrameCapture, x: u32, y: u32, chains: &[Vec<[f32; 4]>], point: [f32; 4]) {
    let visible = chains.iter().all(|chain| {
        chain
            .iter()
            .any(|plane| plane.iter().zip(point).map(|(a, b)| a * b).sum::<f32>() >= 0.)
    });
    assert_eq!(
        capture.pixel(x, y),
        Some(if visible {
            [255, 0, 0, 255]
        } else {
            [0, 0, 255, 255]
        }),
        "pixel ({x}, {y})"
    );
    assert_eq!(
        capture.depth[(y * 64 + x) as usize],
        if visible { 0.5 } else { 1. }
    );
}
fn main() {
    let mut renderer = VulkanRenderer::new(device_options::validation_options()).unwrap();
    let (x, y) = ([1., 0., 0., 0.], [0., 1., 0., 0.]);
    for chains in [
        vec![vec![x]],
        vec![vec![x, y]],
        vec![vec![x], vec![y]],
        vec![vec![x, y], vec![[-1., 0., 0., 0.25]]],
        vec![],
    ] {
        for translation in [0., 0.125, -0.375] {
            check_volume(&mut renderer, chains.clone(), translation);
        }
    }
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    println!("PASS: 15 clipping scenes, 61440 color/depth pixels; validation errors: 0");
}
