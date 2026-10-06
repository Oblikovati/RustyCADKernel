use super::BaseListBasicIterator;
use crate::ncollection::base_list::{BasicListNode, BasicListNodeMut};

impl<N: BasicListNode> BaseListBasicIterator<N, false> {
    /// Dispatches native readonly operator*; `unsafe { cursor.dereference() }`.
    ///
    /// # Safety
    /// Current must carry provenance for an entire live N, readable for the returned borrow, without
    /// mutation through any other reference or copied cursor during that borrow.
    pub unsafe fn dereference(&self) -> &N::Item {
        // SAFETY: caller guarantees the link's concrete type and shared-reference validity.
        unsafe { (&*N::from_list_link(self.raw.current)).list_value() }
    }
    /// Uses native const Value dispatch; `unsafe { cursor.value() }`.
    ///
    /// # Safety
    /// Current must satisfy dereference's concrete-node and shared-borrow guarantees.
    pub unsafe fn value(&self) -> &N::Item {
        // SAFETY: the same node and reference contract is forwarded.
        unsafe { self.dereference() }
    }
    /// Applies native overloaded address-of after Value; `unsafe { cursor.arrow_with(address) }`.
    ///
    /// # Safety
    /// Current must satisfy dereference's live concrete-node and shared-borrow contract.
    /// The policy must preserve that borrow during dispatch. Returned raw-pointer validity,
    /// provenance and read permissions must be established separately before dereference.
    pub unsafe fn arrow_with(
        &self,
        address: impl FnOnce(&N::Item) -> *const N::Item,
    ) -> *const N::Item {
        // SAFETY: caller guarantees shared access while the address policy runs.
        address(unsafe { self.dereference() })
    }
    /// Returns native const operator-> identity; `unsafe { cursor.arrow() }`.
    ///
    /// # Safety
    /// Current must satisfy dereference's shared-node contract for this call.
    pub unsafe fn arrow(&self) -> *const N::Item {
        // SAFETY: caller guarantees a readable concrete node during dispatch.
        unsafe { self.arrow_with(std::ptr::from_ref) }
    }
}
impl<N: BasicListNodeMut> BaseListBasicIterator<N, true> {
    /// Dispatches native writable operator*; `*unsafe { cursor.dereference() } = item`.
    ///
    /// # Safety
    /// Current must carry provenance for an entire live N with writable access and exclusive
    /// access for the returned borrow, including against all copied raw cursors.
    pub unsafe fn dereference(&mut self) -> &mut N::Item {
        // SAFETY: caller supplies concrete projection and exclusive access guarantees.
        unsafe { (&mut *N::from_list_link(self.raw.current)).list_change_value() }
    }
    /// Calls ChangeValue, as native mutable Value does; `*unsafe { cursor.value() } = item`.
    ///
    /// # Safety
    /// Current must satisfy dereference's concrete-node and exclusive-borrow contract.
    pub unsafe fn value(&mut self) -> &mut N::Item {
        // SAFETY: caller provides the same exclusive-node guarantee.
        unsafe { self.dereference() }
    }
    /// Uses native ChangeValue's writable dispatch; `*unsafe { cursor.change_value() } = item`.
    ///
    /// # Safety
    /// Current must satisfy dereference's concrete-node and exclusive-borrow contract.
    pub unsafe fn change_value(&mut self) -> &mut N::Item {
        // SAFETY: caller provides the same exclusive-node guarantee.
        unsafe { self.dereference() }
    }
    /// Applies mutable overloaded address-of after ChangeValue; `unsafe { cursor.arrow_with(address) }`.
    ///
    /// # Safety
    /// Current must satisfy dereference's live concrete-node and exclusive-borrow contract.
    /// The policy must preserve that borrow during dispatch. Returned raw-pointer validity,
    /// provenance and write permissions must be established separately before dereference.
    pub unsafe fn arrow_with(
        &mut self,
        address: impl FnOnce(&mut N::Item) -> *mut N::Item,
    ) -> *mut N::Item {
        // SAFETY: caller guarantees exclusive access while the address policy runs.
        address(unsafe { self.dereference() })
    }
    /// Returns native writable operator-> identity; `unsafe { cursor.arrow() }`.
    ///
    /// # Safety
    /// Current must satisfy dereference's exclusive-node contract for this call.
    pub unsafe fn arrow(&mut self) -> *mut N::Item {
        // SAFETY: caller guarantees an exclusively accessible node during dispatch.
        unsafe { self.arrow_with(std::ptr::from_mut) }
    }
}
