//! Native Clear can stop with dead slots; keep those bytes outside all initialized T containers.
use super::{AllocVec, BaseAllocator, DynamicArray};
use crate::{ncollection::raw_element_lifetime::construct_in_slot, standard::Failure};
use std::ops::Range;
mod admission;
mod raw_block;
use raw_block::ClearRawBlock;
/// Owns Clear interruption without exposing an invalid ordinary array.
/// `let state = array.into_clear_state();` transfers payloads to layout-preserving raw owners.
/// Dropping this state deliberately leaks payload allocations; that containment fallback has
/// no native cleanup-parity claim. Complete Clear or repair ended slots before extraction.
/// Original payload addresses are retained. Receiver addresses and descriptor contents are not:
/// the retained descriptor allocation contains empty placeholders during this explicit protocol.
pub struct DynamicArrayClearState<T> {
    backbone: DynamicArray<T>,
    raw_blocks: Vec<ClearRawBlock<T>>,
    used: usize,
    ended: usize,
    repaired: usize,
    completed: bool,
}
impl<T> DynamicArrayClearState<T> {
    fn take_array(mut array: DynamicArray<T>) -> Self {
        let used = array.used;
        let mut raw_blocks = Vec::with_capacity(array.blocks.len());
        for block in &mut array.blocks {
            raw_blocks.push(ClearRawBlock::take_from(block));
        }
        array.used = 0;
        Self {
            backbone: array,
            raw_blocks,
            used,
            ended: 0,
            repaired: 0,
            completed: false,
        }
    }
    /// Reads the protocol's native logical size, including dead slots; `state.size()`.
    pub fn size(&self) -> usize {
        self.used
    }
    /// Reads retained payload block count; `assert_eq!(state.block_count(), 3);`.
    pub fn block_count(&self) -> usize {
        self.raw_blocks.len()
    }
    /// Reports exactly the slots needing reconstruction; `assert_eq!(state.dead_slots(), 0..2);`.
    pub fn dead_slots(&self) -> Range<usize> {
        self.repaired..self.ended
    }
    /// Borrows one proven live logical slot without constructing any whole initialized slice.
    /// `assert!(state.live_value(0).is_none());` after slot0 destruction.
    pub fn live_value(&self, index: usize) -> Option<&T> {
        if index >= self.used || self.dead_slots().contains(&index) {
            return None;
        }
        let (block, offset) = self.backbone.element_coordinates(index);
        let storage = self.raw_blocks.get(block)?;
        if offset >= storage.registered_length() {
            return None;
        }
        // The state predicate and admitted prefix jointly prove this single T remains live.
        Some(unsafe { &*storage.pointer(offset) })
    }
    /// Executes native destruction/free phases; `unsafe { state.clear_with_destruction(false, |p, _| p.drop_in_place()) };`.
    /// Panics propagate unchanged with progress retained. Repair before starting another pass.
    /// # Safety
    /// Each callback has exclusive access to its supplied live slot and must end that lifetime
    /// exactly once in place, including on unwind. Calling drop_in_place meets this requirement.
    /// Do not free its allocation, retain slot/metadata references, or create a conflicting owner
    /// borrow. No other reference may observe a lifetime ended here. The Size reference borrows
    /// only this state's logical metadata. Reentrant original-receiver/descriptor observers are
    /// unsupported. Allocator release must not unwind. Zero stride intentionally invokes no
    /// destructors and abandons their cleanup obligations, preserving pinned OCCT 3d097a0 Clear.
    pub unsafe fn clear_with_destruction(
        &mut self,
        release: bool,
        mut destroy: impl FnMut(*mut T, &usize),
    ) {
        assert!(
            self.dead_slots().is_empty(),
            "Clear dead slots {:?}: expected complete repair before another pass",
            self.dead_slots()
        );
        self.ended = 0;
        self.repaired = 0;
        if self.backbone.increment != 0 {
            self.destroy_registered(&mut destroy);
        }
        self.finish_clear(release);
    }
    fn destroy_registered(&mut self, destroy: &mut impl FnMut(*mut T, &usize)) {
        while self.ended < self.used {
            let (block, offset) = self.backbone.element_coordinates(self.ended);
            let slot = self.raw_blocks[block].mutable_pointer(offset);
            // The callback contract ends the failed destructor's lifetime too.
            self.ended += 1;
            destroy(slot, &self.used);
        }
    }
    fn finish_clear(&mut self, release: bool) {
        if release {
            for block in self.raw_blocks.drain(..) {
                block.release_empty();
            }
            self.backbone.blocks.clear(true);
        }
        self.used = 0;
        self.ended = 0;
        self.repaired = 0;
        self.completed = true;
    }
    /// Reconstructs only dead slots; `state.repair_destroyed_with(|index| index as i32)`.
    /// Constructor panic preserves completed writes and the remaining hole range. It propagates
    /// its original payload unchanged. The new T is fully constructed before any slot write.
    pub fn repair_destroyed_with(&mut self, mut construct: impl FnMut(usize) -> T) {
        while self.repaired < self.ended {
            let index = self.repaired;
            let (block, offset) = self.backbone.element_coordinates(index);
            let slot = self.raw_blocks[block].mutable_pointer(offset);
            unsafe { construct_in_slot(slot, || construct(index)) };
            self.repaired += 1;
        }
    }
    /// Restores an ordinary array only when all holes are repaired, or Clear completed.
    /// `let array = state.try_into_array();` returns the unchanged state while holes remain.
    pub fn try_into_array(mut self) -> Result<DynamicArray<T>, Self> {
        if !self.dead_slots().is_empty() {
            return Err(self);
        }
        self.restore_typed_blocks();
        self.backbone.used = self.used;
        self.backbone.needs_tail_scan = !self.completed && self.backbone.increment == 0;
        Ok(self.backbone)
    }
    fn restore_typed_blocks(&mut self) {
        for (target, raw) in self
            .backbone
            .blocks
            .iter_mut()
            .zip(self.raw_blocks.drain(..))
        {
            let length = if self.completed {
                0
            } else {
                raw.registered_length()
            };
            *target = raw.into_initialized(length);
        }
    }
}
#[cfg(test)]
mod tests;
