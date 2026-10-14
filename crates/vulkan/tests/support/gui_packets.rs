//! Test packets use independent pixel rectangles and straight linear colors.
#![allow(dead_code)]
use martensite_vulkan::*;
/// Create a straight-color clipped packet; `gui_quad([64,64],color,rect)`.
pub fn gui_quad(extent: [u32; 2], color: [f32; 4], clipping: [f32; 4]) -> RenderLayer {
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let draw = gui_image_quad(extent, color, clipping, image);
    RenderLayer::new(vec![draw], false, false, false)
}
/// Own the atlas in a draw; `gui_image_quad(extent,color,rect,image)`.
pub fn gui_image_quad(
    extent: [u32; 2],
    color: [f32; 4],
    clipping: [f32; 4],
    image: RasterImage,
) -> PrimitiveDraw {
    let positions = [[8., 8., 0.], [56., 8., 0.], [56., 56., 0.], [8., 56., 0.]];
    let vertices = positions
        .map(|point| RenderVertex::new(point, color))
        .to_vec();
    let uv = vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
    let style = TexturedStyle::new(image, uv, 0.).unwrap();
    PrimitiveDraw::textured(vertices, vec![0, 1, 2, 0, 2, 3], pixel_clip(extent), style)
        .unwrap()
        .with_clipping(rectangle_clip(clipping))
}
/// Match the app OpenGL clip convention; `pixel_clip([64,64])`.
pub fn pixel_clip([width, height]: [u32; 2]) -> [f32; 16] {
    let x = 2. / width as f32;
    let y = -2. / height as f32;
    [
        x, 0., 0., 0., 0., y, 0., 0., 0., 0., 1., 0., -1., 1., 0., 1.,
    ]
}
/// Intersect all four sides independently; `rectangle_clip([0.,0.,64.,64.])`.
pub fn rectangle_clip([left, top, right, bottom]: [f32; 4]) -> RenderClipping {
    let planes = [
        [1., 0., 0., -left],
        [-1., 0., 0., right],
        [0., 1., 0., -top],
        [0., -1., 0., bottom],
    ];
    RenderClipping::new(
        planes.into_iter().map(|plane| vec![plane]).collect(),
        identity_clip(),
    )
    .unwrap()
}
