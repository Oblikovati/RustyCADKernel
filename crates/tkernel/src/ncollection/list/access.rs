//! Membership checks retain pointer provenance and repair stale safe cursor predecessors.
use super::{
    BaseListRawIterator, BaseListStorage, List, ListCursor, ListLinkNode, no_such_object, owner,
};
use crate::standard::Failure;
impl<T> List<T> {
    /// Borrows the actual native list base for raw iterator construction; `list.base_storage().size()`.
    pub fn base_storage(&self) -> &BaseListStorage {
        &self.base
    }
    pub(super) fn locate_cursor(
        &self,
        cursor: ListCursor,
        member: &str,
    ) -> Result<BaseListRawIterator, Failure> {
        self.find_cursor(cursor.address)
            .ok_or_else(|| no_such_object(member, &format!("{cursor:?} outside this list")))
    }
    pub(super) fn find_cursor(&self, target: Option<usize>) -> Option<BaseListRawIterator> {
        let mut cursor = BaseListRawIterator::new(&self.base);
        while !cursor.current.is_null() {
            if super::address(cursor.current) == target {
                return Some(cursor);
            }
            cursor.previous = cursor.current;
            cursor.current = unsafe { (*cursor.current).next() };
        }
        None
    }
    pub(super) fn endpoint_ref(&self, node: *mut ListLinkNode) -> Option<&T> {
        if node.is_null() {
            return None;
        }
        Some(unsafe { owner::item_ref(node) })
    }
    pub(super) fn endpoint_mut(&mut self, node: *mut ListLinkNode) -> Option<&mut T> {
        if node.is_null() {
            return None;
        }
        Some(unsafe { owner::item_mut(node) })
    }
}
impl ListCursor {
    pub(super) fn update_raw(&mut self, cursor: BaseListRawIterator) {
        self.address = super::address(cursor.current);
        self.previous = super::address(cursor.previous);
    }
}
