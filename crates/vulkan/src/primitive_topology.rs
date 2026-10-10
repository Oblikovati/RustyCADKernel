//! Lower supported OpenGL surface assembly to Vulkan triangle lists (OCCT 3d097a0).
use crate::{RenderError, error::rejected};
use rustycad_tkservice::graphic3d::Graphic3dTypeOfPrimitiveArray;
pub(crate) fn triangle_indices(
    kind: Graphic3dTypeOfPrimitiveArray,
    indices: &[u32],
) -> Result<Vec<u32>, RenderError> {
    use Graphic3dTypeOfPrimitiveArray::{Trianglefans, Triangles, Trianglestrips};
    match kind {
        // OpenGL ignores incomplete primitives at the end of each draw, including each bound.
        Triangles => Ok(indices[..indices.len() / 3 * 3].to_vec()),
        Trianglestrips => Ok(strip_triangles(indices)),
        Trianglefans => Ok(fan_triangles(indices)),
        _ => Err(rejected(
            kind,
            "triangles, triangle strips or triangle fans",
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
