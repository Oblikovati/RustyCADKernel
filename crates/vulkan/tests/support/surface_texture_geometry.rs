//! UVs, normals and winding are real Graphic3d attribute/index buffers, not renderer shortcuts.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
pub fn surface_texture_quad(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let array = allocate_texture_quad(mode);
    let points = [
        ([-8., -8., 0.], [0., 0.]),
        ([8., -8., 0.], [1., 0.]),
        ([8., 8., 0.], [1., 1.]),
        ([-8., 8., 0.], [0., 1.]),
    ];
    for (point, uv) in points {
        array
            .add_vertex_normal_texel(point, [0., 0., 1.], uv)
            .unwrap();
    }
    let indices = if mode.starts_with("back") {
        [1, 3, 2, 1, 4, 3]
    } else {
        [1, 2, 3, 1, 3, 4]
    };
    for index in indices {
        array.add_edge(index).unwrap();
    }
    if mode.ends_with("vertex") {
        for index in 1..=4 {
            array.set_vertex_color_rgb(index, 0.25, 0.5, 0.75).unwrap();
        }
    }
    array
}
fn allocate_texture_quad(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let colored = if mode.ends_with("vertex") {
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
    } else {
        0
    };
    let flags = GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL | GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL | colored;
    Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Triangles, 4, 0, 6, flags)
            .unwrap(),
    )
}
