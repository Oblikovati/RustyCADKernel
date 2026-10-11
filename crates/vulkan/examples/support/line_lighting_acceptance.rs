//! Analytic nodal lighting checks preserve native solid coverage and clipping.
use martensite_vulkan::*;
use rustycad_tkservice::graphic3d::Graphic3dTypeOfLightSource;

fn lit_segment(shading: RenderShading, pattern: u16) -> PrimitiveDraw {
    let vertices =
        [[-0.8, 0., 0.], [0.8, 0., 0.]].map(|point| RenderVertex::new(point, [0., 0., 0., 1.]));
    let lighting = line_lighting_state(shading);
    PrimitiveDraw::segments(
        vertices.to_vec(),
        vec![0, 1],
        identity_clip(),
        LineStyle::new(3., pattern, 1).unwrap(),
    )
    .unwrap()
    .with_lighting(lighting)
    .unwrap()
}

fn line_lighting_state(shading: RenderShading) -> RenderLighting {
    RenderLighting::new(
        vec![[0., 0., 1.]; 2],
        super::material(),
        identity_clip(),
        identity_clip(),
        false,
        shading,
    )
    .unwrap()
    .with_lights(super::lights(Graphic3dTypeOfLightSource::Directional))
}

fn visible_lit_pixels(renderer: &mut VulkanRenderer, draw: PrimitiveDraw, clipped: bool) -> usize {
    let capture = renderer
        .render(&RenderFrame::new(64, 64, [0., 0., 0., 1.], vec![draw]).unwrap())
        .unwrap();
    let mut visible = 0;
    for (index, pixel) in capture.rgba.as_chunks::<4>().0.iter().enumerate() {
        if *pixel == [0, 0, 0, 255] {
            continue;
        }
        assert_eq!(*pixel, [137, 0, 137, 255]);
        assert_eq!(capture.depth[index], 0.5);
        assert!(!clipped || index % 64 >= 32);
        visible += 1;
    }
    assert!(visible > 0);
    visible
}

/// Check native solid fallback for patterned lit lines and clipped segments; `check_lit_lines(&mut renderer)`.
pub fn check_lit_lines(renderer: &mut VulkanRenderer) {
    for shading in [RenderShading::Gouraud, RenderShading::Phong] {
        let solid = visible_lit_pixels(renderer, lit_segment(shading, 0xffff), false);
        let patterned = visible_lit_pixels(renderer, lit_segment(shading, 0x00ff), false);
        assert_eq!(patterned, solid);
        let clipped = lit_segment(shading, 0xffff).with_clipping(
            RenderClipping::new(vec![vec![[1., 0., 0., 0.]]], identity_clip()).unwrap(),
        );
        assert!(visible_lit_pixels(renderer, clipped, true) < solid);
    }
}
