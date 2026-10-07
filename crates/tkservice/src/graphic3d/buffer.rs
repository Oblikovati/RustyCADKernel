//! `Graphic3d_Buffer`: the vertex attribute buffer of primitive arrays (interleaved by
//! default), with `Graphic3d_Attribute` and its type enumerations.
//!
//! The class has virtual members, so it is the trait [`Graphic3dBufferSource`] over the
//! embedded [`Graphic3dBufferBase`] (visualization-design.md section 1). The bytes keep the
//! native layout bit for bit; the attribute table that the native class stores after the data
//! bytes of the same allocation is a separate list here, invisible to the byte layout.
use super::Graphic3dBufferRange;
use crate::json_dump::{base_class, class_begin, field_integer};
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient, Transient};
use std::io;
use std::sync::OnceLock;

/// The role of a vertex attribute (`Graphic3d_TypeOfAttribute`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dTypeOfAttribute {
    /// `Graphic3d_TOA_POS`: vertex position.
    Pos = 0,
    /// `Graphic3d_TOA_NORM`: normal.
    Norm = 1,
    /// `Graphic3d_TOA_UV`: texture coordinates.
    Uv = 2,
    /// `Graphic3d_TOA_COLOR`: per-vertex color.
    Color = 3,
    /// `Graphic3d_TOA_CUSTOM`: custom attributes.
    Custom = 4,
}

/// The data type of a vertex attribute (`Graphic3d_TypeOfData`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dTypeOfData {
    /// `Graphic3d_TOD_USHORT`: unsigned 16-bit integer.
    Ushort = 0,
    /// `Graphic3d_TOD_UINT`: unsigned 32-bit integer.
    Uint = 1,
    /// `Graphic3d_TOD_VEC2`: 2-components float vector.
    Vec2 = 2,
    /// `Graphic3d_TOD_VEC3`: 3-components float vector.
    Vec3 = 3,
    /// `Graphic3d_TOD_VEC4`: 4-components float vector.
    Vec4 = 4,
    /// `Graphic3d_TOD_VEC4UB`: 4-components unsigned byte vector.
    Vec4ub = 5,
    /// `Graphic3d_TOD_FLOAT`: float value.
    Float = 6,
}

/// A vertex attribute: its role and data type (`Graphic3d_Attribute`).
///
/// Example: `Graphic3dAttribute { id: Graphic3dTypeOfAttribute::Pos, data_type: Graphic3dTypeOfData::Vec3 }.stride() == 12`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Graphic3dAttribute {
    /// `Id`.
    pub id: Graphic3dTypeOfAttribute,
    /// `DataType`.
    pub data_type: Graphic3dTypeOfData,
}

impl Graphic3dAttribute {
    /// The byte size of this attribute; `attribute.stride()`.
    pub fn stride(&self) -> i32 {
        Self::stride_of(self.data_type)
    }
    /// The byte size of a data type; `Graphic3dAttribute::stride_of(Graphic3dTypeOfData::Vec4ub) == 4`.
    pub fn stride_of(data_type: Graphic3dTypeOfData) -> i32 {
        match data_type {
            Graphic3dTypeOfData::Ushort => 2,
            Graphic3dTypeOfData::Uint
            | Graphic3dTypeOfData::Float
            | Graphic3dTypeOfData::Vec4ub => 4,
            Graphic3dTypeOfData::Vec2 => 8,
            Graphic3dTypeOfData::Vec3 => 12,
            Graphic3dTypeOfData::Vec4 => 16,
        }
    }
}

/// The fields of `Graphic3d_Buffer`: the bytes and their layout.
pub struct Graphic3dBufferState {
    /// The `NCollection_Buffer` part: the data bytes (`mySize` is their count).
    pub data: Buffer,
    /// `Stride`: the distance between two vertices of an interleaved array.
    pub stride: i32,
    /// `NbElements`: the number of elements in use.
    pub nb_elements: i32,
    /// `NbAttributes`: the number of vertex attributes.
    pub nb_attributes: i32,
    /// The attribute table (stored after the data bytes natively).
    pub attributes: Vec<Graphic3dAttribute>,
}

