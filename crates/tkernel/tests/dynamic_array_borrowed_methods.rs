//! Borrowed adapters select independent native constructor and assignment operations.
use rustycad_tkernel::ncollection::{ArrayError, BaseAllocator, DynamicArray};
use std::{
    cell::RefCell,
    fmt::Write,
    panic::{AssertUnwindSafe, catch_unwind},
};
mod dynamic_array_borrowed_methods {
    pub mod contracts;
    pub mod moving;
    pub mod oracle;
}
#[derive(Default)]
struct BorrowedProbe {
    events: String,
    failure: &'static str,
    allocator: Option<BaseAllocator>,
    allocations: usize,
    move_count: usize,
}
thread_local! {
    // Default/Drop have no injectable arguments; each independent test owns this context.
    static BORROWED_PROBE: RefCell<BorrowedProbe> = RefCell::default();
}
#[derive(Debug)]
struct BorrowedItem {
    value: i32,
}
fn event(text: String) -> &'static str {
    BORROWED_PROBE.with(|cell| {
        let mut probe = cell.borrow_mut();
        let count = probe.allocator.as_ref().map(allocation_count).unwrap_or(0);
        let added = count - probe.allocations;
        probe.events.push_str(&"A;".repeat(added));
        probe.allocations = count;
        write!(probe.events, "{text};").unwrap();
        probe.failure
    })
}
fn allocation_count(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|entry| entry.alloc_count as usize)
        .sum()
}
impl Default for BorrowedItem {
    fn default() -> Self {
        assert_ne!(event("D".into()), "default", "default 0: expected success");
        Self { value: 0 }
    }
}
impl Drop for BorrowedItem {
    fn drop(&mut self) {
        event(format!("X{}", self.value));
    }
}
fn copy_construct(source: &BorrowedItem) -> BorrowedItem {
    assert_ne!(
        event(format!("C{}", source.value)),
        "construct",
        "copy 99: expected success"
    );
    BorrowedItem {
        value: source.value,
    }
}
fn copy_assign(target: &mut BorrowedItem, source: &BorrowedItem) {
    assert_ne!(
        event(format!("P{},{}", target.value, source.value)),
        "assign",
        "copy assignment: expected success"
    );
    target.value = source.value;
}
fn move_assign(target: &mut BorrowedItem, source: &mut BorrowedItem) {
    let failure = event(format!("M{},{}", target.value, source.value));
    target.value = source.value;
    source.value = -7;
    let call = BORROWED_PROBE.with(|probe| {
        let mut probe = probe.borrow_mut();
        probe.move_count += 1;
        probe.move_count
    });
    assert!(
        !((call == 1 && failure == "move") || (target.value == 99 && failure == "final_move")),
        "move 1: expected success"
    );
}
fn fixture(failure: &'static str) -> DynamicArray<BorrowedItem> {
    BORROWED_PROBE.with(|probe| *probe.borrow_mut() = BorrowedProbe::default());
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut array = DynamicArray::with_allocator(2, Some(allocator.clone()));
    array.append(BorrowedItem { value: 10 });
    array.append(BorrowedItem { value: 20 });
    BORROWED_PROBE.with(|probe| {
        *probe.borrow_mut() = BorrowedProbe {
            failure,
            allocations: allocation_count(&allocator),
            allocator: Some(allocator),
            ..BorrowedProbe::default()
        }
    });
    array
}
fn operate<'a, I: TryInto<usize> + Copy + std::fmt::Debug>(
    array: &'a mut DynamicArray<BorrowedItem>,
    source: &BorrowedItem,
    operation: u8,
    index: I,
) -> Result<&'a mut BorrowedItem, ArrayError> {
    match operation {
        0 => Ok(array.append_ref_with_constructor(source, copy_construct)),
        1 => array.set_value_ref_with_assignment(index, source, copy_construct, move_assign),
        2 => array.insert_before_ref_with_assignment(index, source, move_assign, copy_assign),
        _ => array.insert_after_ref_with_assignment(index, source, move_assign, copy_assign),
    }
}
