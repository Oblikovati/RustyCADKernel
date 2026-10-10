//! Each bound is a separate native draw, with an optional uniform color fallback.
use crate::{RenderError, error::rejected};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::Graphic3dBoundBuffer;
pub(crate) struct PrimitiveRange {
    pub indices: Vec<u32>,
    pub color: [f32; 4],
}
pub(crate) fn primitive_ranges(
    bounds: &Handle<Graphic3dBoundBuffer>,
    indices: &[u32],
    fallback: [f32; 4],
) -> Result<Vec<PrimitiveRange>, RenderError> {
    let Some(bounds) = bounds.get() else {
        return Ok(vec![PrimitiveRange {
            indices: indices.to_vec(),
            color: fallback,
        }]);
    };
    let count = bounds.state().nb_bounds;
    if count < 0 {
        return Err(rejected(count, "a nonnegative bound count"));
    }
    let mut offset = 0usize;
    (0..count)
        .map(|index| {
            let range = bound_range(bounds, index, offset, indices, fallback)?;
            offset += range.indices.len();
            Ok(range)
        })
        .collect()
}
fn bound_range(
    bounds: &Graphic3dBoundBuffer,
    index: i32,
    offset: usize,
    indices: &[u32],
    fallback: [f32; 4],
) -> Result<PrimitiveRange, RenderError> {
    let stored = bounds.bound(index);
    let count = stored
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| rejected((index, stored), "an allocated nonnegative bound count"))?;
    let end = offset
        .checked_add(count)
        .ok_or_else(|| rejected(count, "a bound within the index count"))?;
    let range = indices
        .get(offset..end)
        .ok_or_else(|| rejected((offset, end), "a bound within the index count"))?;
    Ok(PrimitiveRange {
        indices: range.to_vec(),
        color: bound_color(bounds, index, fallback)?,
    })
}
fn bound_color(
    bounds: &Graphic3dBoundBuffer,
    index: i32,
    fallback: [f32; 4],
) -> Result<[f32; 4], RenderError> {
    if !bounds.state().has_colors {
        return Ok(fallback);
    }
    bounds
        .color(index)
        .map(|color| color.data())
        .ok_or_else(|| rejected(index, "an allocated bound color"))
}
