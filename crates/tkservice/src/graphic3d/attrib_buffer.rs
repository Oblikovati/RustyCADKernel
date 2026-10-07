//! `Graphic3d_AttribBuffer`: a vertex buffer that can be mutable (with an invalidated byte
//! range) and non-interleaved (each attribute packed for all vertices).
use super::{
    Graphic3dAttribute, Graphic3dBufferBase, Graphic3dBufferMembers, Graphic3dBufferRange,
    Graphic3dBufferSource,
};
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkernel::standard::exceptions::{out_of_range, program_error};
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};

/// The fields of `Graphic3d_AttribBuffer`.
#[derive(Clone, Copy, Debug, Default)]
struct AttribBufferFields {
    invalidated_range: Graphic3dBufferRange,
    is_interleaved: bool,
    is_mutable: bool,
}

/// An attribute buffer; `Graphic3dAttribBuffer::new(allocator)?.is_interleaved()`.
pub struct Graphic3dAttribBuffer {
    transient: StandardTransient,
    base: Graphic3dBufferBase,
    fields: SharedState<AttribBufferFields>,
}
rustycad_tkernel::impl_transient!(Graphic3dAttribBuffer, "Graphic3d_AttribBuffer", dyn Graphic3dBufferSource, transient,
    interfaces: [dyn Graphic3dBufferSource]);

/// `Standard_OutOfRange` for a byte size past the 32-bit range of a mutable buffer.
fn beyond_32_bits(member: &str, size: usize) -> Failure {
    out_of_range(&format!(
        "Graphic3d_AttribBuffer::{member}, Mutable flag cannot be used for buffer exceeding 32-bit address space; got {size} bytes"
    ))
}

impl Graphic3dBufferSource for Graphic3dAttribBuffer {
    fn buffer_base(&self) -> &Graphic3dBufferBase {
        &self.base
    }
    fn is_interleaved(&self) -> bool {
        self.fields.read().is_interleaved
    }
    fn is_mutable(&self) -> bool {
        self.fields.read().is_mutable
    }
    fn invalidated_range(&self) -> Graphic3dBufferRange {
        self.fields.read().invalidated_range
    }
    fn validate(&self) {
        self.fields.write().invalidated_range.clear();
    }
    /// The whole data; past 32-bit sizes raises `Standard_OutOfRange`.
    fn invalidate(&self) -> Result<(), Failure> {
        let size = self.data_size();
        if size > i32::MAX as usize {
            return Err(beyond_32_bits("Invalidate()", size));
        }
        self.invalidate_range(Graphic3dBufferRange::new(0, size as i32))
    }
}

impl Graphic3dAttribBuffer {
    /// An interleaved, immutable, empty buffer; `Graphic3dAttribBuffer::new(allocator)?`.
    pub fn new(allocator: BaseAllocator) -> Result<Self, Failure> {
        let fields = AttribBufferFields {
            is_interleaved: true,
            ..Default::default()
        };
        Ok(Self {
            transient: StandardTransient::default(),
            base: Graphic3dBufferBase::new(allocator)?,
            fields: SharedState::new(fields),
        })
    }
    fn data_size(&self) -> usize {
        self.base.state.read().data.size()
    }
    /// `Init` with the 32-bit check of mutable buffers; `buffer.init_checked(4, &attributes)?`.
    pub fn init_checked(
        &self,
        count: i32,
        attributes: &[Graphic3dAttribute],
    ) -> Result<bool, Failure> {
        if !self.init(count, attributes)? {
            return Ok(false);
        }
        if self.data_size() > i32::MAX as usize && self.is_mutable() {
            return Err(beyond_32_bits("Init()", self.data_size()));
        }
        Ok(true)
    }
    /// Sets the mutable flag; past 32-bit sizes a mutable flag raises `Standard_OutOfRange`.
    /// Example: `buffer.set_mutable(true)?`.
    pub fn set_mutable(&self, is_mutable: bool) -> Result<(), Failure> {
        if self.data_size() > i32::MAX as usize && is_mutable {
            return Err(beyond_32_bits("SetMutable()", self.data_size()));
        }
        self.fields.write().is_mutable = is_mutable;
        Ok(())
    }
    /// Sets interleaving; an allocated buffer raises `Standard_ProgramError`. `buffer.set_interleaved(false)?`.
    pub fn set_interleaved(&self, is_interleaved: bool) -> Result<(), Failure> {
        let allocated = self.nb_max_elements();
        if allocated != 0 {
            return Err(program_error(&format!(
                "Graphic3d_AttribBuffer::SetInterleaved() should not be called for allocated buffer; got {allocated} elements"
            )));
        }
        self.fields.write().is_interleaved = is_interleaved;
        Ok(())
    }
    /// `invalidate`: unites a byte range; past 32-bit sizes raises `Standard_OutOfRange`.
    /// Example: `buffer.invalidate_range(Graphic3dBufferRange::new(0, 12))?`.
    pub fn invalidate_range(&self, range: Graphic3dBufferRange) -> Result<(), Failure> {
        let size = self.data_size();
        if size > i32::MAX as usize {
            return Err(out_of_range(&format!(
                "Graphic3d_Buffer::Invalidate() cannot be used for buffer exceeding 32-bit address space; got {size} bytes"
            )));
        }
        self.fields.write().invalidated_range.unite(&range);
        Ok(())
    }
}

