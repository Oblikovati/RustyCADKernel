//! Material opacity, pass ordering and depth suppression are observable GPU contracts.
use martensite_vulkan::*;
use rustycad_tkservice::graphic3d::Graphic3dTypeOfLightSource;

fn opacity_lighting(
    shading: RenderShading,
    front: f32,
    back: f32,
    colored: bool,
) -> RenderLighting {
    RenderLighting::new(
        vec![[0., 0., 1.]; 4],
        opacity_material(front, back),
        identity_clip(),
        identity_clip(),
        false,
        shading,
    )
    .unwrap()
    .with_lights(super::lights(Graphic3dTypeOfLightSource::Directional))
    .with_vertex_colors(colored, false)
}
fn opacity_material(front: f32, back: f32) -> RenderMaterial {
    let mut material = super::material();
    material.common[0].diffuse[3] = front;
    material.common[1].diffuse[3] = back;
    material
}
fn translucent_quad(shading: RenderShading, opacity: f32, vertex_alpha: f32) -> PrimitiveDraw {
    let (vertices, indices) =
        super::lighting_geometry::quad_geometry(0., false, [1., 1., 1., vertex_alpha]);
    PrimitiveDraw::new(vertices, indices, identity_clip())
        .unwrap()
        .with_lighting(opacity_lighting(
            shading,
            opacity,
            opacity,
            vertex_alpha != 1.,
        ))
        .unwrap()
}
fn check_pixels(
    renderer: &mut VulkanRenderer,
    draws: Vec<PrimitiveDraw>,
    expected: [u8; 4],
    depth: f32,
) {
    let capture = renderer
        .render(&RenderFrame::new(64, 64, [0., 0., 0., 1.], draws).unwrap())
        .unwrap();
    for pixel in capture.rgba.as_chunks::<4>().0 {
        assert_eq!(*pixel, expected);
    }
    for actual in capture.depth {
        assert_eq!(actual, depth);
    }
}
fn check_back_opacity(renderer: &mut VulkanRenderer, shading: RenderShading) {
    let (vertices, indices) = super::lighting_geometry::quad_geometry(0., true, [1.; 4]);
    let draw = PrimitiveDraw::new(vertices, indices, identity_clip())
        .unwrap()
        .with_lighting(opacity_lighting(shading, 1., 0.75, false))
        .unwrap();
    let red = if shading == RenderShading::PhongFacet {
        120
    } else {
        0
    };
    check_pixels(renderer, vec![draw], [red, 0, 120, 255], 1.);
}
fn check_ordered_depth(renderer: &mut VulkanRenderer, shading: RenderShading) {
    let (vertices, indices) = super::lighting_geometry::quad_geometry(0.2, false, [0., 1., 0., 1.]);
    let opaque = PrimitiveDraw::new(vertices, indices, identity_clip()).unwrap();
    let transparent = translucent_quad(shading, 0.5, 1.);
    // The foreground is submitted first; native unordered blending defers it after opaque geometry.
    let capture = renderer
        .render(&RenderFrame::new(64, 64, [0., 0., 0., 1.], vec![transparent, opaque]).unwrap())
        .unwrap();
    assert_ordered_blend_capture(&capture);
}
fn assert_ordered_blend_capture(capture: &FrameCapture) {
    for pixel in capture.rgba.as_chunks::<4>().0 {
        assert_eq!([pixel[0], pixel[2], pixel[3]], [99, 99, 255]);
        // Vulkan sRGB conversion precision permits these adjacent encodings of linear 0.5.
        assert!(
            (187..=188).contains(&pixel[1]),
            "received green {}; expected sRGB 0.5 in [187,188]",
            pixel[1]
        );
    }
    assert!(capture.depth.iter().all(|depth| *depth == 0.6));
}
fn check_clipped_opacity(renderer: &mut VulkanRenderer, shading: RenderShading) {
    let draw = translucent_quad(shading, 0.5, 1.)
        .with_clipping(RenderClipping::new(vec![vec![[1., 0., 0., 0.]]], identity_clip()).unwrap());
    let capture = renderer
        .render(&RenderFrame::new(64, 64, [0., 0., 0., 1.], vec![draw]).unwrap())
        .unwrap();
    assert_clipped_opacity_capture(&capture);
}
fn assert_clipped_opacity_capture(capture: &FrameCapture) {
    for (index, pixel) in capture.rgba.as_chunks::<4>().0.iter().enumerate() {
        assert_eq!(
            *pixel,
            if index % 64 >= 32 {
                [99, 0, 99, 255]
            } else {
                [0, 0, 0, 255]
            }
        );
        assert_eq!(capture.depth[index], 1.);
    }
}
/// Exercise material and vertex alpha across all Phong-family algorithms; `check_translucent_surfaces(&mut renderer)`.
pub fn check_translucent_surfaces(renderer: &mut VulkanRenderer) {
    for shading in [
        RenderShading::PhongFacet,
        RenderShading::Gouraud,
        RenderShading::Phong,
    ] {
        check_front_opacity(renderer, shading);
        check_back_opacity(renderer, shading);
        check_ordered_depth(renderer, shading);
        check_clipped_opacity(renderer, shading);
    }
}
fn check_front_opacity(renderer: &mut VulkanRenderer, shading: RenderShading) {
    for (opacity, vertex_alpha, expected) in [
        (0.5, 1., [99, 0, 99, 255]),
        (0., 1., [0, 0, 0, 255]),
        (1., 0.5, [99, 0, 99, 255]),
    ] {
        check_pixels(
            renderer,
            vec![translucent_quad(shading, opacity, vertex_alpha)],
            expected,
            1.,
        );
    }
    let opaque = translucent_quad(shading, 0.5, 1.)
        .with_surface_blending(false)
        .unwrap();
    check_pixels(renderer, vec![opaque], [137, 0, 137, 255], 0.5);
}
