//! Transfers use the actual BaseList chain; cross-resource copies commit one node at a time.
use super::{List, ListCursor};
use crate::standard::Failure;
impl<T: Clone> List<T> {
    /// Copies contents while retaining this allocator (OCC25348); `destination.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.clear();
        for value in source {
            self.append_ref(value);
        }
        self
    }
    /// Appends and empties another list; `destination.append_list(&mut source)`.
    pub fn append_list(&mut self, source: &mut Self) {
        if self.allocator() != source.allocator() {
            self.append_copied_list(source);
            return;
        }
        self.owners.reserve(source.owners.len());
        unsafe { self.base.append_list(&mut source.base) };
        self.owners.append(&mut source.owners);
    }
    /// Prepends another list, panicking on native failure; `destination.prepend_list(&mut source)`.
    pub fn prepend_list(&mut self, source: &mut Self) {
        self.prepend_list_checked(source).expect(
            "native list prepend failed; expected a nonempty destination for a foreign allocator",
        );
    }
    /// Preserves native cross-resource prepend failures; `destination.prepend_list_checked(&mut source)?`.
    pub fn prepend_list_checked(&mut self, source: &mut Self) -> Result<(), Failure> {
        if self.allocator() != source.allocator() {
            return self.insert_copied_list_before(source, &mut self.cursor());
        }
        self.owners.reserve(source.owners.len());
        unsafe { self.base.prepend_list(&mut source.base) };
        self.owners.append(&mut source.owners);
        Ok(())
    }
    /// Splices before a node while retaining caller cursor state; `list.insert_list_before(&mut source, &cursor)?`.
    pub fn insert_list_before(
        &mut self,
        source: &mut Self,
        cursor: &ListCursor,
    ) -> Result<(), Failure> {
        let mut position = *cursor;
        self.insert_list_before_at(source, &mut position)
    }
    /// Splices before current and updates native previous; `list.insert_list_before_at(&mut source, &mut cursor)?`.
    pub fn insert_list_before_at(
        &mut self,
        source: &mut Self,
        cursor: &mut ListCursor,
    ) -> Result<(), Failure> {
        if self.allocator() != source.allocator() {
            return self.insert_copied_list_before(source, cursor);
        }
        let mut raw = self.locate_cursor(*cursor, "InsertBefore")?;
        self.owners.reserve(source.owners.len());
        unsafe { self.base.insert_list_before(&mut source.base, &mut raw)? };
        self.owners.append(&mut source.owners);
        cursor.update_raw(raw);
        Ok(())
    }
    /// Splices after current; an end cursor appends, as native does; `list.insert_list_after(&mut source, &cursor)?`.
    pub fn insert_list_after(
        &mut self,
        source: &mut Self,
        cursor: &ListCursor,
    ) -> Result<(), Failure> {
        if !cursor.more() {
            self.append_list(source);
            return Ok(());
        }
        let mut raw = self.locate_cursor(*cursor, "InsertAfter")?;
        if self.allocator() != source.allocator() {
            let mut following = *cursor;
            following.next(self);
            return self.insert_copied_list_before(source, &mut following);
        }
        self.owners.reserve(source.owners.len());
        unsafe { self.base.insert_list_after(&mut source.base, &mut raw)? };
        self.owners.append(&mut source.owners);
        Ok(())
    }
    fn append_copied_list(&mut self, source: &mut Self) {
        // Native appendList commits each node before constructing the next one.
        for value in source.iter() {
            self.append_ref(value);
        }
        source.clear();
    }
    fn insert_copied_list_before(
        &mut self,
        source: &mut Self,
        cursor: &mut ListCursor,
    ) -> Result<(), Failure> {
        // Native checks after each copy; an empty source never validates the cursor.
        for value in source.iter() {
            self.insert_before_at_ref(value, cursor)?;
        }
        source.clear();
        Ok(())
    }
}
impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(self.allocator().clone());
        copy.assign(self);
        copy
    }
    fn clone_from(&mut self, source: &Self) {
        // OCC25348 retains destination allocator, unlike copy construction.
        self.assign(source);
    }
}
impl<T> List<T> {
    /// Preserves native self-append's early return; `list.append_self()`.
    pub fn append_self(&mut self) {}
    /// Preserves native self-prepend's early return; `list.prepend_self()`.
    pub fn prepend_self(&mut self) {}
    /// Preserves native self-insertion-before's early return even at end; `list.insert_before_self(&cursor)`.
    pub fn insert_before_self(&mut self, _cursor: &ListCursor) {}
}