/// The attribute-wise invalidation of `Graphic3d_AttribBuffer`.
impl Graphic3dAttribBuffer {
    /// The byte range of `lower..=upper` vertices of one packed attribute.
    fn packed_range(&self, attribute: i32, lower: i32, upper: i32) -> Graphic3dBufferRange {
        let max_vertices = self.nb_max_elements();
        let mut start = 0;
        for index in 0..attribute {
            start += self.attribute(index).map_or(0, |a| a.stride()) * max_vertices;
        }
        let stride = self.attribute(attribute).map_or(0, |a| a.stride());
        Graphic3dBufferRange::new(start + stride * lower, stride * (upper - lower + 1))
    }
    /// `Invalidate(theAttributeIndex)`: one attribute, or everything when interleaved; a bad
    /// index raises `Standard_OutOfRange`. Example: `buffer.invalidate_attribute(1)?`.
    pub fn invalidate_attribute(&self, attribute: i32) -> Result<(), Failure> {
        if attribute < 0 || attribute >= self.nb_attributes() {
            return Err(out_of_range(&format!(
                "Graphic3d_AttribBuffer::Invalidate(); got attribute {attribute}, expected [0, {})",
                self.nb_attributes()
            )));
        }
        if self.is_interleaved() {
            return self.invalidate();
        }
        self.invalidate_range(self.packed_range(attribute, 0, self.nb_max_elements() - 1))
    }
    /// `Invalidate(theAttributeIndex, lower, upper)`: vertices of one attribute; bad bounds
    /// raise `Standard_OutOfRange`. Example: `buffer.invalidate_attribute_vertices(0, 1, 2)?`.
    pub fn invalidate_attribute_vertices(
        &self,
        attribute: i32,
        lower: i32,
        upper: i32,
    ) -> Result<(), Failure> {
        if attribute < 0
            || attribute >= self.nb_attributes()
            || lower < 0
            || lower > upper
            || upper >= self.nb_max_elements()
        {
            return Err(out_of_range(&format!(
                "Graphic3d_AttribBuffer::Invalidate(); got attribute {attribute}, vertices [{lower}, {upper}]"
            )));
        }
        if self.is_interleaved() {
            return self.invalidate_vertices(lower, upper);
        }
        self.invalidate_range(self.packed_range(attribute, lower, upper))
    }
    /// `Invalidate(lower, upper)`: vertices of every attribute; bad bounds raise `Standard_OutOfRange`.
    /// Example: `buffer.invalidate_vertices(0, 3)?`.
    pub fn invalidate_vertices(&self, lower: i32, upper: i32) -> Result<(), Failure> {
        if lower < 0 || lower > upper || upper >= self.nb_max_elements() {
            return Err(out_of_range(&format!(
                "Graphic3d_AttribBuffer::Invalidate(); got vertices [{lower}, {upper}], expected within [0, {})",
                self.nb_max_elements()
            )));
        }
        if self.is_interleaved() {
            let stride = self.stride();
            return self.invalidate_range(Graphic3dBufferRange::new(
                stride * lower,
                stride * (upper - lower + 1),
            ));
        }
        (0..self.nb_attributes())
            .try_for_each(|attribute| self.invalidate_attribute_vertices(attribute, lower, upper))
    }
}
