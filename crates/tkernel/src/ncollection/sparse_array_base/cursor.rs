//! Direct native base traversal retains its final within-block offset.
use super::{SparseArrayBase, SparseArrayBlock};
use std::ffi::c_void;
/// Borrowed historical base iterator; `SparseArrayBaseIterator::new(&base)`.
#[derive(Clone, Copy, Default)]
pub struct SparseArrayBaseIterator<'a> {
    owner: Option<&'a SparseArrayBase>,
    block: usize,
    offset: usize,
    more: bool,
    cached_block: Option<SparseArrayBlock>,
}
impl<'a> SparseArrayBaseIterator<'a> {
    /// Starts on the first constructed object; `SparseArrayBaseIterator::new(&base)`.
    pub fn new(owner: &'a SparseArrayBase) -> Self {
        Self::from_optional(Some(owner))
    }
    /// Constructs the native nullable-source cursor; `SparseArrayBaseIterator::from_optional(None)`.
    pub fn from_optional(owner: Option<&'a SparseArrayBase>) -> Self {
        let mut iterator = Self::default();
        iterator.initialize_optional(owner);
        iterator
    }
    /// Rebinds to a base; `iterator.initialize(&base)`.
    pub fn initialize(&mut self, owner: &'a SparseArrayBase) {
        self.initialize_optional(Some(owner));
    }
    /// Rebinds or detaches like native init; `iterator.initialize_optional(None)`.
    pub fn initialize_optional(&mut self, owner: Option<&'a SparseArrayBase>) {
        self.owner = owner;
        self.restart();
    }
    /// Restarts native traversal; `iterator.restart()`.
    pub fn restart(&mut self) {
        self.more = false;
        let Some(owner) = self.owner else {
            return;
        };
        self.offset = 0;
        self.block = 0;
        while self.block < owner.block_slots() {
            if !owner.block_address(self.block).is_null() {
                self.cache_block(owner);
                self.more = true;
                self.seek();
                return;
            }
            self.block += 1;
        }
    }
    /// Reports a live object; `iterator.more()`.
    pub fn more(&self) -> bool {
        self.more
    }
    /// Returns native Index, zero when detached; `iterator.index()`.
    pub fn index(&self) -> usize {
        self.owner
            .map_or(0, |owner| self.block * owner.block_size() + self.offset)
    }
    /// Advances without changing an already exhausted iterator; `iterator.next()`.
    pub fn next(&mut self) {
        if self.more {
            self.offset += 1;
            self.seek();
        }
    }
    /// Returns a raw current item address; `iterator.value()`.
    pub fn value(&self) -> Option<*mut c_void> {
        if !self.more {
            return None;
        }
        self.owner
            .and_then(|owner| owner.get_value(self.index()).ok())
    }
    /// Returns native protected value() even after exhaustion; `unsafe { cursor.native_value_address() }`.
    /// Detached cursors or cursors without a cached block return None instead of native undefined pointer access.
    /// # Safety
    /// The cached block must remain allocated and the current owner's stride/offset must address
    /// that allocation. The returned slot can be unoccupied; dereference requires a live object.
    pub unsafe fn native_value_address(&self) -> Option<*mut c_void> {
        let owner = self.owner?;
        let block = self.cached_block?;
        // OCCT 3d097a retains myBlock after exhaustion and empty/null init; More does not guard value().
        Some(unsafe { owner.get_item(block, self.offset) })
    }
    fn cache_block(&mut self, owner: &SparseArrayBase) {
        let address = owner.block_address(self.block);
        self.cached_block = Some(unsafe { owner.get_block(address) });
    }
    fn seek(&mut self) {
        let Some(owner) = self.owner else {
            return;
        };
        loop {
            if self.offset >= owner.block_size() && !self.next_block(owner) {
                self.more = false;
                return;
            }
            if owner.has_value(self.index()) {
                return;
            }
            self.offset += 1;
        }
    }
    fn next_block(&mut self, owner: &SparseArrayBase) -> bool {
        self.block += 1;
        while self.block < owner.block_slots() {
            if !owner.block_address(self.block).is_null() {
                self.cache_block(owner);
                self.offset = 0;
                return true;
            }
            self.block += 1;
        }
        false
    }
}
