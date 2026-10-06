//! One-based OCCT sequences whose items live in individually allocated, stable nodes.
use super::{AllocatedBox, BaseAllocator};
use crate::standard::Failure;
/// Allocator-owned one-based sequence; `let mut s = Sequence::default(); s.append(1);`.
pub struct Sequence<T> {
    resource: BaseAllocator,
    nodes: Vec<AllocatedBox<SequenceNode<T>>>,
}
/// Shared double-ended iterator in index order; `sequence.iter()`.
pub struct SequenceIter<'a, T> {
    remaining: std::slice::Iter<'a, AllocatedBox<SequenceNode<T>>>,
}
/// Exclusive double-ended iterator in index order; `sequence.iter_mut()`.
pub struct SequenceIterMut<'a, T> {
    remaining: std::slice::IterMut<'a, AllocatedBox<SequenceNode<T>>>,
}
/// Detached bidirectional node position (OCCT's `Sequence::Iterator`): the current node
/// and the node before it; `let cursor = sequence.cursor()`.
///
/// Equality compares only the current node, as OCCT's `IsEqual` does.
#[derive(Clone, Copy, Debug, Default)]
pub struct SequenceCursor {
    address: Option<usize>,
    previous: Option<usize>,
}
impl<T> Sequence<T> {
    /// Creates an empty sequence with a chosen allocator; `Sequence::<i32>::with_allocator(arena)`.
    pub fn with_allocator(resource: BaseAllocator) -> Self {
        Self {
            resource,
            nodes: Vec::new(),
        }
    }
    /// Returns selected allocator identity; `sequence.allocator() == &arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.resource
    }
    /// Returns the element count; `sequence.len() == 3`.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    /// Returns the signed length; `sequence.length() == 3`.
    pub fn length(&self) -> i32 {
        self.nodes.len() as i32
    }
    /// Checks for an empty sequence; `sequence.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// Returns the fixed lower bound; `sequence.lower() == 1`.
    pub const fn lower(&self) -> usize {
        1
    }
    /// Returns the upper bound, zero when empty; `sequence.upper() == sequence.len()`.
    pub fn upper(&self) -> usize {
        self.len()
    }
    /// Borrows the first item, `None` where OCCT raises NoSuchObject; `sequence.first() == Some(&10)`.
    pub fn first(&self) -> Option<&T> {
        self.nodes.first().map(|node| &node.value)
    }
    /// Borrows the first item exclusively (`ChangeFirst`); `*sequence.first_mut().unwrap() = 1`.
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.nodes.first_mut().map(|node| &mut node.value)
    }
    /// Borrows the final item; `sequence.last() == Some(&30)`.
    pub fn last(&self) -> Option<&T> {
        self.nodes.last().map(|node| &node.value)
    }
    /// Borrows the final item exclusively (`ChangeLast`); `*sequence.last_mut().unwrap() = 3`.
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.nodes.last_mut().map(|node| &mut node.value)
    }
    /// Borrows an item by one-based index; `sequence.value(1)?`.
    pub fn value(&self, index: usize) -> Result<&T, Failure> {
        let offset = self.offset("Value", index)?;
        Ok(&self.nodes[offset])
    }
    /// Borrows an item exclusively by one-based index; `*sequence.change_value(1)? = 10`.
    pub fn change_value(&mut self, index: usize) -> Result<&mut T, Failure> {
        let offset = self.offset("ChangeValue", index)?;
        Ok(&mut self.nodes[offset])
    }
    /// Replaces an existing item; `sequence.set_value(1, 10)?`.
    pub fn set_value(&mut self, index: usize, value: T) -> Result<(), Failure> {
        *self.change_value(index)? = value;
        Ok(())
    }
    /// Borrows an item by zero-based offset; `sequence.at(0)?`.
    pub fn at(&self, offset: usize) -> Result<&T, Failure> {
        self.zero_based("At", offset)?;
        self.value(offset + 1)
    }
    /// Borrows an item exclusively by zero-based offset; `*sequence.change_at(0)? = 5`.
    pub fn change_at(&mut self, offset: usize) -> Result<&mut T, Failure> {
        self.zero_based("ChangeAt", offset)?;
        self.change_value(offset + 1)
    }
    /// Borrows elements in order; `sequence.iter().sum::<i32>()`.
    pub fn iter(&self) -> SequenceIter<'_, T> {
        SequenceIter {
            remaining: self.nodes.iter(),
        }
    }
    /// Borrows exclusive elements in order; `sequence.iter_mut().for_each(|v| *v+=1)`.
    pub fn iter_mut(&mut self) -> SequenceIterMut<'_, T> {
        SequenceIterMut {
            remaining: self.nodes.iter_mut(),
        }
    }
    /// Captures a position at the first element; `let cursor=sequence.cursor()`.
    pub fn cursor(&self) -> SequenceCursor {
        SequenceCursor::new(self, true)
    }
    /// One-based index to vector offset, or OCCT's OutOfRange naming the index.
    fn offset(&self, member: &str, index: usize) -> Result<usize, Failure> {
        if index == 0 || index > self.len() {
            return Err(out_of_range(member, index, &format!("1..={}", self.len())));
        }
        Ok(index - 1)
    }
    fn zero_based(&self, member: &str, offset: usize) -> Result<(), Failure> {
        if offset >= self.len() {
            return Err(out_of_range(member, offset, &format!("0..{}", self.len())));
        }
        Ok(())
    }
}
impl<T> Default for Sequence<T> {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl<T> FromIterator<T> for Sequence<T> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        let mut sequence = Self::default();
        for item in items {
            sequence.append(item);
        }
        sequence
    }
}
/// `sequence[i]` is OCCT's `operator()(i)`; it panics with the OutOfRange message where
/// `value` returns the error.
impl<T> std::ops::Index<usize> for Sequence<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.value(index)
            .unwrap_or_else(|error| panic!("{}", error.message()))
    }
}
impl<T> std::ops::IndexMut<usize> for Sequence<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.change_value(index)
            .unwrap_or_else(|error| panic!("{}", error.message()))
    }
}
fn address<T>(value: &T) -> usize {
    value as *const T as usize
}
/// OCCT's `Standard_OutOfRange` for a sequence member, naming index and valid range.
fn out_of_range(member: &str, index: usize, expected: &str) -> Failure {
    Failure::out_of_range(&format!(
        "NCollection_Sequence::{member} index {index}; expected {expected}"
    ))
}
/// OCCT's `Standard_NoSuchObject` for a cursor that names no node of this sequence.
fn no_such_node(member: &str, cursor: SequenceCursor) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!(
            "NCollection_Sequence::{member} cursor {cursor:?}; expected a node of this sequence"
        ),
        "",
    )
}
mod base_iterator;
// Native raw topology operations require the documented live-node ownership contracts.
#[allow(unsafe_code)]
mod base_storage;
pub use base_iterator::BaseSequenceIterator;
pub use base_storage::{BaseSequence, BaseSequenceRawIterator, DelSeqNode};
mod borrowed;
mod cursor_contracts;
mod editing;
mod iteration;
mod links;
// Raw node callbacks restore the exact typed allocation and its original allocator.
#[allow(unsafe_code)]
mod node_allocation;
// Safety contracts cover opaque link Send/Sync and preserve exclusive node mutation.
#[allow(unsafe_code)]
mod nodes;
pub use nodes::{SeqNode, SequenceNode};
mod method_contracts;
mod operations;
mod signed_indices;
