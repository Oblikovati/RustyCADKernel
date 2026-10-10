//! `Select3D_BVHIndexBuffer`: a `Graphic3d_Buffer` of unsigned element indexes, each with a
//! patch size when patches are enabled.
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkservice::graphic3d::{
    Graphic3dBufferBase, Graphic3dBufferMembers as _, Graphic3dBufferSource,
};

/// The size of one index (`sizeof(unsigned int)`).
const INDEX_SIZE: usize = 4;

/// The BVH index buffer of a primitive array.
///
/// Example: `let buffer = Select3DBvhIndexBuffer::new(Graphic3dBufferBase::default_allocator())?; buffer.init(4, false)?;`.
pub struct Select3DBvhIndexBuffer {
    transient: StandardTransient,
    base: Graphic3dBufferBase,
    has_patches: SharedState<bool>,
}
rustycad_tkernel::impl_transient!(Select3DBvhIndexBuffer, "Select3D_BVHIndexBuffer", dyn Graphic3dBufferSource, transient,
    interfaces: [dyn Graphic3dBufferSource]);

impl Graphic3dBufferSource for Select3DBvhIndexBuffer {
    fn buffer_base(&self) -> &Graphic3dBufferBase {
        &self.base
    }
}

impl Select3DBvhIndexBuffer {
    /// An empty buffer over an allocator; `Select3DBvhIndexBuffer::new(allocator)?`.
    pub fn new(allocator: BaseAllocator) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            base: Graphic3dBufferBase::new(allocator)?,
            has_patches: SharedState::new(false),
        })
    }
    /// `HasPatches`; `buffer.has_patches()`.
    pub fn has_patches(&self) -> bool {
        *self.has_patches.read()
    }
    /// `Init`: `nb_elems` indexes (with their patch sizes when asked); false when the
    /// allocation fails. Example: `buffer.init(8, true)?`.
    pub fn init(&self, nb_elems: i32, has_patches: bool) -> Result<bool, Failure> {
        self.release()?;
        *self.has_patches.write() = has_patches;
        let stride = INDEX_SIZE * if has_patches { 2 } else { 1 };
        let mut state = self.base.state.write();
        (state.stride, state.nb_elements, state.nb_attributes) = (stride as i32, nb_elems, 0);
        if nb_elems != 0
            && !state
                .data
                .allocate(stride.wrapping_mul(nb_elems as usize))?
        {
            drop(state);
            self.release()?;
            return Ok(false);
        }
        Ok(true)
    }
    /// `Index`: the index at a position (0 past the bytes); `buffer.index(0)`.
    pub fn index(&self, index: i32) -> i32 {
        self.read_at::<u32>(self.element_offset(index)).unwrap_or(0) as i32
    }
    /// `PatchSize`: the patch size at a position, 1 without patches; `buffer.patch_size(0)`.
    pub fn patch_size(&self, index: i32) -> i32 {
        if !self.has_patches() {
            return 1;
        }
        self.read_at::<u32>(self.element_offset(index) + INDEX_SIZE)
            .unwrap_or(0) as i32
    }
    /// `SetIndex(index, value)`; `buffer.set_index(0, 5)`.
    pub fn set_index(&self, index: i32, value: i32) {
        self.write_at(self.element_offset(index), value as u32);
    }
    /// `SetIndex(index, value, patchSize)`; `buffer.set_index_patch(0, 5, 2)`.
    pub fn set_index_patch(&self, index: i32, value: i32, patch_size: i32) {
        let offset = self.element_offset(index);
        self.write_at(offset, value as u32);
        self.write_at(offset + INDEX_SIZE, patch_size as u32);
    }
}
