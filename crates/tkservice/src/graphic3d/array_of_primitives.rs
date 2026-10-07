//! `Graphic3d_ArrayOfPrimitives`: the vertex, index and bound buffers of one primitive array
//! (points, segments, polylines, triangles, strips, fans, quads, polygons).
//!
//! The vertex buffer keeps the native interleaved layout bit for bit (position `vec3`, then
//! the optional normal `vec3`, texel `vec2` and color `vec4ub`), which the Vulkan driver uploads
//! unchanged (visualization-design.md section 1). The vertex, edge and bound members are in
//! array_vertices.rs and array_edges.rs; the classes of each primitive type in array_classes.rs.
use super::{
    GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_DEINTERLEAVED, GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_MUTABLE,
    GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR, GRAPHIC3D_ARRAY_FLAGS_INDEXES_MUTABLE,
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR, GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL, Graphic3dArrayFlags, Graphic3dAttribBuffer,
    Graphic3dAttribute, Graphic3dBoundBuffer, Graphic3dBuffer, Graphic3dBufferBase,
    Graphic3dBufferMembers, Graphic3dBufferSource, Graphic3dIndexBuffer,
    Graphic3dIndexBufferMembers, Graphic3dIndexBufferSource, Graphic3dMutableIndexBuffer,
    Graphic3dTypeOfAttribute, Graphic3dTypeOfData, Graphic3dTypeOfPrimitiveArray,
};
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};

/// The byte offset and stride of an optional vertex attribute (`my*Data`, `my*Stride`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Graphic3dAttributeSlot {
    /// The byte offset of the first value, `None` when the array has no such attribute.
    pub offset: Option<usize>,
    /// The distance between two values.
    pub stride: usize,
}

/// A primitive array; `Graphic3dArrayOfPrimitives::create_array(Graphic3dTypeOfPrimitiveArray::Triangles, 3, 0, 0)?`.
pub struct Graphic3dArrayOfPrimitives {
    transient: StandardTransient,
    pub(crate) indices: Handle<dyn Graphic3dIndexBufferSource>,
    pub(crate) attribs: Handle<dyn Graphic3dBufferSource>,
    pub(crate) bounds: Handle<Graphic3dBoundBuffer>,
    pub(crate) position_stride: usize,
    pub(crate) normals: Graphic3dAttributeSlot,
    pub(crate) texels: Graphic3dAttributeSlot,
    pub(crate) colors: Graphic3dAttributeSlot,
    pub(crate) kind: Graphic3dTypeOfPrimitiveArray,
}
rustycad_tkernel::impl_transient!(
    Graphic3dArrayOfPrimitives,
    "Graphic3d_ArrayOfPrimitives",
    StandardTransient,
    transient
);

/// The vertex buffer of the flags: an attribute buffer for mutable or packed arrays.
fn vertex_buffer(
    allocator: &BaseAllocator,
    flags: Graphic3dArrayFlags,
) -> Result<Handle<dyn Graphic3dBufferSource>, Failure> {
    let (mutable, deinterleaved) = (
        flags & GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_MUTABLE != 0,
        flags & GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_DEINTERLEAVED != 0,
    );
    if !mutable && !deinterleaved {
        return Ok(Handle::new(Graphic3dBuffer::new(allocator.clone())?).cast());
    }
    let buffer = Graphic3dAttribBuffer::new(allocator.clone())?;
    buffer.set_mutable(mutable)?;
    buffer.set_interleaved(!deinterleaved)?;
    Ok(Handle::new(buffer).cast())
}

/// The index buffer of `max_edges` indices, 16-bit below 65535 vertices; `None` on failure.
fn index_buffer(
    allocator: &BaseAllocator,
    flags: Graphic3dArrayFlags,
    max_vertexs: i32,
    max_edges: i32,
) -> Result<Option<Handle<dyn Graphic3dIndexBufferSource>>, Failure> {
    let indices: Handle<dyn Graphic3dIndexBufferSource> =
        if flags & GRAPHIC3D_ARRAY_FLAGS_INDEXES_MUTABLE != 0 {
            Handle::new(Graphic3dMutableIndexBuffer::new(allocator.clone())?).cast()
        } else {
            Handle::new(Graphic3dIndexBuffer::new(allocator.clone())?).cast()
        };
    let Some(buffer) = indices.get() else {
        return Ok(None);
    };
    let ok = if max_vertexs < i32::from(u16::MAX) {
        buffer.init_indices::<u16>(max_edges)?
    } else {
        buffer.init_indices::<u32>(max_edges)?
    };
    if !ok {
        return Ok(None);
    }
    buffer.set_nb_elements(0);
    Ok(Some(indices))
}

