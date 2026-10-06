//! Explicit growth and value construction match native append publication phases.
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "linear_vector_append_policy/fixture.rs"]
mod fixture;
#[path = "linear_vector_append_policy/item.rs"]
mod item;
use fixture::append_case;
use item::{AppendItem, AppendTrace};

macro_rules! native_append {
    ($test:ident,$row:literal,$cap:expr,$count:expr,$mode:expr,$moving:expr,$fail:expr) => {
        #[test]
        fn $test() {
            append_case($row, $cap, $count, $mode, $moving, $fail);
        }
    };
}
native_append!(copy_kept, "copy-kept", 4, 2, 'C', false, false);
native_append!(copy_grow, "copy-grow", 2, 2, 'C', false, false);
native_append!(copy_empty, "copy-empty", 0, 0, 'C', false, false);
native_append!(copy_movegrow, "copy-movegrow", 2, 2, 'C', true, false);
native_append!(move_kept, "move-kept", 4, 2, 'M', false, false);
native_append!(move_grow, "move-grow", 2, 2, 'M', false, false);
native_append!(move_empty, "move-empty", 0, 0, 'M', false, false);
native_append!(move_movegrow, "move-movegrow", 2, 2, 'M', true, false);
native_append!(default_kept, "default-kept", 4, 2, 'A', false, false);
native_append!(default_grow, "default-grow", 2, 2, 'A', false, false);
native_append!(default_empty, "default-empty", 0, 0, 'A', false, false);
native_append!(default_movegrow, "default-movegrow", 2, 2, 'A', true, false);
native_append!(emplace_kept, "emplace-kept", 4, 2, 'E', false, false);
native_append!(emplace_grow, "emplace-grow", 2, 2, 'E', false, false);
native_append!(emplace_empty, "emplace-empty", 0, 0, 'E', false, false);
native_append!(emplace_movegrow, "emplace-movegrow", 2, 2, 'E', true, false);
native_append!(copy_fail_kept, "copy-fail-kept", 4, 2, 'C', false, true);
native_append!(copy_fail_grow, "copy-fail-grow", 2, 2, 'C', true, true);
native_append!(move_fail_kept, "move-fail-kept", 4, 2, 'M', false, true);
native_append!(move_fail_grow, "move-fail-grow", 2, 2, 'M', false, true);
native_append!(
    default_fail_empty,
    "default-fail-empty",
    0,
    0,
    'A',
    false,
    true
);
native_append!(
    default_fail_grow,
    "default-fail-grow",
    2,
    2,
    'A',
    true,
    true
);
native_append!(
    emplace_fail_kept,
    "emplace-fail-kept",
    4,
    2,
    'E',
    false,
    true
);
native_append!(
    emplace_fail_grow,
    "emplace-fail-grow",
    2,
    2,
    'E',
    false,
    true
);

#[test]
fn borrowed_elements_preserve_owner_lifetime_and_mutable_returned_slot() {
    let owner = String::from("borrowed");
    let mut vector = LinearVector::with_capacity(1);
    vector.append(owner.as_str());
    *vector.emplace_append_with_relocation(|item| *item, || "initial") = owner.as_str();
    assert_eq!(vector.as_slice(), [owner.as_str(), owner.as_str()]);
}

#[test]
fn owned_callback_captures_drop_once_when_relocation_panics() {
    let owner = Rc::new(());
    let captured = Rc::clone(&owner);
    let mut vector = LinearVector::with_capacity(1);
    vector.append(1);
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.emplace_append_with_relocation(
            |_| panic!("growth value 1: expected success"),
            || {
                drop(captured);
                2
            },
        );
    }));
    assert!(failed.is_err());
    assert_eq!(Rc::strong_count(&owner), 1);
    assert_eq!(vector.as_slice(), [1]);
}

#[test]
fn growth_panic_cleans_partial_replacement_and_skips_final_constructor() {
    let trace = Rc::new(AppendTrace::default());
    let mut vector = fixture::filled(2, 2, &trace);
    trace.active.set(true);
    let called = Cell::new(false);
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.emplace_append_with_relocation(reject_eleven, || {
            called.set(true);
            AppendItem::new(99, &trace)
        });
    }));
    trace.active.set(false);
    assert!(failed.is_err());
    assert!(!called.get());
    assert_eq!((vector.len(), vector.capacity()), (2, 2));
    assert!(vector.iter().map(|v| v.value).eq([10, 11]));
    assert_eq!(*trace.events.borrow(), "C10,D10,");
}

fn reject_eleven(source: &mut AppendItem) -> AppendItem {
    assert_ne!(
        source.value, 11,
        "growth value 11: expected successful relocation"
    );
    source.copied()
}
