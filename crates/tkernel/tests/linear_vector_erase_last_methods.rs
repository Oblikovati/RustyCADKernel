//! EraseLast destroys in its original slot, including a recoverable destructor panic.
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "linear_vector_erase_last_methods/item.rs"]
mod item;
use item::{LastItem, LastTrace};

fn native_row(name: &str, capacity: usize, count: usize, calls: usize, fail: i32, append: bool) {
    let trace = Rc::new(LastTrace::default());
    let mut vector = populated(capacity, count, &trace);
    let before = vector.data_ptr();
    trace.failing_value.set(fail);
    trace.active.set(true);
    let outcomes = erase_calls(&mut vector, calls);
    if append {
        vector.emplace_append(|| LastItem::new(99, &trace));
    }
    let row = result_row(name, &vector, before, &trace, &outcomes);
    trace.active.set(false);
    assert_native_row(&row);
    drop(vector);
    assert_eq!(trace.resources.get(), 0);
}

fn assert_native_row(row: &str) {
    assert!(
        include_str!("../../../tools/oracle/tkernel_linear_erase_last.expected")
            .lines()
            .any(|line| line == row),
        "missing native EraseLast row {row:?}"
    );
}

fn populated(capacity: usize, count: usize, trace: &Rc<LastTrace>) -> LinearVector<LastItem> {
    let mut vector = LinearVector::with_capacity(capacity);
    for index in 0..count {
        vector.emplace_append(|| LastItem::new(10 + i32::try_from(index).unwrap(), trace));
    }
    *trace.slots.borrow_mut() = vector.iter().map(std::ptr::from_ref).collect();
    vector
}

fn erase_calls(vector: &mut LinearVector<LastItem>, calls: usize) -> String {
    let mut outcomes = String::new();
    for _ in 0..calls {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| vector.erase_last()));
        outcomes.push_str(if result.is_ok() { "ok," } else { "throw," });
    }
    outcomes
}

fn result_row(
    name: &str,
    vector: &LinearVector<LastItem>,
    before: *const LastItem,
    trace: &LastTrace,
    outcomes: &str,
) -> String {
    let values: String = vector
        .iter()
        .map(|item| format!("{},", item.value))
        .collect();
    format!(
        "{name}:{outcomes}:{}:{}:{}:{}:{values}:{}",
        vector.len(),
        vector.capacity(),
        u8::from(before == vector.data_ptr()),
        trace.events.borrow(),
        trace.resources.get()
    )
}

macro_rules! native_last {
    ($test:ident,$row:literal,$cap:expr,$count:expr,$calls:expr,$fail:expr,$append:expr) => {
        #[test]
        fn $test() {
            native_row($row, $cap, $count, $calls, $fail, $append);
        }
    };
}
native_last!(empty, "empty", 0, 0, 1, -99, false);
native_last!(reserved_empty, "reserved-empty", 4, 0, 1, -99, false);
native_last!(one, "one", 4, 1, 1, -99, false);
native_last!(many, "many", 4, 3, 1, -99, false);
native_last!(all, "all", 4, 3, 4, -99, false);
native_last!(append, "append", 4, 3, 1, -99, true);
native_last!(throw_tail, "throw-tail", 4, 3, 1, 12, false);
native_last!(throw_single, "throw-single", 4, 1, 1, 10, false);
native_last!(throw_then_next, "throw-then-next", 4, 3, 2, 12, false);
native_last!(throw_then_append, "throw-then-append", 4, 3, 1, 12, true);

#[test]
fn borrowed_elements_keep_original_owner_lifetime() {
    let owner = String::from("borrowed");
    let mut vector = LinearVector::with_capacity(2);
    vector.append(owner.as_str());
    vector.append(owner.as_str());
    vector.erase_last();
    assert_eq!(vector.as_slice(), [owner.as_str()]);
}
