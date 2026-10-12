//! Indexed, direct and bounded native arrays retain their original triangle ordering.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
const EDGE_POINTS: [[f32; 3]; 4] = [[-8., -8., 0.], [8., -8., 0.], [8., 8., 0.], [-8., 8., 0.]];
const EDGE_RANKS: [usize; 6] = [0, 1, 2, 0, 2, 3];
pub fn edge_triangle_array(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let array = allocate_edge_triangles(mode);
    let direct = mode.contains("unindexed");
    let ranks = edge_triangle_ranks(mode);
    for (index, rank) in ranks.iter().enumerate().take(if direct { 6 } else { 4 }) {
        append_edge_triangle_vertex(&array, mode, if direct { *rank } else { index });
    }
    if !direct {
        append_edge_triangle_indices(&array, ranks);
    }
    array
}
fn edge_triangle_ranks(mode: &str) -> [usize; 6] {
    if mode.contains("back") {
        return [0, 2, 1, 0, 3, 2];
    }
    EDGE_RANKS
}
fn append_edge_triangle_indices(array: &Graphic3dArrayOfPrimitives, ranks: [usize; 6]) {
    for rank in ranks {
        array.add_edge(rank as i32 + 1).unwrap();
    }
}
fn allocate_edge_triangles(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let direct = mode.contains("unindexed");
    let bounded = mode.contains("bounds");
    let flags = edge_triangle_flags(mode);
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(
            Graphic3dTypeOfPrimitiveArray::Triangles,
            if direct { 6 } else { 4 },
            if bounded { 2 } else { 0 },
            if direct { 0 } else { 6 },
            flags,
        )
        .unwrap(),
    );
    if bounded {
        array.add_bound(3).unwrap();
        array.add_bound(3).unwrap();
    }
    array
}
fn edge_triangle_flags(mode: &str) -> i32 {
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL
        | GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL
        | if mode.contains("vertex") {
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
        } else {
            0
        }
}
fn append_edge_triangle_vertex(array: &Graphic3dArrayOfPrimitives, mode: &str, rank: usize) {
    let mut point = EDGE_POINTS[rank];
    if mode.contains("perspective") {
        point[2] = if rank < 2 { -4. } else { 4. };
    }
    let vertex = array
        .add_vertex_normal_texel(
            point,
            [0., 0., 1.],
            [
                if rank == 1 || rank == 2 { 1. } else { 0. },
                if rank >= 2 { 1. } else { 0. },
            ],
        )
        .unwrap();
    if mode.contains("vertex") {
        array.set_vertex_color_rgb(vertex, 0.25, 0.5, 0.75).unwrap();
    }
}
pub fn edge_fixture_lines(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let polyline = mode.contains("polyline");
    let normals = mode.contains("phong") || mode.contains("gouraud") || mode.contains("facet");
    let flags = if normals {
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL
    } else {
        0
    } | if mode.contains("vertex") {
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
    } else {
        0
    };
    let kind = if polyline {
        Graphic3dTypeOfPrimitiveArray::Polylines
    } else {
        Graphic3dTypeOfPrimitiveArray::Segments
    };
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(kind, 4, i32::from(polyline), 4, flags).unwrap(),
    );
    if polyline {
        array.add_bound(4).unwrap();
    }
    for point in EDGE_POINTS {
        let rank = array.add_vertex(point[0], point[1], point[2]).unwrap();
        if normals {
            array.set_vertex_normal(rank, 0., 0., 1.).unwrap();
        }
        if mode.contains("vertex") {
            array.set_vertex_color_rgb(rank, 0.25, 0.5, 0.75).unwrap();
        }
        array.add_edge(rank).unwrap();
    }
    array
}
