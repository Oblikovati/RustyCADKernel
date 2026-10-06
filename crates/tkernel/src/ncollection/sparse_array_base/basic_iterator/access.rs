use super::{BaseSparseArrayBasicIterator, BasicSparseArraySource, BasicSparseArraySourceMut};
impl<C: BasicSparseArraySource<V>, V, const M: bool> BaseSparseArrayBasicIterator<'_, C, V, M> {
    unsafe fn item_pointer(&self) -> *mut V {
        let source = self
            .source
            .expect("SparseArray iterator detached; expected a live occupied item");
        let block = self
            .block
            .expect("SparseArray iterator without block; expected a live occupied item");
        unsafe { source.sparse_item(block, self.offset) }
    }
    /// Borrows readonly Value as used by native typed wrappers; `unsafe { cursor.value() }`.
    /// # Safety
    /// Current slot/projection must be live and readable, without mutation through aliases for the borrow.
    pub unsafe fn value(&self) -> &V {
        unsafe { &*self.item_pointer() }
    }
}
impl<C: BasicSparseArraySource<V>, V> BaseSparseArrayBasicIterator<'_, C, V, false> {
    /// Implements native const dereference with custom getItem dispatch; `unsafe { cursor.dereference() }`.
    /// # Safety
    /// Same valid-slot and returned-borrow requirements as `value`.
    pub unsafe fn dereference(&self) -> &V {
        unsafe { self.value() }
    }
    /// Implements native readonly arrow; `unsafe { cursor.arrow() }`.
    /// # Safety
    /// Current occupied slot and custom item projection must remain live for this call.
    pub unsafe fn arrow(&self) -> *const V {
        unsafe { self.item_pointer().cast_const() }
    }
}
impl<C: BasicSparseArraySourceMut<V>, V> BaseSparseArrayBasicIterator<'_, C, V, true> {
    /// Borrows writable ChangeValue as used by native typed wrappers; `unsafe { cursor.change_value() }`.
    /// # Safety
    /// The projected object must be live and exclusively accessible for the returned borrow, including copied cursors.
    pub unsafe fn change_value(&mut self) -> &mut V {
        unsafe { &mut *self.item_pointer() }
    }
    /// Implements native writable dereference; `unsafe { cursor.dereference() }`.
    /// # Safety
    /// Same exclusive-object requirements as `change_value`.
    pub unsafe fn dereference(&mut self) -> &mut V {
        unsafe { self.change_value() }
    }
    /// Implements native writable arrow without manufacturing a reference; `unsafe { cursor.arrow() }`.
    /// # Safety
    /// Current occupied slot and writable projection must be live; dereference additionally requires exclusivity.
    pub unsafe fn arrow(&self) -> *mut V {
        unsafe { self.item_pointer() }
    }
}
