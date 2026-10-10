//! Graphic3d buffers are read at frame extraction so shared buffer mutations stay visible.
use crate::{PrimitiveDraw, RenderError, RenderVertex, error::rejected};
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
        line: Option<crate::LineStyle>,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        let positions = self.positions()?;
        let indices = self.read_indices(positions.len())?;
        let vertices = positions
            .into_iter()
            .enumerate()
            .map(|(index, position)| self.vertex(index, position, color))
            .collect::<Result<Vec<_>, _>>()?;
        crate::primitive_ranges::primitive_ranges(&self.bounds, &indices, color)?
            .into_iter()
            .map(|range| self.range_draw(range, &vertices, matrix, line))
            .collect()
    }
    fn range_draw(
        &self,
        range: crate::primitive_ranges::PrimitiveRange,
        vertices: &[RenderVertex],
        matrix: [f32; 16],
        line: Option<crate::LineStyle>,
    ) -> Result<PrimitiveDraw, RenderError> {
        let indices = if line.is_some() {
            crate::primitive_topology::line_indices(self.kind, &range.indices)?
        } else {
            crate::primitive_topology::triangle_indices(self.kind, &range.indices)?
        };
        let colored = self
            .attributes
            .attribute_data(Graphic3dTypeOfAttribute::Color)
            .is_some();
        let vertices = range_vertices(vertices, range.color, colored);
        match line {
            Some(style) => PrimitiveDraw::segments(vertices, indices, matrix, style),
            None => PrimitiveDraw::new(vertices, indices, matrix),
        }
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
}

fn read_position(
    buffer: &dyn Graphic3dBufferSource,
    kind: Graphic3dTypeOfData,
    offset: usize,
) -> Result<[f32; 3], RenderError> {
    validate_homogeneous_position(buffer, kind, offset)?;
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

fn validate_count(count: i32) -> Result<(), RenderError> {
    if count < 0 {
        return Err(rejected(count, "a nonnegative buffer element count"));
    }
    Ok(())
}

fn validate_homogeneous_position(
    buffer: &dyn Graphic3dBufferSource,
    kind: Graphic3dTypeOfData,
    offset: usize,
) -> Result<(), RenderError> {
    if kind != Graphic3dTypeOfData::Vec4 {
        return Ok(());
    }
    // OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 multiplies the full occVertex
    // in Graphic3d_ShaderManager.cxx; dropping a non-unit w changes clip coordinates.
    let w = buffer.read_at::<f32>(offset + 12);
    if w != Some(1.0) {
        return Err(rejected(
            w,
            "unit homogeneous w in the current renderer slice",
        ));
    }
    Ok(())
}

fn range_vertices(
    vertices: &[RenderVertex],
    color: [f32; 4],
    vertex_colors: bool,
) -> Vec<RenderVertex> {
    // OpenGl_PrimitiveArray::drawArray sets the bound's uniform; a vertex color still wins.
    vertices
        .iter()
        .map(|vertex| RenderVertex {
            color: if vertex_colors { vertex.color } else { color },
            ..*vertex
        })
        .collect()
}