/// The embedded state of every buffer class.
pub struct Graphic3dBufferBase {
    /// The buffer fields, shared by every handle to the buffer.
    pub state: SharedState<Graphic3dBufferState>,
}

impl Graphic3dBufferBase {
    /// `Graphic3d_Buffer(theAlloc)`: an empty buffer over an allocator. The native
    /// `NCollection_Buffer` constructor allocates zero bytes, which an accumulating allocator
    /// refuses with `Standard_ProgramError`. Example:
    /// `Graphic3dBufferBase::new(Graphic3dBufferBase::default_allocator())?`.
    pub fn new(allocator: BaseAllocator) -> Result<Self, Failure> {
        let data = Buffer::new(Some(allocator), 0)?;
        let state = Graphic3dBufferState {
            data,
            stride: 0,
            nb_elements: 0,
            nb_attributes: 0,
            attributes: Vec::new(),
        };
        Ok(Self {
            state: SharedState::new(state),
        })
    }
    /// `DefaultAllocator`: the shared 16-byte aligned allocator; `Graphic3dBufferBase::default_allocator()`.
    pub fn default_allocator() -> BaseAllocator {
        static ALLOCATOR: OnceLock<BaseAllocator> = OnceLock::new();
        ALLOCATOR.get_or_init(|| BaseAllocator::aligned(16)).clone()
    }
}

/// A buffer class; implementers return their embedded base. The provided members are the
/// native virtuals and their bodies.
pub trait Graphic3dBufferSource: Transient {
    /// The embedded base; `buffer.buffer_base().state.read().stride`.
    fn buffer_base(&self) -> &Graphic3dBufferBase;
    /// Whether the attributes are interleaved (true); `buffer.is_interleaved()`.
    fn is_interleaved(&self) -> bool {
        true
    }
    /// Whether the buffer can be invalidated (false); `buffer.is_mutable()`.
    fn is_mutable(&self) -> bool {
        false
    }
    /// The invalidated byte range (empty); `buffer.invalidated_range()`.
    fn invalidated_range(&self) -> Graphic3dBufferRange {
        Graphic3dBufferRange::default()
    }
    /// Clears the invalidated range (nothing here); `buffer.validate()`.
    fn validate(&self) {}
    /// Invalidates the whole buffer (nothing here); `buffer.invalidate()?`.
    fn invalidate(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `DumpJson`: the class name, the `NCollection_Buffer` dump and the layout counters.
    /// Example: `buffer.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        graphic3d_buffer_dump_json(self.buffer_base(), stream, depth)
    }
}
rustycad_tkernel::impl_transient_interface!(dyn Graphic3dBufferSource, "Graphic3d_Buffer", Buffer);

/// The body of `Graphic3d_Buffer::DumpJson`; `graphic3d_buffer_dump_json(base, &mut stream, -1)?`.
pub fn graphic3d_buffer_dump_json(
    base: &Graphic3dBufferBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "Graphic3d_Buffer")?;
    let state = base.state.read();
    base_class(stream, depth, &|s, d| state.data.dump_json(s, d))?;
    field_integer(stream, "Stride", state.stride)?;
    field_integer(stream, "NbElements", state.nb_elements)?;
    field_integer(stream, "NbAttributes", state.nb_attributes)
}

/// The native `Graphic3d_Buffer`; `Graphic3dBuffer::new(Graphic3dBufferBase::default_allocator())?`.
pub struct Graphic3dBuffer {
    transient: StandardTransient,
    base: Graphic3dBufferBase,
}
impl Graphic3dBufferSource for Graphic3dBuffer {
    fn buffer_base(&self) -> &Graphic3dBufferBase {
        &self.base
    }
}
rustycad_tkernel::impl_transient!(Graphic3dBuffer, "Graphic3d_Buffer", dyn Graphic3dBufferSource, transient,
    interfaces: [dyn Graphic3dBufferSource]);

impl Graphic3dBuffer {
    /// An empty buffer over an allocator; `Graphic3dBuffer::new(allocator)?`.
    pub fn new(allocator: BaseAllocator) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            base: Graphic3dBufferBase::new(allocator)?,
        })
    }
}
