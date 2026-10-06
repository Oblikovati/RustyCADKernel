//! Direct native base traversal retains its final within-block offset.
use super::SparseArrayBase;
use std::ffi::c_void;
/// Borrowed historical base iterator; `SparseArrayBaseIterator::new(&base)`.
#[derive(Clone, Copy, Default)]
pub struct SparseArrayBaseIterator<'a> {
    owner: Option<&'a SparseArrayBase>,
    block: usize,
    offset: usize,
    more: bool,
}
impl<'a> SparseArrayBaseIterator<'a> {
    /// Starts on the first constructed object; `SparseArrayBaseIterator::new(&base)`.
    pub fn new(owner: &'a SparseArrayBase) -> Self {
        let mut iterator = Self::default();
        iterator.initialize(owner);
        iterator
    }
    /// Rebinds to a base; `iterator.initialize(&base)`.
    pub fn initialize(&mut self, owner: &'a SparseArrayBase) {
        self.owner = Some(owner);
        self.restart();
    }
    /// Restarts native traversal; `iterator.restart()`.
    pub fn restart(&mut self) {
        self.more = false;
        self.offset = 0;
        self.block = 0;
        let Some(owner) = self.owner else {
            return;
        };
        while self.block < owner.block_slots() {
            if !owner.block_address(self.block).is_null() {
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
                self.offset = 0;
                return true;
            }
            self.block += 1;
        }
        false
    }
}
