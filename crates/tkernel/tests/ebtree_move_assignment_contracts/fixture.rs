use crate::trace::{Item, Trace};
use rustycad_tkernel::ncollection::{BaseAllocator, EBTree, HashKey};
use std::fmt::{Debug, Formatter, Result};
impl PartialEq for Item<'O'> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl Debug for Item<'O'> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        Debug::fmt(&self.value, formatter)
    }
}
impl HashKey for Item<'O'> {
    fn occt_hash(&self) -> usize {
        self.value as usize
    }
}
pub type Editable = EBTree<Item<'O'>, Item<'B'>>;
pub fn editable_with_traces(
    object: &Trace,
    bounds: &Trace,
    allocator: BaseAllocator,
    start: i32,
) -> Editable {
    object.install_defaults();
    let mut tree = Editable::with_allocator(allocator);
    for value in start..start + 3 {
        tree.add(object.item(value), bounds.item(value));
    }
    object.clear();
    bounds.clear();
    tree
}
pub fn editable_populated(trace: &Trace, allocator: BaseAllocator, start: i32) -> Editable {
    editable_with_traces(trace, trace, allocator, start)
}
pub fn optional_editable(
    full: bool,
    trace: &Trace,
    allocator: BaseAllocator,
    start: i32,
) -> Editable {
    if full {
        editable_populated(trace, allocator, start)
    } else {
        Editable::with_allocator(allocator)
    }
}
