//! Native point/sprite alpha fixtures keep array order and layer depth writes.
use rustycad_tkernel::{ncollection::Vec4, quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
pub const MODES: [&str; 21] = [
    "point-alpha-opaque",
    "point-alpha-auto",
    "point-alpha-blend",
    "point-alpha-mask",
    "point-alpha-forced",
    "point-alpha-zero",
    "point-alpha-cutoff",
    "point-alpha-vertices",
    "point-alpha-depth",
    "point-alpha-clipped",
    "sprite-alpha-aspect",
    "sprite-alpha-opaque",
    "sprite-alpha-auto",
    "sprite-alpha-blend",
    "sprite-alpha-mask",
    "sprite-alpha-user",
    "sprite-alpha-user-vertices",
    "sprite-alpha-zero",
    "sprite-alpha-vertices",
    "sprite-alpha-color",
    "sprite-alpha-color-vertices",
];
/// Replace a live retained point array; `replace_alpha_points(group, "point-alpha-forced")`.
pub fn replace_alpha_points(group: &dyn Graphic3dGroup, mode: &str) {
    assert!(
        MODES.contains(&mode),
        "received {mode}; expected a native point alpha mode"
    );
    let style = group.aspects();
    group.clear(false);
    configure_point_alpha(&*style, mode);
    group.set_group_primitives_aspect(&style);
    group.add_primitive_array(&point_alpha_array(mode), true);
    configure_point_clipping(group, mode);
}
fn point_alpha_array(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let colored = mode.ends_with("vertices") || mode == "point-alpha-depth";
    let flags = if colored {
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
    } else {
        0
    };
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Points, 3, 0, 0, flags)
            .unwrap(),
    );
    for index in 0..3 {
        append_alpha_point(&array, mode, index, colored);
    }
    array
}
fn append_alpha_point(array: &Graphic3dArrayOfPrimitives, mode: &str, index: usize, colored: bool) {
    let x = if mode == "point-alpha-depth" {
        0.375
    } else {
        [-6.125, 0.375, 6.625][index]
    };
    let z = if mode == "point-alpha-depth" {
        [2., 0., -2.][index]
    } else {
        0.
    };
    let rank = array.add_vertex(x, 0.375, z).unwrap();
    if colored {
        let rgba = [[255, 0, 0, 64], [0, 255, 0, 128], [255, 255, 255, 0]][index];
        array
            .set_vertex_color4(rank, Vec4::from_array(rgba))
            .unwrap();
    }
}
fn configure_point_alpha(style: &dyn Graphic3dAspectsSource, mode: &str) {
    let alpha = if mode.ends_with("zero") {
        0.
    } else if mode.ends_with("cutoff") {
        0.1
    } else {
        0.25
    };
    style.set_interior_color_rgba(ColorRgba::new(1., 0., 0., alpha).unwrap());
    style.set_alpha_mode(point_alpha_mode(mode), 0.9);
    style.set_marker_type(alpha_marker_type(mode));
    configure_alpha_image(style, mode);
    style.set_marker_scale(5.).unwrap();
    style.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
}
fn configure_alpha_image(style: &dyn Graphic3dAspectsSource, mode: &str) {
    style.set_marker_image(Handle::default());
    if !mode.starts_with("sprite-alpha-user") {
        return;
    }
    let image = rustycad_tkservice::image::ImagePixMap::default();
    let pixels: Vec<u8> = (0..16)
        .flat_map(|index| [0, 255, 0, [0, 25, 64, 128][index % 4]])
        .collect();
    assert!(image.init_wrapper(
        rustycad_tkservice::image::ImageFormat::Rgba,
        Some(pixels),
        4,
        4,
        16
    ));
    image.set_top_down(true);
    let marker = Graphic3dMarkerImage::from_images(Handle::new(image), Handle::default()).unwrap();
    style.set_marker_image(Handle::new(marker));
}
fn alpha_marker_type(mode: &str) -> AspectTypeOfMarker {
    if mode.starts_with("sprite-alpha-user") {
        return AspectTypeOfMarker::Userdefined;
    }
    if mode.starts_with("sprite-alpha-color") {
        return AspectTypeOfMarker::Ball;
    }
    if mode.starts_with("sprite-") {
        return AspectTypeOfMarker::Plus;
    }
    AspectTypeOfMarker::Point
}
fn point_alpha_mode(mode: &str) -> Graphic3dAlphaMode {
    match mode {
        "point-alpha-opaque" | "sprite-alpha-opaque" => Graphic3dAlphaMode::Opaque,
        "point-alpha-auto" | "sprite-alpha-auto" => Graphic3dAlphaMode::BlendAuto,
        "point-alpha-blend" | "sprite-alpha-blend" => Graphic3dAlphaMode::Blend,
        "point-alpha-mask" | "sprite-alpha-mask" => Graphic3dAlphaMode::Mask,
        _ => Graphic3dAlphaMode::MaskBlend,
    }
}
fn configure_point_clipping(group: &dyn Graphic3dGroup, mode: &str) {
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    if mode == "point-alpha-clipped" {
        planes.append(Handle::new(
            Graphic3dClipPlane::from_equation(rustycad_tkernel::ncollection::Vec4::new(
                1., 0., 0., 0.,
            ))
            .unwrap(),
        ));
    }
    group.structure().set_clip_planes(&planes);
}
