//! Genuine normal/color arrays distinguish fixed point lighting from surface lighting.
use rustycad_tkernel::{
    quantity::{Color, ColorSpace},
    standard::Handle,
};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};

pub fn fixed_marker_array(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let colored = mode.contains("vertices");
    let flags = GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL
        | if colored {
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
        } else {
            0
        };
    let points = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Points, 3, 0, 0, flags)
            .unwrap(),
    );
    for (index, normal) in [[0., 0., 1.], [0.6, 0., 0.8], [0., 0.6, 0.8]]
        .into_iter()
        .enumerate()
    {
        let rank = index as i32 + 1;
        points
            .set_vertice(rank, -6.125 + 6.25 * index as f32, 0.375, 0.)
            .unwrap();
        points
            .set_vertex_normal(rank, normal[0], normal[1], normal[2])
            .unwrap();
        if colored {
            points
                .set_vertex_color(rank, Color::new(0.5, 0.75, 0.25, ColorSpace::Rgb).unwrap())
                .unwrap();
        }
    }
    points
}
pub fn fixed_marker_aspect(aspect: &Graphic3dAspects, mode: &str) {
    aspect.set_marker_scale(5.).unwrap();
    aspect.set_marker_type(if mode.contains("ball") {
        AspectTypeOfMarker::Ball
    } else if mode.contains("plus") {
        AspectTypeOfMarker::Plus
    } else {
        AspectTypeOfMarker::Point
    });
    if mode.contains("opacity") {
        let mut material = aspect.front_material();
        material.set_transparency(0.5).unwrap();
        aspect.set_front_material(material);
    }
    if mode.contains("user") {
        fixed_custom_marker(aspect, mode);
    }
}
fn fixed_custom_marker(aspect: &Graphic3dAspects, mode: &str) {
    use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
    let image = Handle::new(ImagePixMap::default());
    let pixels = [
        [128, 64, 32, 255],
        [32, 128, 64, 128],
        [64, 32, 128, 0],
        [255, 128, 64, 255],
    ]
    .into_iter()
    .flat_map(|pixel| [pixel; 4].into_iter().flatten())
    .collect();
    assert!(image.init_wrapper(ImageFormat::Rgba, Some(pixels), 4, 4, 16));
    image.set_top_down(!mode.contains("bottom"));
    aspect.set_marker_type(AspectTypeOfMarker::Userdefined);
    aspect.set_marker_image(Handle::new(
        Graphic3dMarkerImage::from_images(image, Handle::default()).unwrap(),
    ));
}
