//! Scoped access to the actual Size field while the last item is destroyed in its own slot.
use super::DynamicArray;
use crate::standard::Failure;

struct EraseSizePhase<'a> {
    used: &'a mut usize,
    uncertain: &'a mut bool,
    previous_uncertainty: bool,
    completed: bool,
}
impl<'a> EraseSizePhase<'a> {
    fn begin(used: &'a mut usize, uncertain: &'a mut bool) -> Self {
        let previous_uncertainty = *uncertain;
        *uncertain = true;
        Self {
            used,
            uncertain,
            previous_uncertainty,
            completed: false,
        }
    }
    fn complete(mut self) {
        self.completed = true;
    }
}
impl Drop for EraseSizePhase<'_> {
    fn drop(&mut self) {
        *self.used -= 1;
        if self.completed {
            *self.uncertain = self.previous_uncertainty;
        }
    }
}
impl<T> DynamicArray<T> {
    /// Destroys the last item in its original slot while exposing its owner's actual Size field.
    /// `unsafe { array.erase_last_with_destruction(|slot, size| { assert!(*size > 0); slot.drop_in_place(); })? };`
    /// The scoped size reference borrows only metadata, independently of the element block.
    /// Empty arrays do not invoke the callback. Noncanonical initialized prefixes are rejected.
    /// # Safety
    /// The callback receives exclusive access to one live T removed from automatic ownership.
    /// It must destroy that value exactly once in place and must not free its allocation. No
    /// conflicting element or whole-owner reference may be created. Neither the slot nor metadata
    /// reference may be accessed after the callback returns or unwinds. For native parity the
    /// callback and destructor must not unwind: OCCT 3d097a0 destroyAt is noexcept. Rust unwinding
    /// decrements Size and leaves the slot unowned, preventing automatic double destruction; the
    /// callback remains responsible for cleanup if it panics before ending the value's lifetime.
    pub unsafe fn erase_last_with_destruction(
        &mut self,
        destroy: impl FnOnce(*mut T, &usize),
    ) -> Result<(), Failure> {
        if self.used == 0 {
            return Ok(());
        }
        self.validate_erasure_prefix()?;
        let index = self.used - 1;
        let block = &mut self.blocks[index / self.increment];
        let offset = index % self.increment;
        let phase = EraseSizePhase::begin(&mut self.used, &mut self.needs_tail_scan);
        // Validation proves this is the final live slot; removing ownership does not move bytes.
        let slot = unsafe { block.as_mut_ptr().add(offset) };
        unsafe { block.set_len(offset) };
        destroy(slot, phase.used);
        phase.complete();
        Ok(())
    }
    fn validate_erasure_prefix(&self) -> Result<(), Failure> {
        for (index, block) in self.blocks.iter().enumerate() {
            let expected = self
                .used
                .saturating_sub(index * self.increment)
                .min(self.increment);
            if block.len() != expected {
                let reason = format!(
                    "block {index} initialized length {}: expected canonical prefix length {expected}",
                    block.len()
                );
                return Err(Failure::with_type("Standard_DomainError", &reason, ""));
            }
        }
        Ok(())
    }
}
