//! Unsafe native insertion preserves observable holes until explicit caller repair.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    mem::ManuallyDrop,
    rc::Rc,
};
#[path = "linear_vector_insert_methods/mod.rs"]
mod insert_cases;
use insert_cases::{InsertFixture, InsertSpec};

fn run_case(spec: InsertSpec) {
    let mut fixture = InsertFixture::new(&spec);
    let outcome = fixture.apply(&spec);
    let row = fixture.row(&spec, outcome);
    let expected = include_str!("../../../tools/oracle/tkernel_linear_insert.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native insertion row {row:?}"
    );
    // Fixture Drop reconstructs only known dead slots and destroys the unpublished live tail.
}

macro_rules! native_insert {
    ($test:ident,$row:literal,$cap:expr,$count:expr,$index:expr,$after:expr,$moving:expr,$fail:expr,$value:expr) => {
        #[test]
        fn $test() {
            run_case(InsertSpec {
                name: $row,
                capacity: $cap,
                count: $count,
                index: $index,
                after: $after,
                moving: $moving,
                fail_shift: $fail,
                fail_insert: $value,
            });
        }
    };
}

native_insert!(head_copy, "head-copy", 8, 3, 0, false, false, -99, false);
native_insert!(
    middle_copy,
    "middle-copy",
    8,
    3,
    1,
    false,
    false,
    -99,
    false
);
native_insert!(tail_copy, "tail-copy", 8, 3, 3, false, false, -99, false);
native_insert!(after_head, "after-head", 8, 3, 0, true, false, -99, false);
native_insert!(after_tail, "after-tail", 8, 3, 2, true, true, -99, false);
native_insert!(middle_move, "middle-move", 8, 3, 1, false, true, -99, false);
native_insert!(grow_copy, "grow-copy", 3, 3, 1, false, false, -99, false);
native_insert!(grow_move, "grow-move", 3, 3, 1, false, true, -99, false);
native_insert!(empty_copy, "empty-copy", 0, 0, 0, false, false, -99, false);
native_insert!(bad_before, "bad-before", 8, 3, 4, false, false, -99, false);
native_insert!(bad_after, "bad-after", 8, 3, 3, true, false, -99, false);
native_insert!(empty_after, "empty-after", 0, 0, 0, true, false, -99, false);
native_insert!(shift_first, "shift-first", 8, 3, 0, false, false, 12, false);
native_insert!(
    shift_second,
    "shift-second",
    8,
    3,
    0,
    false,
    false,
    11,
    false
);
native_insert!(shift_last, "shift-last", 8, 3, 0, false, false, 10, false);
native_insert!(
    insert_copy_fail,
    "insert-copy-fail",
    8,
    3,
    1,
    false,
    false,
    -99,
    true
);
native_insert!(
    insert_move_fail,
    "insert-move-fail",
    8,
    3,
    1,
    false,
    true,
    -99,
    true
);
native_insert!(tail_fail, "tail-fail", 8, 3, 3, false, false, -99, true);
native_insert!(empty_fail, "empty-fail", 0, 0, 0, false, false, -99, true);
native_insert!(
    grow_insert_fail,
    "grow-insert-fail",
    3,
    3,
    1,
    false,
    false,
    -99,
    true
);

#[test]
fn safe_existing_insert_rejects_range_with_ordinary_ownership() {
    let mut vector = LinearVector::with_capacity(4);
    vector.append(String::from("retained"));
    assert!(vector.insert_before(3, String::from("rejected")).is_err());
    assert_eq!(vector.as_slice(), ["retained"]);
}

#[test]
fn borrowed_items_remain_valid_when_destructive_callbacks_cannot_unwind() {
    let owner = String::from("borrowed");
    let mut vector = LinearVector::with_capacity(4);
    vector.append(owner.as_str());
    // SAFETY: references have non-panicking Drop; all callbacks are non-panicking copies.
    unsafe {
        vector
            .insert_after_with_constructors(0, |v| *v, |v| *v, || owner.as_str())
            .unwrap()
    };
    assert_eq!(vector.as_slice(), [owner.as_str(), owner.as_str()]);
}
