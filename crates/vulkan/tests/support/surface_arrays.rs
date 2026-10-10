//! Two separated surfaces expose primitive connectivity, index ordering and bound colors.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
/// Build paired surfaces; `surface_array("surface-strip-indexed", 1)`.
pub fn surface_array(mode: &str, repeats: i32) -> Handle<Graphic3dArrayOfPrimitives> {
    let (kind, indexed, vertex_colors) = surface_configuration(mode);
    let points = surface_points(kind);
    let size = points.len() as i32;
    let array = allocate_surface(kind, size, repeats, indexed, vertex_colors);
    for [x, y, z] in points {
        array.add_vertex(x, y, z).unwrap();
    }
    append_surface_bounds(&array, size / 2, repeats, indexed);
    if vertex_colors {
        for index in 1..=size {
            array.set_vertex_color_rgb(index, 1., 1., 0.).unwrap();
        }
    }
    array
}
fn surface_configuration(mode: &str) -> (Graphic3dTypeOfPrimitiveArray, bool, bool) {
    use Graphic3dTypeOfPrimitiveArray::{Trianglefans, Triangles, Trianglestrips};
    match mode {
        "surface-strip" => (Trianglestrips, false, false),
        "surface-strip-indexed" => (Trianglestrips, true, false),
        "surface-fan" => (Trianglefans, false, false),
        "surface-fan-indexed" => (Trianglefans, true, false),
        "surface-bound-colors" => (Triangles, true, false),
        "surface-vertex-colors" => (Trianglestrips, true, true),
        _ => panic!("received surface mode {mode}; expected a surface capture mode"),
    }
}
fn surface_points(kind: Graphic3dTypeOfPrimitiveArray) -> Vec<[f32; 3]> {
    let left = match kind {
        Graphic3dTypeOfPrimitiveArray::Triangles => {
            vec![[-8., -6., 0.], [-2., -6., 0.], [-5., 6., 0.]]
        }
        Graphic3dTypeOfPrimitiveArray::Trianglefans => {
            vec![[-8., -6., 0.], [-2., -6., 0.], [-2., 6., 0.], [-8., 6., 0.]]
        }
        _ => vec![[-8., -6., 0.], [-2., -6., 0.], [-8., 6., 0.], [-2., 6., 0.]],
    };
    left.iter()
        .copied()
        .chain(left.iter().map(|p| [p[0] + 10., p[1], p[2]]))
        .collect()
}
fn append_surface_bounds(
    array: &Graphic3dArrayOfPrimitives,
    size: i32,
    repeats: i32,
    indexed: bool,
) {
    for _ in 0..repeats {
        array.add_bound_rgb(size, 1., 0., 0.).unwrap();
        array.add_bound_rgb(size, 0., 1., 0.).unwrap();
        if indexed {
            for index in 1..=2 * size {
                array.add_edge(index).unwrap();
            }
        }
    }
}
/// Replace geometry while retaining its style; `replace_surface(group, "surface-fan", 1)`.
pub fn replace_surface(
    group: &dyn Graphic3dGroup,
    mode: &str,
    repeats: i32,
) -> Handle<Graphic3dArrayOfPrimitives> {
    let aspects = group.aspects();
    group.clear(false);
    group.set_group_primitives_aspect(&aspects);
    let array = surface_array(mode, repeats);
    group.add_primitive_array(&array, true);
    array
}

fn allocate_surface(
    kind: Graphic3dTypeOfPrimitiveArray,
    size: i32,
    repeats: i32,
    indexed: bool,
    colored: bool,
) -> Handle<Graphic3dArrayOfPrimitives> {
    let flags = GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR
        | if colored {
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
        } else {
            0
        };
    let edges = if indexed { size * repeats } else { 0 };
    Handle::new(Graphic3dArrayOfPrimitives::new(kind, size, 2 * repeats, edges, flags).unwrap())
}
