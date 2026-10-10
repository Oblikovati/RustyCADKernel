//! Graphic3d buffers are read at frame extraction so shared buffer mutations stay visible.
use crate::{RenderError, RenderVertex, TriangleDraw, error::rejected};
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
#[derive(Clone)]
pub(crate) struct PrimitiveSubmission {
    pub kind: Graphic3dTypeOfPrimitiveArray,
    pub indices: Handle<dyn Graphic3dIndexBufferSource>,
    pub attributes: Handle<dyn Graphic3dBufferSource>,
    pub bounds: Handle<Graphic3dBoundBuffer>,
}
impl PrimitiveSubmission {
    pub fn positions(&self) -> Result<Vec<[f32; 3]>, RenderError> {
        let Some(buffer) = self.attributes.get() else {
            return Err(rejected("null attributes", "a Graphic3d vertex buffer"));
        };
        validate_count(buffer.nb_elements())?;
        let (index, offset, stride) = buffer
            .attribute_data(Graphic3dTypeOfAttribute::Pos)
            .ok_or_else(|| {
                rejected(
                    "missing position",
                    "a Vec2, Vec3 or Vec4 position attribute",
                )
            })?;
        let kind = buffer
            .attribute(index)
            .ok_or_else(|| rejected(index, "a valid position attribute"))?
            .data_type;
        (0..buffer.nb_elements())
            .map(|i| read_position(buffer, kind, offset + stride * i as usize))
            .collect()
    }
    pub fn draws(
        &self,
        color: [f32; 4],
        matrix: [f32; 16],
    ) -> Result<Vec<TriangleDraw>, RenderError> {
        if self.kind != Graphic3dTypeOfPrimitiveArray::Triangles {
            return Err(rejected(
                self.kind,
                "triangle primitives in the current renderer slice",
            ));
        }
        let positions = self.positions()?;
        let vertices = positions
            .into_iter()
            .enumerate()
            .map(|(i, p)| self.vertex(i, p, color))
            .collect::<Result<Vec<_>, _>>()?;
        let indices = self.read_indices(vertices.len())?;
        self.ranges(&indices)?
            .into_iter()
            .map(|range| TriangleDraw::new(vertices.clone(), range, matrix))
            .collect()
    }
    fn vertex(
        &self,
        index: usize,
        position: [f32; 3],
        fallback: [f32; 4],
    ) -> Result<RenderVertex, RenderError> {
        let buffer = &*self.attributes;
        let Some((attribute, offset, stride)) =
            buffer.attribute_data(Graphic3dTypeOfAttribute::Color)
        else {
            return Ok(RenderVertex::new(position, fallback));
        };
        let kind = buffer
            .attribute(attribute)
            .ok_or_else(|| rejected(attribute, "a valid color attribute"))?
            .data_type;
        let at = offset + stride * index;
        let color = match kind {
            Graphic3dTypeOfData::Vec4ub => buffer
                .read_at::<Vec4<u8>>(at)
                .map(|v| v.data().map(|c| f32::from(c) / 255.)),
            Graphic3dTypeOfData::Vec4 => buffer.read_at::<Vec4<f32>>(at).map(|v| v.data()),
            _ => return Err(rejected(kind, "Vec4ub or Vec4 vertex colors")),
        }
        .ok_or_else(|| rejected(at, "allocated color bytes"))?;
        Ok(RenderVertex::new(position, color))
    }
    fn read_indices(&self, count: usize) -> Result<Vec<u32>, RenderError> {
        if self.indices.is_null() {
            return (0..count)
                .map(|i| u32::try_from(i).map_err(|_| rejected(i, "a 32-bit vertex index")))
                .collect();
        }
        if !matches!(self.indices.stride(), 2 | 4) {
            return Err(rejected(self.indices.stride(), "16-bit or 32-bit indices"));
        }
        validate_count(self.indices.nb_elements())?;
        (0..self.indices.nb_elements())
            .map(|i| {
                self.indices
                    .index(i)
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or_else(|| rejected(i, "allocated nonnegative vertex index"))
            })
            .collect()
    }
    fn ranges(&self, indices: &[u32]) -> Result<Vec<Vec<u32>>, RenderError> {
        if self.bounds.is_null() {
            return Ok(vec![indices.to_vec()]);
        }
        validate_bounds(&self.bounds)?;
        let mut ranges = Vec::new();
        let mut offset = 0usize;
        for index in 0..self.bounds.state().nb_bounds {
            let count = bound_count(&self.bounds, index)?;
            let end = offset
                .checked_add(count)
                .ok_or_else(|| rejected(count, "a bound within the index count"))?;
            let range = indices
                .get(offset..end)
                .ok_or_else(|| rejected((offset, end), "a bound within the index count"))?;
            ranges.push(range.to_vec());
            offset = end;
        }
        Ok(ranges)
    }
}
fn read_position(
    buffer: &dyn Graphic3dBufferSource,
    kind: Graphic3dTypeOfData,
    offset: usize,
) -> Result<[f32; 3], RenderError> {
    let dimensions = match kind {
        Graphic3dTypeOfData::Vec2 => 2,
        Graphic3dTypeOfData::Vec3 | Graphic3dTypeOfData::Vec4 => 3,
        _ => return Err(rejected(kind, "Vec2, Vec3 or Vec4 positions")),
    };
    let mut position = [0.; 3];
    for (i, value) in position.iter_mut().enumerate().take(dimensions) {
        *value = buffer
            .read_at::<f32>(offset + i * 4)
            .ok_or_else(|| rejected(offset + i * 4, "allocated position bytes"))?;
    }
    Ok(position)
}

fn bound_count(bounds: &Graphic3dBoundBuffer, index: i32) -> Result<usize, RenderError> {
    bounds
        .bound(index)
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| rejected(index, "a nonnegative bound count"))
}
fn validate_count(count: i32) -> Result<(), RenderError> {
    if count < 0 {
        return Err(rejected(count, "a nonnegative buffer element count"));
    }
    Ok(())
}

fn validate_bounds(bounds: &Graphic3dBoundBuffer) -> Result<(), RenderError> {
    if bounds.state().has_colors {
        return Err(rejected(
            "bound colors",
            "uncolored bounds in the current renderer slice",
        ));
    }
    validate_count(bounds.state().nb_bounds)
}
