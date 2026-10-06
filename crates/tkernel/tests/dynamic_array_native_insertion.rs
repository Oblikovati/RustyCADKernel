//! Exact native insertion traces use explicit callbacks for C++ user assignment operations.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray};
use std::{
    cell::RefCell,
    fmt::Write,
    panic::{AssertUnwindSafe, catch_unwind},
};
mod dynamic_array_native_insertion {
    pub mod convenience;
    pub mod oracle;
}
#[derive(Default)]
struct AssignmentProbe {
    events: String,
    allocator: Option<BaseAllocator>,
    allocations: usize,
    fail_default: bool,
    fail_move: usize,
    move_calls: usize,
    fail_copy: bool,
}
thread_local! {
    // Default has no argument: each test injects its own thread-local observation context.
    static INSERTION_PROBE: RefCell<AssignmentProbe> = RefCell::default();
}
#[derive(Debug)]
struct InsertionItem {
    value: i32,
}
fn allocation_count(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count as usize)
        .sum()
}
fn record_default() {
    INSERTION_PROBE.with(|probe| {
        let mut probe = probe.borrow_mut();
        if let Some(allocator) = &probe.allocator {
            let count = allocation_count(allocator);
            let allocated = count - probe.allocations;
            probe.events.push_str(&"A;".repeat(allocated));
            probe.allocations = count;
        }
        probe.events.push_str("D;");
        assert!(
            !probe.fail_default,
            "injected default failure: expected successful construction"
        );
    });
}
impl Default for InsertionItem {
    fn default() -> Self {
        record_default();
        Self { value: 0 }
    }
}
impl Clone for InsertionItem {
    fn clone(&self) -> Self {
        panic!(
            "clone construction for value {}: expected clone_from assignment",
            self.value
        );
    }
    fn clone_from(&mut self, source: &Self) {
        copy_assignment(self, source);
    }
}
fn copy_assignment(target: &mut InsertionItem, source: &InsertionItem) {
    INSERTION_PROBE.with(|probe| {
        let mut probe = probe.borrow_mut();
        write!(probe.events, "C({},{});", target.value, source.value).unwrap();
        assert!(
            !probe.fail_copy,
            "injected copy failure: expected successful assignment"
        );
    });
    target.value = source.value;
}
fn move_assignment(target: &mut InsertionItem, source: &mut InsertionItem) {
    INSERTION_PROBE.with(|probe| {
        let mut probe = probe.borrow_mut();
        write!(probe.events, "M({},{});", target.value, source.value).unwrap();
        probe.move_calls += 1;
        assert_ne!(
            probe.move_calls, probe.fail_move,
            "injected move failure: expected successful assignment"
        );
    });
    target.value = source.value;
    source.value = -7;
}
fn insertion_fixture() -> DynamicArray<InsertionItem> {
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut array = DynamicArray::with_allocator(2, Some(allocator.clone()));
    for value in [10, 20, 30, 40] {
        array.append(InsertionItem { value });
    }
    INSERTION_PROBE.with(|probe| {
        *probe.borrow_mut() = AssignmentProbe {
            allocations: allocation_count(&allocator),
            allocator: Some(allocator),
            ..AssignmentProbe::default()
        }
    });
    array
}
fn observed_values(array: &DynamicArray<InsertionItem>) -> Vec<i32> {
    array.iter().map(|item| item.value).collect()
}
fn observed_events() -> String {
    INSERTION_PROBE.with(|probe| probe.borrow().events.clone())
}