/// The vertex attributes of the flags: position, then normal, texel and color.
fn vertex_attributes(flags: Graphic3dArrayFlags) -> Vec<Graphic3dAttribute> {
    use Graphic3dTypeOfAttribute as A;
    use Graphic3dTypeOfData as D;
    let optional = [
        (GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL, A::Norm, D::Vec3),
        (GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL, A::Uv, D::Vec2),
        (GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR, A::Color, D::Vec4ub),
    ];
    let mut attributes = vec![Graphic3dAttribute {
        id: A::Pos,
        data_type: D::Vec3,
    }];
    attributes.extend(
        optional
            .iter()
            .filter(|(flag, _, _)| flags & flag != 0)
            .map(|(_, id, data_type)| Graphic3dAttribute {
                id: *id,
                data_type: *data_type,
            }),
    );
    attributes
}

/// The slot of an attribute role in the vertex buffer.
fn slot_of(
    buffer: &dyn Graphic3dBufferSource,
    attrib: Graphic3dTypeOfAttribute,
) -> Graphic3dAttributeSlot {
    buffer.attribute_data(attrib).map_or_else(
        Graphic3dAttributeSlot::default,
        |(_, offset, stride)| Graphic3dAttributeSlot {
            offset: Some(offset),
            stride,
        },
    )
}

