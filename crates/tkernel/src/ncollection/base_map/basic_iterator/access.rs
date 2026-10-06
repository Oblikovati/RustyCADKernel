#![allow(unsafe_code)]
use super::{BaseMapBasicIterator, BasicMapNode};
use crate::ncollection::base_map::BasicMapNodeMut;

impl<N: BasicMapNode, const M: bool, const K: bool, const O: bool>
    BaseMapBasicIterator<'_, N, M, K, O>
{
    unsafe fn readonly_value(&self) -> &N::Value {
        // SAFETY: the public adapter forwards node lifetime and shared-reference guarantees.
        unsafe { (&*self.node).map_value() }
    }
    unsafe fn node_key(&self) -> &N::Key {
        // SAFETY: the public key adapter forwards the same readable-node contract.
        unsafe { (&*self.node).map_key() }
    }
}

impl<N: BasicMapNode, const M: bool, const O: bool> BaseMapBasicIterator<'_, N, M, false, O> {
    /// Reads native mapped-node `Key()`; `unsafe { cursor.key() }`.
    ///
    /// # Safety
    /// The node must be nonnull, live and readable, with no mutation of its key while borrowed.
    pub unsafe fn key(&self) -> &N::Key {
        // SAFETY: caller guarantees the node and key remain readable.
        unsafe { self.node_key() }
    }
}

impl<N: BasicMapNode, const M: bool> BaseMapBasicIterator<'_, N, M, true, false> {
    /// Reads native unordered key-only `Key()` through node `Value()`; `unsafe { cursor.key() }`.
    ///
    /// # Safety
    /// The node must satisfy `value`'s readable-node and reference-lifetime contract.
    pub unsafe fn key(&self) -> &N::Value {
        // SAFETY: native unordered key-only nodes expose their key through Value.
        unsafe { self.readonly_value() }
    }
}

impl<N: BasicMapNode, const M: bool> BaseMapBasicIterator<'_, N, M, true, true> {
    /// Reads native ordered key-only `Key()` through node `Key()`; `unsafe { cursor.key() }`.
    ///
    /// # Safety
    /// The node must be nonnull, live and readable, with no mutation of its key while borrowed.
    pub unsafe fn key(&self) -> &N::Key {
        // SAFETY: unlike unordered key-only nodes, ordered nodes have a distinct Key member.
        unsafe { self.node_key() }
    }
}

impl<N: BasicMapNodeMut, const O: bool> BaseMapBasicIterator<'_, N, true, false, O> {
    /// Borrows the native mutable mapped dereference; `*unsafe { cursor.change_value() } = value;`.
    ///
    /// # Safety
    /// The node must have writable provenance and stay live and aligned. No other reference
    /// may access this value during the returned borrow, including references from copied cursors.
    pub unsafe fn change_value(&mut self) -> &mut N::Value {
        // SAFETY: the caller guarantees exclusive access to the live node's mapped value.
        unsafe { (&mut *self.node).map_value_mut() }
    }
    /// Returns native mutable arrow identity; `unsafe { cursor.change_value_ptr() }`.
    ///
    /// # Safety
    /// The current node must satisfy `change_value`'s exclusive-node contract for this call.
    pub unsafe fn change_value_ptr(&mut self) -> *mut N::Value {
        // SAFETY: reference exists only through conversion to a raw pointer.
        std::ptr::from_mut(unsafe { self.change_value() })
    }
}

macro_rules! readonly_access {
    () => {
        /// Reads native const/key-only dereference through Value; `unsafe { cursor.value() }`.
        ///
        /// # Safety
        /// The current node must be nonnull, live and readable, with no mutation of its value
        /// for the lifetime of the returned reference, including through copied cursors.
        pub unsafe fn value(&self) -> &N::Value {
            // SAFETY: the caller guarantees the same readable-node contract.
            unsafe { self.readonly_value() }
        }
        /// Returns native const arrow identity; `unsafe { cursor.value_ptr() }`.
        ///
        /// # Safety
        /// The current node must satisfy value's readable-node contract for this call.
        pub unsafe fn value_ptr(&self) -> *const N::Value {
            // SAFETY: the reference lasts only through its conversion to a raw pointer.
            std::ptr::from_ref(unsafe { self.readonly_value() })
        }
    };
}

impl<N: BasicMapNode, const O: bool> BaseMapBasicIterator<'_, N, false, false, O> {
    readonly_access!();
}
impl<N: BasicMapNode, const M: bool, const O: bool> BaseMapBasicIterator<'_, N, M, true, O> {
    readonly_access!();
}
impl<N: BasicMapNodeMut, const O: bool> BaseMapBasicIterator<'_, N, true, false, O> {
    /// Reads native writable dereference through ChangeValue; `*unsafe { cursor.value() } = value;`.
    ///
    /// # Safety
    /// The node must satisfy change_value's exclusive-node and reference-lifetime contract.
    pub unsafe fn value(&mut self) -> &mut N::Value {
        // SAFETY: caller provides the same writable-node contract.
        unsafe { self.change_value() }
    }
    /// Returns native mutable arrow identity through ChangeValue; `unsafe { cursor.value_ptr() }`.
    ///
    /// # Safety
    /// The node must satisfy change_value's exclusive-node contract for this call.
    pub unsafe fn value_ptr(&mut self) -> *mut N::Value {
        // SAFETY: caller provides the same writable-node contract.
        unsafe { self.change_value_ptr() }
    }
}
