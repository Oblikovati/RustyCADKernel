//! Paired legacy forced-alpha lines distinguish order, depth writes and live state.
use rustycad_tkernel::{ncollection::Vec4, quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
pub const MODES: [&str; 11] = [
    "line-alpha-opaque",
    "line-alpha-auto",
    "line-alpha-blend",
    "line-alpha-mask",
    "line-alpha-forced",
    "line-alpha-zero",
    "line-alpha-vertices",
    "line-alpha-depth",
    "line-alpha-wide",
    "line-alpha-dash",
    "line-alpha-clipped",
];
/// Construct native alpha strokes; `alpha_array("line-alpha-vertices")`.
pub fn alpha_array(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    assert!(
        MODES.contains(&mode),
        "received {mode}; expected a native alpha line mode"
    );
    let colored = matches!(mode, "line-alpha-vertices" | "line-alpha-depth");
    let flags = if colored {
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
    } else {
        0
    };
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Segments, 6, 0, 0, flags)
            .unwrap(),
    );
    for (segment, y) in [-4.125, 0.375, 4.625].into_iter().enumerate() {
        append_alpha_segment(&array, mode, segment, y, colored);
    }
    array
}
/// Replace the retained array and alpha policy; `replace_alpha_lines(group, "line-alpha-forced")`.
pub fn replace_alpha_lines(
    group: &dyn Graphic3dGroup,
    mode: &str,
) -> Handle<Graphic3dArrayOfPrimitives> {
    let style = group.aspects();
    group.clear(false);
    configure_alpha_style(&*style, mode);
    group.set_group_primitives_aspect(&style);
    let array = alpha_array(mode);
    group.add_primitive_array(&array, true);
    configure_alpha_clipping(group, mode);
    array
}
fn append_alpha_segment(
    array: &Graphic3dArrayOfPrimitives,
    mode: &str,
    segment: usize,
    y: f32,
    colored: bool,
) {
    let (y, z) = if mode == "line-alpha-depth" {
        (0.375, [2., 0., -2.][segment])
    } else {
        (y, 0.)
    };
    let color = Vec4::from_array([[255, 0, 0, 64], [0, 255, 0, 128], [255, 255, 255, 0]][segment]);
    for x in [-8., 8.] {
        let index = array.add_vertex(x, y, z).unwrap();
        if colored {
            array.set_vertex_color4(index, color).unwrap();
        }
    }
}
fn configure_alpha_style(style: &dyn Graphic3dAspectsSource, mode: &str) {
    let opacity = if mode == "line-alpha-zero" { 0. } else { 0.25 };
    style.set_interior_color_rgba(ColorRgba::new(1., 0., 0., opacity).unwrap());
    style.set_alpha_mode(alpha_policy(mode), 0.9);
    style
        .set_line_width(if mode == "line-alpha-wide" { 3. } else { 1. })
        .unwrap();
    style.set_line_type(if mode == "line-alpha-dash" {
        AspectTypeOfLine::Dash
    } else {
        AspectTypeOfLine::Solid
    });
}
fn alpha_policy(mode: &str) -> Graphic3dAlphaMode {
    match mode {
        "line-alpha-opaque" => Graphic3dAlphaMode::Opaque,
        "line-alpha-auto" => Graphic3dAlphaMode::BlendAuto,
        "line-alpha-blend" => Graphic3dAlphaMode::Blend,
        "line-alpha-mask" => Graphic3dAlphaMode::Mask,
        _ => Graphic3dAlphaMode::MaskBlend,
    }
}
fn configure_alpha_clipping(group: &dyn Graphic3dGroup, mode: &str) {
    if mode != "line-alpha-clipped" {
        return;
    }
    let plane = Handle::new(Graphic3dClipPlane::from_equation(Vec4::new(1., 0., 0., 0.)).unwrap());
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    planes.append(plane);
    group.structure().set_clip_planes(&planes);
}
