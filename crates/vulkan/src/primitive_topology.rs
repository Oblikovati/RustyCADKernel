//! Lower supported OpenGL surface assembly to Vulkan triangle lists (OCCT 3d097a0).
use crate::{RenderError, error::rejected};
use rustycad_tkservice::graphic3d::Graphic3dTypeOfPrimitiveArray;
pub(crate) fn triangle_indices(
    kind: Graphic3dTypeOfPrimitiveArray,
    indices: &[u32],
) -> Result<Vec<u32>, RenderError> {
    use Graphic3dTypeOfPrimitiveArray::{
        Polygons, Quadrangles, Quadranglestrips, Trianglefans, Triangles, Trianglestrips,
    };
    match kind {
        // OpenGL ignores incomplete primitives at the end of each draw, including each bound.
        Triangles => Ok(indices[..indices.len() / 3 * 3].to_vec()),
        Quadrangles => Ok(quad_triangles(indices)),
        Quadranglestrips => Ok(quad_strip_triangles(indices)),
        Trianglestrips => Ok(strip_triangles(indices)),
        Trianglefans | Polygons => Ok(fan_triangles(indices)),
        _ => Err(rejected(
            kind,
            "triangles, triangle strips, triangle fans, convex polygons, quadrangles or quadrangle strips",
        )),
    }
}
fn strip_triangles(indices: &[u32]) -> Vec<u32> {
    indices
        .windows(3)
        .enumerate()
        .flat_map(|(index, triple)| {
            if index.is_multiple_of(2) {
                [triple[0], triple[1], triple[2]]
            } else {
                [triple[1], triple[0], triple[2]]
            }
        })
        .collect()
}
fn fan_triangles(indices: &[u32]) -> Vec<u32> {
    let Some((&center, rim)) = indices.split_first() else {
        return vec![];
    };
    rim.windows(2)
        .flat_map(|edge| [center, edge[0], edge[1]])
        .collect()
}

pub(crate) fn line_indices(
    kind: Graphic3dTypeOfPrimitiveArray,
    indices: &[u32],
) -> Result<Vec<u32>, RenderError> {
    use Graphic3dTypeOfPrimitiveArray::{Polylines, Segments};
    match kind {
        Segments => Ok(indices[..indices.len() / 2 * 2].to_vec()),
        Polylines => Ok(indices
            .windows(2)
            .flat_map(|pair| [pair[0], pair[1]])
            .collect()),
        _ => Err(rejected(kind, "segments or polylines")),
    }
}

// Paired colored captures at OCCT 3d097a0 expose the native GL_QUADS 0-to-2 diagonal.
fn quad_triangles(indices: &[u32]) -> Vec<u32> {
    indices
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|q| [q[0], q[1], q[2], q[0], q[2], q[3]])
        .collect()
}
fn quad_strip_triangles(indices: &[u32]) -> Vec<u32> {
    indices
        .windows(4)
        .step_by(2)
        .flat_map(|q| [q[0], q[1], q[3], q[2], q[0], q[3]])
        .collect()
}
