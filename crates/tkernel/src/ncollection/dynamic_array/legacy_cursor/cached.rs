//! Native iterator pointers survive block ownership transfer; reloads still consult the old owner.
use super::{DynamicArray, DynamicArrayIterator, Failure};
mod capture;
mod navigation;

/// Captures real element pointers independently of the container's later block ownership.
/// `let cursor = DynamicArrayCachedIterator::from_shared(&array, 0)?; let pointer = cursor.pointer();`
/// Pointers do not keep storage alive. Dereferencing requires the current allocation owner to
/// retain initialized elements and requires the caller to preserve Rust aliasing rules.
/// Checked navigation takes the original owner explicitly; it never dereferences a stored owner.
pub struct DynamicArrayCachedIterator<T, const MUTABLE: bool = false> {
    position: DynamicArrayIterator<T, MUTABLE>,
    current: *mut T,
    block_end: *mut T,
}
impl<T, const M: bool> Copy for DynamicArrayCachedIterator<T, M> {}
impl<T, const M: bool> Clone for DynamicArrayCachedIterator<T, M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, const M: bool> Default for DynamicArrayCachedIterator<T, M> {
    fn default() -> Self {
        Self {
            position: DynamicArrayIterator::default(),
            current: std::ptr::null_mut(),
            block_end: std::ptr::null_mut(),
        }
    }
}
impl<T, const M: bool> DynamicArrayCachedIterator<T, M> {
    /// Checks the cached pointer, even after blocks move to another owner; `cursor.more()`.
    pub fn more(&self) -> bool {
        !self.current.is_null()
    }
    /// Returns the actual cached pointer, including null at end; `let pointer = cursor.pointer();`.
    /// Returning this address does not establish a live value or grant write access.
    pub fn pointer(&self) -> *const T {
        self.current.cast_const()
    }
    /// Reads the cached value, even when a different array now owns its block.
    /// `let value = unsafe { cursor.value() };`.
    /// # Safety
    /// The non-null pointer must still denote an initialized T in its original live allocation.
    /// Its current allocation owner must outlive the returned borrow. No incompatible mutation,
    /// destruction, allocation release or exclusive reference may overlap that borrow. Moving
    /// blocks does not renew pointer provenance invalidated by another conflicting access.
    pub unsafe fn value(&self) -> &T {
        assert!(
            self.more(),
            "cached pointer is null: expected a live initialized slot"
        );
        // The caller guarantees initialization, lifetime and compatible shared access.
        unsafe { &*self.current }
    }
    /// Copies pointer state into a read-only cursor without resolving the original owner again.
    /// `let constant = mutable.as_const_cursor();`.
    pub fn as_const_cursor(&self) -> DynamicArrayCachedIterator<T, false> {
        DynamicArrayCachedIterator {
            position: self.position.as_const_cursor(),
            current: self.current,
            block_end: self.block_end,
        }
    }
    /// Compares native owner identity and position, independently of cached pointer equality.
    /// `cursor.same_position(&other)` can be false when their pointers are equal after a move.
    pub fn same_position<const O: bool>(&self, other: &DynamicArrayCachedIterator<T, O>) -> bool {
        self.position == other.position
    }
}
impl<T> DynamicArrayCachedIterator<T, true> {
    /// Returns writable provenance captured from an exclusive raw-storage borrow.
    /// `let pointer = cursor.change_pointer();` still requires unsafe caller access.
    pub fn change_pointer(&self) -> *mut T {
        self.current
    }
    /// Mutates the cached value in its current owner's allocation.
    /// `*unsafe { cursor.change_value() } = replacement;`.
    /// # Safety
    /// The original pointer must retain writable provenance to one initialized live T. Its current
    /// allocation owner must outlive the exclusive borrow. No overlapping references or accesses
    /// through cursor copies may exist; no destruction, move-out or free may occur during the borrow.
    pub unsafe fn change_value(&mut self) -> &mut T {
        assert!(
            self.more(),
            "cached pointer is null: expected an exclusively accessible live slot"
        );
        // Mutable construction/reload obtains raw provenance directly from AllocVec::as_mut_ptr.
        unsafe { &mut *self.current }
    }
}
