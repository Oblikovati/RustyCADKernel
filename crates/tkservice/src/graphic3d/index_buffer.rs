//! `Graphic3d_IndexBuffer` (16- or 32-bit vertex indices) and `Graphic3d_MutableIndexBuffer`
//! (an index buffer that tracks the invalidated byte range).
use super::buffer::graphic3d_buffer_dump_json;
use super::{
    Graphic3dBufferBase, Graphic3dBufferMembers, Graphic3dBufferRange, Graphic3dBufferSource,
    Graphic3dBufferValue,
};
use crate::json_dump::{base_class, class_begin};
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use std::io;

/// An index buffer class (`Graphic3d_IndexBuffer` and the classes derived from it).
pub trait Graphic3dIndexBufferSource: Graphic3dBufferSource {}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dIndexBufferSource,
    "Graphic3d_IndexBuffer",
    dyn Graphic3dBufferSource
);

/// The members of `Graphic3d_IndexBuffer`.
pub trait Graphic3dIndexBufferMembers: Graphic3dIndexBufferSource {
    /// `Init<IndexType_t>`: `count` indices of `T` (u16 or u32 only, else false).
    /// Example: `indices.init_indices::<u16>(6)?`.
    fn init_indices<T: Graphic3dBufferValue>(&self, count: i32) -> Result<bool, Failure> {
        self.release()?;
        let mut state = self.buffer_base().state.write();
        state.stride = T::SIZE as i32;
        if T::SIZE != 2 && T::SIZE != 4 {
            return Ok(false);
        }
        (state.nb_elements, state.nb_attributes) = (count, 0);
        if count != 0 && !state.data.allocate(T::SIZE.wrapping_mul(count as usize))? {
            drop(state);
            self.release()?;
            return Ok(false);
        }
        Ok(true)
    }
    /// `InitInt32`: 32-bit indices; `indices.init_int32(6)?`.
    fn init_int32(&self, count: i32) -> Result<bool, Failure> {
        self.init_indices::<i32>(count)
    }
    /// The index at a position, widened to int; `None` past the bytes (undefined natively).
    /// Example: `indices.index(0)`.
    fn index(&self, position: i32) -> Option<i32> {
        if self.stride() == 2 {
            return self.value::<u16>(position).map(i32::from);
        }
        self.value::<u32>(position).map(|value| value as i32)
    }
    /// Sets the index at a position, narrowed to the index size; false past the bytes.
    /// Example: `indices.set_index(0, 5)`.
    fn set_index(&self, position: i32, value: i32) -> bool {
        if self.stride() == 2 {
            return self.set_value(position, value as u16);
        }
        self.set_value(position, value as u32)
    }
}
impl<T: Graphic3dIndexBufferSource + ?Sized> Graphic3dIndexBufferMembers for T {}

/// The `DumpJson` of `Graphic3d_IndexBuffer`: the class name, then the buffer dump.
fn index_buffer_dump_json(
    base: &Graphic3dBufferBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "Graphic3d_IndexBuffer")?;
    base_class(stream, depth, &|s, d| {
        graphic3d_buffer_dump_json(base, s, d)
    })
}

/// The native `Graphic3d_IndexBuffer`; `Graphic3dIndexBuffer::new(allocator)?`.
pub struct Graphic3dIndexBuffer {
    transient: StandardTransient,
    base: Graphic3dBufferBase,
}
impl Graphic3dBufferSource for Graphic3dIndexBuffer {
    fn buffer_base(&self) -> &Graphic3dBufferBase {
        &self.base
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        index_buffer_dump_json(&self.base, stream, depth)
    }
}
impl Graphic3dIndexBufferSource for Graphic3dIndexBuffer {}
rustycad_tkernel::impl_transient!(Graphic3dIndexBuffer, "Graphic3d_IndexBuffer", dyn Graphic3dIndexBufferSource, transient,
    interfaces: [dyn Graphic3dBufferSource, dyn Graphic3dIndexBufferSource]);

impl Graphic3dIndexBuffer {
    /// An empty index buffer over an allocator; `Graphic3dIndexBuffer::new(allocator)?`.
    pub fn new(allocator: BaseAllocator) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            base: Graphic3dBufferBase::new(allocator)?,
        })
    }
}

/// An index buffer whose changes are tracked as an invalidated byte range.
///
/// Example: `let indices = Graphic3dMutableIndexBuffer::new(allocator)?; indices.invalidate_indices(0, 2)?;`.
pub struct Graphic3dMutableIndexBuffer {
    transient: StandardTransient,
    base: Graphic3dBufferBase,
    invalidated_range: SharedState<Graphic3dBufferRange>,
}
impl Graphic3dBufferSource for Graphic3dMutableIndexBuffer {
    fn buffer_base(&self) -> &Graphic3dBufferBase {
        &self.base
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn invalidated_range(&self) -> Graphic3dBufferRange {
        *self.invalidated_range.read()
    }
    fn validate(&self) {
        self.invalidated_range.write().clear();
    }
    /// The whole data: `[0, size)` (the size narrowed to int, as natively).
    fn invalidate(&self) -> Result<(), Failure> {
        let size = self.base.state.read().data.size() as i32;
        self.invalidate_range(Graphic3dBufferRange::new(0, size));
        Ok(())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        index_buffer_dump_json(&self.base, stream, depth)
    }
}
impl Graphic3dIndexBufferSource for Graphic3dMutableIndexBuffer {}
rustycad_tkernel::impl_transient!(Graphic3dMutableIndexBuffer, "Graphic3d_MutableIndexBuffer", dyn Graphic3dIndexBufferSource, transient,
    interfaces: [dyn Graphic3dBufferSource, dyn Graphic3dIndexBufferSource]);

impl Graphic3dMutableIndexBuffer {
    /// An empty mutable index buffer; `Graphic3dMutableIndexBuffer::new(allocator)?`.
    pub fn new(allocator: BaseAllocator) -> Result<Self, Failure> {
        let base = Graphic3dBufferBase::new(allocator)?;
        Ok(Self {
            transient: StandardTransient::default(),
            base,
            invalidated_range: SharedState::new(Graphic3dBufferRange::default()),
        })
    }
    /// `Invalidate(lower, upper)`: the bytes of indices `lower..=upper`; `lower > upper` raises
    /// `Standard_OutOfRange`. Example: `indices.invalidate_indices(0, 2)?`.
    pub fn invalidate_indices(&self, lower: i32, upper: i32) -> Result<(), Failure> {
        if lower > upper {
            return Err(out_of_range(&format!(
                "Graphic3d_MutableIndexBuffer::Invalidate(); got lower {lower} > upper {upper}"
            )));
        }
        let stride = self.stride();
        self.invalidate_range(Graphic3dBufferRange::new(
            stride.wrapping_mul(lower),
            stride.wrapping_mul(upper - lower + 1),
        ));
        Ok(())
    }
    /// `invalidate`: unites a byte range with the invalidated range; `indices.invalidate_range(range)`.
    pub fn invalidate_range(&self, range: Graphic3dBufferRange) {
        self.invalidated_range.write().unite(&range);
    }
}