impl Graphic3dArrayOfPrimitives {
    /// An array without buffers of a type (before `init`).
    fn empty(kind: Graphic3dTypeOfPrimitiveArray) -> Self {
        Self {
            transient: StandardTransient::default(),
            indices: Handle::default(),
            attribs: Handle::default(),
            bounds: Handle::default(),
            position_stride: 0,
            normals: Graphic3dAttributeSlot::default(),
            texels: Graphic3dAttributeSlot::default(),
            colors: Graphic3dAttributeSlot::default(),
            kind,
        }
    }
    /// The protected constructor: `init` of an array of `kind` with room for vertices, bounds
    /// and edges. Allocation failures leave null buffers, as natively; an allocator refusal is
    /// the error. Example: `Graphic3dArrayOfPrimitives::new(kind, 4, 0, 6, flags)?`.
    pub fn new(
        kind: Graphic3dTypeOfPrimitiveArray,
        max_vertexs: i32,
        max_bounds: i32,
        max_edges: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<Self, Failure> {
        let mut array = Self::empty(kind);
        let allocator = Graphic3dBufferBase::default_allocator();
        array.attribs = vertex_buffer(&allocator, flags)?;
        if max_vertexs < 1 {
            return Ok(array);
        }
        if max_edges > 0 {
            let Some(indices) = index_buffer(&allocator, flags, max_vertexs, max_edges)? else {
                return Ok(array);
            };
            array.indices = indices;
        }
        if !array.init_vertices(max_vertexs, flags)? {
            (array.attribs, array.indices) = (Handle::default(), Handle::default());
            return Ok(array);
        }
        array.init_bounds(&allocator, max_bounds, flags)?;
        Ok(array)
    }
    /// Allocates and zeroes the vertex buffer and records the attribute slots.
    fn init_vertices(
        &mut self,
        max_vertexs: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<bool, Failure> {
        let Some(buffer) = self.attribs.get() else {
            return Ok(false);
        };
        if !buffer.init(max_vertexs, &vertex_attributes(flags))? {
            return Ok(false);
        }
        self.position_stride = slot_of(buffer, Graphic3dTypeOfAttribute::Pos).stride;
        (self.normals, self.texels, self.colors) = (
            slot_of(buffer, Graphic3dTypeOfAttribute::Norm),
            slot_of(buffer, Graphic3dTypeOfAttribute::Uv),
            slot_of(buffer, Graphic3dTypeOfAttribute::Color),
        );
        let used = (buffer.stride() as usize).wrapping_mul(buffer.nb_max_elements() as usize);
        let mut state = buffer.buffer_base().state.write();
        state
            .data
            .as_mut_slice()
            .iter_mut()
            .take(used)
            .for_each(|byte| *byte = 0);
        drop(state);
        if flags
            & (GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_MUTABLE | GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_DEINTERLEAVED)
            == 0
        {
            buffer.set_nb_elements(0);
        }
        Ok(true)
    }
    /// Allocates the bound buffer; a failure nullifies every buffer, as natively.
    fn init_bounds(
        &mut self,
        allocator: &BaseAllocator,
        max_bounds: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<(), Failure> {
        if max_bounds <= 0 {
            return Ok(());
        }
        let bounds = Graphic3dBoundBuffer::new(allocator.clone())?;
        if !bounds.init(max_bounds, flags & GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR != 0)? {
            (self.attribs, self.indices) = (Handle::default(), Handle::default());
            return Ok(());
        }
        bounds.set_nb_bounds(0);
        self.bounds = Handle::new(bounds);
        Ok(())
    }
}

/// The type, buffers and counts.
impl Graphic3dArrayOfPrimitives {
    /// The vertex buffer; `array.attributes()`.
    pub fn attributes(&self) -> Handle<dyn Graphic3dBufferSource> {
        self.attribs.clone()
    }
    /// The primitive type; `array.primitive_type()`.
    pub fn primitive_type(&self) -> Graphic3dTypeOfPrimitiveArray {
        self.kind
    }
    /// Whether vertices have normals; `array.has_vertex_normals()`.
    pub fn has_vertex_normals(&self) -> bool {
        self.normals.offset.is_some()
    }
    /// Whether vertices have colors; `array.has_vertex_colors()`.
    pub fn has_vertex_colors(&self) -> bool {
        self.colors.offset.is_some()
    }
    /// Whether vertices have texels; `array.has_vertex_texels()`.
    pub fn has_vertex_texels(&self) -> bool {
        self.texels.offset.is_some()
    }
    /// The vertex count in use (0 without vertex buffer, a null dereference natively);
    /// `array.vertex_number()`.
    pub fn vertex_number(&self) -> i32 {
        self.attribs.get().map_or(0, |buffer| buffer.nb_elements())
    }
    /// The allocated vertex count; `array.vertex_number_allocated()`.
    pub fn vertex_number_allocated(&self) -> i32 {
        self.attribs
            .get()
            .map_or(0, |buffer| buffer.nb_max_elements())
    }
    /// The index buffer; `array.indices()`.
    pub fn indices(&self) -> Handle<dyn Graphic3dIndexBufferSource> {
        self.indices.clone()
    }
    /// The bound buffer; `array.bounds()`.
    pub fn bounds(&self) -> Handle<Graphic3dBoundBuffer> {
        self.bounds.clone()
    }
    /// `CreateArray(type, maxVertexs, maxBounds, maxEdges, flags)`: the array class of a type
    /// (the adjacency types are plain arrays); null for `UNDEFINED`.
    /// Example: `Graphic3dArrayOfPrimitives::create_array_with_bounds(kind, 4, 1, 0, 0)?`.
    pub fn create_array_with_bounds(
        kind: Graphic3dTypeOfPrimitiveArray,
        max_vertexs: i32,
        max_bounds: i32,
        max_edges: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<Handle<Self>, Failure> {
        use Graphic3dTypeOfPrimitiveArray as T;
        let (bounds, edges) = match kind {
            T::Undefined => return Ok(Handle::default()),
            T::Points => (0, 0),
            T::Segments | T::Triangles | T::Quadrangles => (0, max_edges),
            T::Trianglestrips | T::Trianglefans | T::Quadranglestrips => (max_bounds, 0),
            _ => (max_bounds, max_edges),
        };
        super::array_classes::create_class(
            kind,
            Self::new(kind, max_vertexs, bounds, edges, flags)?,
        )
    }
    /// `CreateArray(type, maxVertexs, maxEdges, flags)`: without bounds; `Graphic3dArrayOfPrimitives::create_array(kind, 3, 0, 0)?`.
    pub fn create_array(
        kind: Graphic3dTypeOfPrimitiveArray,
        max_vertexs: i32,
        max_edges: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<Handle<Self>, Failure> {
        Self::create_array_with_bounds(kind, max_vertexs, 0, max_edges, flags)
    }
}
