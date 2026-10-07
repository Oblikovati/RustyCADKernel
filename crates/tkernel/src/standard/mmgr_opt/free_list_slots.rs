//! Canonical pointer-width head slots; retired arrays retain lifetime but never drive traversal.
use super::address_space::WORD;
use crate::standard::{
    Failure, exceptions,
    memory::{self, StandardMemory},
};

#[derive(Default)]
pub(super) struct NativeFreeListSlots {
    active: Option<StandardMemory>,
    retired: Vec<StandardMemory>,
    maximum: usize,
}

impl NativeFreeListSlots {
    pub(super) fn replace_with_zeroed(&mut self, maximum: usize) {
        let replacement = allocate_slots(maximum);
        if let Some(previous) = self.active.take() {
            // Native Initialize loses this pointer; retain ownership without retaining authority.
            self.retired.push(previous);
        }
        self.maximum = maximum;
        self.active = replacement;
    }

    pub(super) fn head(&self, index: usize) -> Result<usize, Failure> {
        let offset = self.slot_offset(index)?;
        let bytes = self.live_bytes()?;
        Ok(decode_slot(&bytes[offset..offset + WORD]))
    }

    pub(super) fn set_head(&mut self, index: usize, address: usize) -> Result<(), Failure> {
        let offset = self.slot_offset(index)?;
        self.live_bytes_mut()?[offset..offset + WORD].copy_from_slice(&address.to_ne_bytes());
        Ok(())
    }

    pub(super) fn validate_index(&self, index: usize) -> Result<(), Failure> {
        self.slot_offset(index)?;
        Ok(())
    }

    pub(super) fn heads(&self) -> Result<impl Iterator<Item = (usize, usize)> + '_, Failure> {
        let bytes = self.live_bytes()?;
        Ok(bytes
            .as_chunks::<WORD>()
            .0
            .iter()
            .enumerate()
            .map(|(index, slot)| (index, decode_slot(slot))))
    }

    pub(super) fn clear_through(&mut self, maximum: usize) -> Result<(), Failure> {
        for slot in self
            .live_bytes_mut()?
            .as_chunks_mut::<WORD>()
            .0
            .iter_mut()
            .take(maximum.saturating_add(1))
        {
            slot.fill(0);
        }
        Ok(())
    }

    fn slot_offset(&self, index: usize) -> Result<usize, Failure> {
        let count = self.live_bytes()?.len() / WORD;
        if index >= count {
            return Err(exceptions::program_error(&format!(
                "Standard_MMgrOpt: slot index {index}; expected index below {count}"
            )));
        }
        Ok(index * WORD)
    }

    fn live_bytes(&self) -> Result<&[u8], Failure> {
        self.active
            .as_ref()
            .map(StandardMemory::as_slice)
            .ok_or_else(|| missing_table(self.maximum))
    }

    fn live_bytes_mut(&mut self) -> Result<&mut [u8], Failure> {
        self.active
            .as_mut()
            .map(StandardMemory::as_mut_slice)
            .ok_or_else(|| missing_table(self.maximum))
    }

    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        // A refused table allocation is a distinct invalid state, never an empty sparse fallback.
        self.heads()
            .is_ok_and(|mut slots| slots.all(|(_, address)| address == 0))
    }
}

fn allocate_slots(maximum: usize) -> Option<StandardMemory> {
    let bytes = maximum.checked_add(1)?.checked_mul(WORD)?;
    memory::allocate(bytes).ok()
}

fn decode_slot(bytes: &[u8]) -> usize {
    let word = bytes.try_into().unwrap_or_else(|_| {
        panic!(
            "Standard_MMgrOpt: slot width {}; expected {WORD}",
            bytes.len()
        )
    });
    usize::from_ne_bytes(word)
}

fn missing_table(maximum: usize) -> Failure {
    exceptions::program_error(&format!(
        "Standard_MMgrOpt: null free-list table for maximum index {maximum}; expected allocated word slots"
    ))
}

#[cfg(test)]
mod tests;
