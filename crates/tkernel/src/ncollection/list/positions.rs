//! Cursor edits allocate native ListValueNode values before position validation.
use super::{
    BaseListRawIterator, List, ListCursor,
    owner::{self, ListNodeOwner},
};
use crate::standard::Failure;
impl<T> List<T> {
    /// Inserts before a cursor while retaining its caller-visible state; `list.insert_before(20, &cursor)?`.
    pub fn insert_before(&mut self, value: T, cursor: &ListCursor) -> Result<&mut T, Failure> {
        self.emplace_before(cursor, || value)
    }
    /// Inserts after a cursor; `list.insert_after(20, &cursor)?`.
    pub fn insert_after(&mut self, value: T, cursor: &ListCursor) -> Result<&mut T, Failure> {
        self.emplace_after(cursor, || value)
    }
    /// Appends and updates both native cursor links; `list.append_at(42, &mut cursor)`.
    pub fn append_at(&mut self, value: T, cursor: &mut ListCursor) {
        self.append_constructed_at(cursor, || value);
    }
    pub(super) fn append_constructed_at(
        &mut self,
        cursor: &mut ListCursor,
        construct: impl FnOnce() -> T,
    ) {
        let owner = ListNodeOwner::construct(self.allocator(), construct);
        let node = owner.link;
        self.owners.push(owner);
        let mut raw = BaseListRawIterator::default();
        unsafe { self.base.append_at(node, &mut raw) };
        cursor.update_raw(raw);
    }
    /// Allocates before evaluating construction arguments; `list.emplace_append(|| Pair(1,2.0))`.
    pub fn emplace_append(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        let owner = ListNodeOwner::construct(self.allocator(), construct);
        let node = owner.link;
        self.owners.push(owner);
        unsafe { self.base.append_node(node) };
        unsafe { owner::item_mut(node) }
    }
    /// Constructs a prepended element; `list.emplace_prepend(|| Pair(1,2.0))`.
    pub fn emplace_prepend(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.prepend_with(construct)
    }
    /// Constructs before a node, retaining caller cursor state; `list.emplace_before(&cursor, || 7)?`.
    pub fn emplace_before(
        &mut self,
        cursor: &ListCursor,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        let mut position = *cursor;
        self.emplace_before_at(&mut position, construct)
    }
    /// Constructs and updates both native iterator links; `list.emplace_before_at(&mut cursor, || 7)?`.
    pub fn emplace_before_at(
        &mut self,
        cursor: &mut ListCursor,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        let owner = ListNodeOwner::construct(self.allocator(), construct);
        let mut raw = self.locate_cursor(*cursor, "InsertBefore")?;
        let node = owner.link;
        self.owners.push(owner);
        unsafe { self.base.insert_before_node(node, &mut raw)? };
        cursor.update_raw(raw);
        Ok(unsafe { owner::item_mut(node) })
    }
    /// Constructs after a position, preserving both cursor links; `list.emplace_after(&cursor, || 7)?`.
    pub fn emplace_after(
        &mut self,
        cursor: &ListCursor,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        let owner = ListNodeOwner::construct(self.allocator(), construct);
        let mut raw = self.locate_cursor(*cursor, "InsertAfter")?;
        let node = owner.link;
        self.owners.push(owner);
        unsafe { self.base.insert_after_node(node, &mut raw)? };
        Ok(unsafe { owner::item_mut(node) })
    }
}
