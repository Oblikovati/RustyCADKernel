use super::{DynamicArray, DynamicArrayCachedIterator, Failure, capture};
use std::ptr;
#[derive(Clone, Copy)]
enum CachedStep {
    Advance,
    Retreat(usize),
    Offset(isize),
}
impl<T, const M: bool> DynamicArrayCachedIterator<T, M> {
    fn navigate(
        &mut self,
        owner: *const DynamicArray<T>,
        step: CachedStep,
        capture: impl FnOnce(usize, usize) -> Result<(*mut T, *mut T), Failure>,
    ) -> Result<&mut Self, Failure> {
        self.require_original_owner(owner)?;
        let mut next = *self;
        if next.apply_step(step)? {
            (next.current, next.block_end) = capture(next.position.index, next.position.end)?;
        }
        *self = next;
        Ok(self)
    }
    fn require_original_owner(&self, owner: *const DynamicArray<T>) -> Result<(), Failure> {
        if self.position.owner == owner {
            return Ok(());
        }
        let reason = format!(
            "navigation owner {owner:p}: expected original owner {:p}",
            self.position.owner
        );
        Err(Failure::with_type("Standard_DomainError", &reason, ""))
    }
    fn apply_step(&mut self, step: CachedStep) -> Result<bool, Failure> {
        match step {
            CachedStep::Advance => self.advance_cached(),
            CachedStep::Retreat(mask) => Ok(self.retreat_cached(mask)),
            CachedStep::Offset(offset) => self.offset_cached(offset),
        }
    }
    fn advance_cached(&mut self) -> Result<bool, Failure> {
        self.position.index = self.position.index.wrapping_add(1);
        if self.position.index >= self.position.end {
            self.current = ptr::null_mut();
            self.block_end = ptr::null_mut();
            return Ok(false);
        }
        if self.current.is_null() {
            return Err(Failure::with_type(
                "Standard_DomainError",
                "cached pointer is null: expected a live block before in-range increment",
                "",
            ));
        }
        self.current = self.current.wrapping_add(1);
        Ok(self.current == self.block_end)
    }
    fn retreat_cached(&mut self, mask: usize) -> bool {
        if self.current.is_null() {
            self.position.index = self.position.end.saturating_sub(1);
            return true;
        }
        self.position.index = self.position.index.wrapping_sub(1);
        if self.position.index < self.position.end && self.position.index & mask != mask {
            self.current = self.current.wrapping_sub(1);
            return false;
        }
        true
    }
    fn offset_cached(&mut self, offset: isize) -> Result<bool, Failure> {
        let index = self.position.index as isize;
        let Some(next) = index.checked_add(offset) else {
            let reason =
                format!("index {index}, offset {offset}: expected representable signed sum");
            return Err(Failure::with_type("Standard_OutOfRange", &reason, ""));
        };
        self.position.index = next as usize;
        Ok(true)
    }
}
impl<T> DynamicArrayCachedIterator<T, false> {
    /// Advances the cached pointer, consulting the original owner only at a block boundary.
    /// `cursor.advance_with(&source)?;` keeps transferred pointers until native reload phases.
    pub fn advance_with(&mut self, owner: &DynamicArray<T>) -> Result<&mut Self, Failure> {
        self.navigate(ptr::from_ref(owner), CachedStep::Advance, |index, end| {
            capture::capture_shared(owner, index, end)
        })
    }
    /// Retreats using the original owner's current mask and native reload decisions.
    /// `cursor.retreat_with(&source)?;` rejects absent initialized slots on reload.
    pub fn retreat_with(&mut self, owner: &DynamicArray<T>) -> Result<&mut Self, Failure> {
        let step = CachedStep::Retreat(owner.increment.wrapping_sub(1));
        self.navigate(ptr::from_ref(owner), step, |index, end| {
            capture::capture_shared(owner, index, end)
        })
    }
    /// Repositions and reloads from the original owner even when offset is zero.
    /// `cursor.offset_with(&source, 0)?;` can change a transferred cached pointer.
    pub fn offset_with(
        &mut self,
        owner: &DynamicArray<T>,
        offset: isize,
    ) -> Result<&mut Self, Failure> {
        self.navigate(
            ptr::from_ref(owner),
            CachedStep::Offset(offset),
            |index, end| capture::capture_shared(owner, index, end),
        )
    }
}
impl<T> DynamicArrayCachedIterator<T, true> {
    /// Advances while obtaining any reloaded writable pointer from exclusive raw storage access.
    /// `cursor.advance_with(&mut source)?;` never dereferences the stored owner address.
    pub fn advance_with(&mut self, owner: &mut DynamicArray<T>) -> Result<&mut Self, Failure> {
        self.navigate(ptr::from_ref(owner), CachedStep::Advance, |index, end| {
            capture::capture_mutable(owner, index, end)
        })
    }
    /// Retreats with the original owner's metadata and exclusive provenance for reloads.
    /// `cursor.retreat_with(&mut source)?;` leaves the cursor unchanged on invalid reload.
    pub fn retreat_with(&mut self, owner: &mut DynamicArray<T>) -> Result<&mut Self, Failure> {
        let step = CachedStep::Retreat(owner.increment.wrapping_sub(1));
        self.navigate(ptr::from_ref(owner), step, |index, end| {
            capture::capture_mutable(owner, index, end)
        })
    }
    /// Applies native signed displacement and reloads writable storage from the original owner.
    /// `cursor.offset_with(&mut source, 0)?;` rejects signed overflow before changing state.
    pub fn offset_with(
        &mut self,
        owner: &mut DynamicArray<T>,
        offset: isize,
    ) -> Result<&mut Self, Failure> {
        self.navigate(
            ptr::from_ref(owner),
            CachedStep::Offset(offset),
            |index, end| capture::capture_mutable(owner, index, end),
        )
    }
}
