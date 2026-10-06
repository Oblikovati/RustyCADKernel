//! Singly linked OCCT lists with allocator-owned stable nodes.
use super::{BaseAllocator, BaseListRawIterator, BaseListStorage, ListLinkNode, ListValueNode};
use crate::standard::Failure;
use owner::{ListNodeOwner, ListNodeOwners};
use std::marker::PhantomData;
/// Allocator-owned list; `List::from_iter_in([1,2], BaseAllocator::common())`.
pub struct List<T> {
    topology: topology::ListTopology,
    owners: ListNodeOwners<T>,
}
// SAFETY: base links are inert ownership metadata; shared access never mutates them.
// Owners enforce the same item Send/Sync bounds as the former boxed chain.
unsafe impl<T: Send> Send for List<T> {}
unsafe impl<T: Sync> Sync for List<T> {}
/// Detached node position, usable across insertions; `list.cursor().value(&list)`.
///
/// `ListCursor::default()` is the end position, like a default `NCollection_List::Iterator`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ListCursor {
    address: Option<usize>,
    previous: Option<usize>,
}
/// Forward shared iterator; `list.iter()`.
pub struct ListIter<'a, T> {
    next: *mut ListLinkNode,
    item: PhantomData<&'a T>,
    previous: Option<usize>,
}
/// Forward exclusive iterator; `list.iter_mut()`.
pub struct ListIterMut<'a, T> {
    next: *mut ListLinkNode,
    item: PhantomData<&'a mut T>,
    previous: Option<usize>,
}
impl<T> List<T> {
    /// Creates an empty list with the chosen resource; `List::<i32>::with_allocator(arena)`.
    pub fn with_allocator(resource: BaseAllocator) -> Self {
        Self {
            topology: topology::ListTopology::new(resource),
            owners: ListNodeOwners::new(),
        }
    }
    /// Constructs items in a selected resource; `List::from_iter_in([1,2], arena)`.
    pub fn from_iter_in(values: impl IntoIterator<Item = T>, resource: BaseAllocator) -> Self {
        let mut list = Self::with_allocator(resource);
        for value in values {
            list.append(value);
        }
        list
    }
    /// Returns allocator identity; `list.allocator() == &arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        self.base_storage().allocator()
    }
    /// Returns the element count; `list.len() == 2`.
    pub fn len(&self) -> usize {
        self.base_storage().size()
    }
    /// Returns OCCT's signed extent; `list.extent() == 2`.
    pub fn extent(&self) -> i32 {
        self.base_storage().extent()
    }
    /// Returns OCCT's signed length, an alias of the extent; `list.length() == 2`.
    pub fn length(&self) -> i32 {
        self.extent()
    }
    /// Checks whether the chain is empty; `list.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.base_storage().is_empty()
    }
    /// Borrows the head, `None` where OCCT raises NoSuchObject; `list.first() == Some(&1)`.
    pub fn first(&self) -> Option<&T> {
        self.endpoint_ref(self.base_storage().first)
    }
    /// Borrows the head for mutation; `*list.first_mut().unwrap() = 42`.
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.endpoint_mut(self.base_storage().first)
    }
    /// Borrows the final element; `list.last() == Some(&2)`.
    pub fn last(&self) -> Option<&T> {
        self.endpoint_ref(self.base_storage().last)
    }
    /// Borrows the final element for mutation; `*list.last_mut().unwrap() = 42`.
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.endpoint_mut(self.base_storage().last)
    }
    /// Borrows all elements in chain order; `list.iter().count()`.
    pub fn iter(&self) -> ListIter<'_, T> {
        ListIter {
            next: self.base_storage().first,
            item: PhantomData,
            previous: None,
        }
    }
    /// Borrows exclusive elements in chain order; `list.iter_mut().for_each(|v| *v += 1)`.
    pub fn iter_mut(&mut self) -> ListIterMut<'_, T> {
        ListIterMut {
            next: self.base_storage().first,
            item: PhantomData,
            previous: None,
        }
    }
    /// Returns a stable position at the head; `let mut cursor = list.cursor()`.
    pub fn cursor(&self) -> ListCursor {
        ListCursor {
            address: address(self.base_storage().first),
            previous: None,
        }
    }
    /// Appends a freshly allocated node; `*list.append(42) == 42`.
    pub fn append(&mut self, value: T) -> &mut T {
        self.emplace_append(|| value)
    }
    /// Prepends a freshly allocated node; `*list.prepend(42) == 42`.
    pub fn prepend(&mut self, value: T) -> &mut T {
        self.prepend_with(|| value)
    }
    /// Allocates before constructing the head value, preserving native placement-new order; `list.prepend_with(|| 42)`.
    pub fn prepend_with(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        let owner = ListNodeOwner::construct(self.allocator(), construct);
        let node = owner.link;
        self.owners.insert(node as usize, owner);
        self.topology
            .edit(|base| unsafe { base.prepend_node(node) });
        unsafe { owner::item_mut(node) }
    }
    /// Keeps selected nodes without reallocating survivors; `list.retain(|value| *value > 0)`.
    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        let result = self.try_retain(|value| Ok::<bool, std::convert::Infallible>(keep(value)));
        match result {
            Ok(()) => (),
            Err(never) => match never {},
        }
    }
    /// Retain until the first failure, preserving earlier removals; `list.try_retain(|value| Ok::<bool, ()>(*value > 0))?`.
    pub fn try_retain<E>(&mut self, mut keep: impl FnMut(&T) -> Result<bool, E>) -> Result<(), E> {
        let mut cursor = self.cursor();
        while let Some(value) = cursor.value(self) {
            if keep(value)? {
                cursor.next(self);
                continue;
            }
            self.remove(&mut cursor)
                .expect("retain cursor remains in its live list");
        }
        Ok(())
    }
    /// Removes the head, reporting OCCT's NoSuchObject when empty; `list.remove_first()?`.
    pub fn remove_first(&mut self) -> Result<(), Failure> {
        if self.is_empty() {
            return Err(no_such_object("RemoveFirst", "an empty list"));
        }
        self.remove(&mut self.cursor())
    }
    /// Frees nodes in native link order without recursive destruction; `list.clear()`.
    pub fn clear(&mut self) {
        self.clear_owned_chain();
    }
    /// Frees all nodes, then adopts `resource` for later nodes; `list.clear_with_allocator(arena)`.
    pub fn clear_with_allocator(&mut self, resource: BaseAllocator) {
        self.clear();
        self.topology.edit(|base| base.allocator = resource);
    }
    /// Reverses links without moving values between nodes; `list.reverse()`.
    pub fn reverse(&mut self) {
        self.topology.edit(|base| unsafe { base.reverse() });
    }
    /// Exchanges nodes and their allocator resources; `first.exchange(&mut second)`.
    pub fn exchange(&mut self, other: &mut Self) {
        self.topology.exchange(&mut other.topology);
        std::mem::swap(&mut self.owners, &mut other.owners);
    }
    /// Transfers the nodes, leaving the source empty with its own allocator (as OCCT's move
    /// constructor does); `List::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        let mut moved = Self::with_allocator(source.allocator().clone());
        moved.exchange(source);
        moved
    }
    /// Replaces this list and its allocator with the source's nodes; `list.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) -> &mut Self {
        self.clear_with_allocator(source.allocator().clone());
        self.exchange(source);
        self
    }
    /// Allocator-container spelling of append; `list.push_back(42)`.
    pub fn push_back(&mut self, value: T) {
        self.append(value);
    }
    /// Allocator-container spelling of first; `list.front() == Some(&42)`.
    pub fn front(&self) -> Option<&T> {
        self.first()
    }
}
impl<T> List<T> {
    /// Removes the first equal value; `list.remove_value(&42)`.
    pub fn remove_value<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: PartialEq<Q>,
    {
        let mut cursor = self.cursor();
        while cursor.more() {
            if cursor.value(self).is_some_and(|item| item == value) {
                return self.remove(&mut cursor).is_ok();
            }
            cursor.next(self);
        }
        false
    }
    /// Checks for an equal value in chain order; `list.contains(&42)`.
    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: PartialEq<Q>,
    {
        self.iter().any(|item| item == value)
    }
}
impl<T> Default for List<T> {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        self.clear();
    }
}
impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter_in(iter, BaseAllocator::common())
    }
}
fn address(node: *mut ListLinkNode) -> Option<usize> {
    (!node.is_null()).then_some(node as usize)
}
/// OCCT's `Standard_NoSuchObject` for a list member, naming the offending state.
fn no_such_object(member: &str, state: &str) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("NCollection_List::{member} on {state}; expected an existing node"),
        "",
    )
}
mod iteration;
mod operations;
mod positions;

mod base_iterator;
mod borrowed;
mod cursor_state;
mod method_contracts;
pub use base_iterator::BaseListIterator;

mod access;
mod owner;
mod removal;

mod native_cursor;
mod topology;
pub use native_cursor::{ListCursorObserver, ListNativeCursor};
pub use topology::ListObserver;
