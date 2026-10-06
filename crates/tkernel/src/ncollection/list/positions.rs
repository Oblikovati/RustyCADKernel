//! Cursor-positioned list edits (`pRemove`, `pInsertBefore`, `pInsertAfter`, `pAppend(node, it)`).
use super::{Link, List, ListCursor, Node, address, find_link, no_such_object};
use crate::standard::Failure;
impl<T> List<T> {
    /// Removes a positioned node and advances the cursor to its successor; an end cursor
    /// reports OCCT's NoSuchObject; `list.remove(&mut cursor)?`.
    pub fn remove(&mut self, cursor: &mut ListCursor) -> Result<(), Failure> {
        let link = locate(&mut self.head, "Remove", *cursor)?;
        let mut node = link.take().expect("located links hold a node");
        *link = node.next.take();
        cursor.address = address(link);
        self.count -= 1;
        Ok(())
    }
    /// Inserts immediately before a cursor, which keeps naming the same node;
    /// `*list.insert_before(20, &cursor)? == 20`.
    pub fn insert_before(&mut self, value: T, cursor: &ListCursor) -> Result<&mut T, Failure> {
        // OCCT allocates the node before pInsertBefore checks the iterator.
        self.emplace_before(cursor, || value)
    }
    /// Inserts immediately after a cursor; `*list.insert_after(20, &cursor)? == 20`.
    pub fn insert_after(&mut self, value: T, cursor: &ListCursor) -> Result<&mut T, Failure> {
        self.emplace_after(cursor, || value)
    }
    /// Appends and repositions `cursor` on the new tail, as OCCT's `Append(item, Iterator&)`;
    /// `list.append_at(42, &mut cursor)`.
    pub fn append_at(&mut self, value: T, cursor: &mut ListCursor) {
        let previous = self.tail_address();
        self.append(value);
        cursor.address = self.tail_address();
        cursor.previous = previous;
    }
    /// Constructs an appended element from arguments captured in a closure; `list.emplace_append(|| Pair(1,2.0))`.
    pub fn emplace_append(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        let node = self.resource.allocate_value_with(|| Node {
            value: construct(),
            next: None,
        });
        self.count += 1;
        &mut find_link(&mut self.head, None).insert(node).value
    }
    /// Constructs a prepended element; `list.emplace_prepend(|| Pair(1,2.0))`.
    pub fn emplace_prepend(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.prepend_with(construct)
    }
    /// Constructs before a node position; `list.emplace_before(&cursor, || Pair(1,2.0))?`.
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
        // Native constructs its ListNode before pInsertBefore validates the cursor.
        let mut node = self.resource.allocate_value_with(|| Node {
            value: construct(),
            next: None,
        });
        let link = locate(&mut self.head, "InsertBefore", *cursor)?;
        node.next = link.take();
        self.count += 1;
        cursor.previous = Some(std::ptr::from_ref(&*node) as usize);
        Ok(&mut link.insert(node).value)
    }
    /// Constructs after a node position; `list.emplace_after(&cursor, || Pair(1,2.0))?`.
    pub fn emplace_after(
        &mut self,
        cursor: &ListCursor,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        // Native constructs its ListNode before pInsertAfter validates the cursor.
        let mut node = self.resource.allocate_value_with(|| Node {
            value: construct(),
            next: None,
        });
        let located = locate(&mut self.head, "InsertAfter", *cursor)?;
        let existing = located.as_mut().expect("located links hold a node");
        node.next = existing.next.take();
        self.count += 1;
        Ok(&mut existing.next.insert(node).value)
    }
    pub(super) fn tail_address(&self) -> Option<usize> {
        let mut node = self.head.as_deref()?;
        while let Some(next) = node.next.as_deref() {
            node = next;
        }
        Some(node as *const Node<T> as usize)
    }
}
/// The link holding the cursor's node, or NoSuchObject for an end or foreign cursor.
///
/// It borrows only the chain, so callers may still update the list's count.
pub(super) fn locate<'a, T>(
    head: &'a mut Link<T>,
    member: &str,
    cursor: ListCursor,
) -> Result<&'a mut Link<T>, Failure> {
    if !cursor.more() {
        return Err(no_such_object(member, "an end cursor"));
    }
    let link = find_link(head, cursor.address);
    if link.is_none() {
        return Err(no_such_object(
            member,
            &format!("{cursor:?} outside this list"),
        ));
    }
    Ok(link)
}
