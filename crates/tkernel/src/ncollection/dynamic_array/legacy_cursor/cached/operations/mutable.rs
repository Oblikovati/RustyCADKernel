use super::{DynamicArray, DynamicArrayCachedIterator, Failure, negate_displacement};
impl<T> DynamicArrayCachedIterator<T, true> {
    /// Returns the previous actual cached pointers after checked native increment.
    /// `let previous = cursor.postfix_advance_with(&mut source)?;` requires no payload cloning.
    pub fn postfix_advance_with(&mut self, owner: &mut DynamicArray<T>) -> Result<Self, Failure> {
        self.postfix_operation(|cursor| cursor.advance_with(owner).map(|_| ()))
    }
    /// Returns the previous cached state after checked native decrement.
    /// `let previous = cursor.postfix_retreat_with(&mut source)?;` preserves transferred pointers.
    pub fn postfix_retreat_with(&mut self, owner: &mut DynamicArray<T>) -> Result<Self, Failure> {
        self.postfix_operation(|cursor| cursor.retreat_with(owner).map(|_| ()))
    }
    /// Copies the cursor and reloads its displaced position from the original owner.
    /// `let copy = cursor.advanced_by_with(&mut source, 0)?;` reloads even a zero offset.
    pub fn advanced_by_with(
        &self,
        owner: &mut DynamicArray<T>,
        offset: isize,
    ) -> Result<Self, Failure> {
        self.copied_operation(|cursor| cursor.offset_with(owner, offset).map(|_| ()))
    }
    /// Applies subtraction after checking that native signed negation is representable.
    /// `cursor.subtract_with(&mut source, 1)?;` preserves state when checks fail.
    pub fn subtract_with(
        &mut self,
        owner: &mut DynamicArray<T>,
        offset: isize,
    ) -> Result<&mut Self, Failure> {
        let opposite = negate_displacement(offset)?;
        self.offset_with(owner, opposite)
    }
    /// Copies the cursor and subtracts displacement using the original owner's current storage.
    /// `let previous = cursor.retreated_by_with(&mut source, 1)?;` leaves this cache unchanged.
    pub fn retreated_by_with(
        &self,
        owner: &mut DynamicArray<T>,
        offset: isize,
    ) -> Result<Self, Failure> {
        self.copied_operation(|cursor| cursor.subtract_with(owner, offset).map(|_| ()))
    }
    /// Reloads a displaced value and borrows it from the explicitly supplied original owner.
    /// `let value = cursor.subscript_with(&mut source, 0)?;` checks the freshly captured live slot.
    /// The reference follows Rust borrowing rules; older raw cursors must retain valid provenance.
    pub fn subscript_with<'a>(
        &self,
        owner: &'a mut DynamicArray<T>,
        offset: isize,
    ) -> Result<&'a mut T, Failure> {
        let displaced = self.advanced_by_with(owner, offset)?;
        displaced.require_subscript_slot(offset)?;
        // Fresh capture proves a live slot in owner, whose borrow bounds this returned reference.
        Ok(unsafe { &mut *displaced.current })
    }
}
