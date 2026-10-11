//! Paired strokes distinguish connectivity, bound resets, color precedence and screen stipple.
#[path = "line_alpha_arrays.rs"]
pub mod alpha;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
pub const MODES: [&str; 15] = [
    "line-segments",
    "line-segments-indexed",
    "line-polyline",
    "line-polyline-indexed",
    "line-bound-colors",
    "line-vertex-colors",
    "line-wide",
    "line-fractional",
    "line-dash",
    "line-dot",
    "line-dotdash",
    "line-custom",
    "line-empty",
    "line-gradient",
    "line-depth",
];
/// Construct repeated indexed or sequential strokes; `line_array("line-polyline", 1)`.
pub fn line_array(mode: &str, repeats: i32) -> Handle<Graphic3dArrayOfPrimitives> {
    assert!(
        MODES.contains(&mode),
        "received {mode}; expected a line fixture mode"
    );
    let segments = mode.starts_with("line-segments");
    let indexed = mode.ends_with("indexed") || repeats > 1;
    let kind = if segments {
        Graphic3dTypeOfPrimitiveArray::Segments
    } else {
        Graphic3dTypeOfPrimitiveArray::Polylines
    };
    let flags = GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR
        | if matches!(mode, "line-vertex-colors" | "line-gradient") {
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
        } else {
            0
        };
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(
            kind,
            8,
            2 * repeats,
            if indexed { 8 * repeats } else { 0 },
            flags,
        )
        .unwrap(),
    );
    append_points(&array, mode);
    for _ in 0..repeats {
        array.add_bound_rgb(4, 1., 0., 0.).unwrap();
        array.add_bound_rgb(4, 0., 1., 0.).unwrap();
        if indexed {
            for index in 1..=8 {
                array.add_edge(index).unwrap();
            }
        }
    }
    array
}
fn append_points(array: &Graphic3dArrayOfPrimitives, mode: &str) {
    for shift in [0., 10.] {
        for [x, y, z] in [[-8., -6., 0.], [-2., -6., 0.], [-2., 6., 0.], [-8., 2., 0.]] {
            array.add_vertex(x + shift, y, z).unwrap();
        }
    }
    if matches!(mode, "line-vertex-colors" | "line-gradient") {
        for index in 1..=8 {
            let colors = [[1., 1., 0.], [1., 0., 1.], [0., 1., 1.], [1., 1., 1.]];
            let [r, g, b] = colors[if mode == "line-gradient" {
                (index as usize - 1) % 4
            } else {
                0
            }];
            array.set_vertex_color_rgb(index, r, g, b).unwrap();
        }
    }
    if mode == "line-depth" {
        for index in 1..=8 {
            let p = array.vertice(index).unwrap();
            array
                .set_vertice(
                    index,
                    p.x() as f32,
                    p.y() as f32,
                    [0., 2., -1., 3.][(index as usize - 1) % 4],
                )
                .unwrap();
        }
    }
}
/// Apply native line state; `configure_line_style(&group.aspects(), "line-dash")`.
pub fn configure_line_style(aspects: &dyn Graphic3dAspectsSource, mode: &str) {
    let width = match mode {
        "line-wide" => 3.,
        "line-fractional" => 2.5,
        _ => 1.,
    };
    aspects.set_line_width(width).unwrap();
    aspects.set_line_type(match mode {
        "line-dash" => AspectTypeOfLine::Dash,
        "line-dot" => AspectTypeOfLine::Dot,
        "line-dotdash" => AspectTypeOfLine::Dotdash,
        "line-empty" => AspectTypeOfLine::Empty,
        _ => AspectTypeOfLine::Solid,
    });
    if mode == "line-custom" {
        aspects.set_line_pattern(0x8421);
        aspects.set_line_stipple_factor(3).unwrap();
    }
}
/// Replace retained geometry and configure its live style; `replace_lines(group, "line-wide", 1)`.
pub fn replace_lines(
    group: &dyn Graphic3dGroup,
    mode: &str,
    repeats: i32,
) -> Handle<Graphic3dArrayOfPrimitives> {
    if mode.starts_with("line-alpha-") {
        return alpha::replace_alpha_lines(group, mode);
    }
    let aspects = group.aspects();
    configure_line_style(&*aspects, mode);
    group.clear(false);
    group.set_group_primitives_aspect(&aspects);
    let array = line_array(mode, repeats);
    group.add_primitive_array(&array, true);
    array
}
