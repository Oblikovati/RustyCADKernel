//! Membership checks retain pointer provenance and repair stale safe cursor predecessors.
use super::{
    BaseListRawIterator, BaseListStorage, List, ListCursor, ListLinkNode, no_such_object, owner,
};
use crate::standard::Failure;
impl<T> List<T> {
    /// Borrows the actual native list base for raw iterator construction; `list.base_storage().size()`.
    pub fn base_storage(&self) -> &BaseListStorage {
        self.topology.base()
    }
    /// Creates a handle to actual native owner metadata; `let observer = list.observer()`.
    pub fn observer(&self) -> super::ListObserver {
        self.topology.observer()
    }
    pub(super) fn locate_cursor(
        &self,
        cursor: ListCursor,
        member: &str,
    ) -> Result<BaseListRawIterator, Failure> {
        self.find_cursor(cursor)
            .ok_or_else(|| no_such_object(member, &format!("{cursor:?} outside this list")))
    }
    pub(super) fn owned_node(&self, target: Option<usize>) -> Option<*mut ListLinkNode> {
        // Return the allocation's retained pointer, never reconstruct provenance from its address.
        self.owners.get(&target?).map(|owner| owner.link)
    }
    fn find_cursor(&self, cursor: ListCursor) -> Option<BaseListRawIterator> {
        let current = self.owned_node(cursor.address)?;
        let previous = if self.base_storage().first == current {
            std::ptr::null_mut()
        } else if let Some(previous) = self
            .owned_node(cursor.previous)
            .filter(|previous| unsafe { (**previous).next() == current })
        {
            previous
        } else {
            self.find_predecessor(current)?
        };
        Some(BaseListRawIterator { current, previous })
    }
    fn find_predecessor(&self, current: *mut ListLinkNode) -> Option<*mut ListLinkNode> {
        // Detached safe cursors survive edits; repair only when the remembered link became stale.
        let mut previous = self.base_storage().first;
        while !previous.is_null() {
            let next = unsafe { (*previous).next() };
            if next == current {
                return Some(previous);
            }
            previous = next;
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
