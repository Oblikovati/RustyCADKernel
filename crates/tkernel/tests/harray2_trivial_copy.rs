//! Explicit trivial matrix copying bypasses Clone and preserves inherited flat metadata.
#[path = "harray2_trivial_copy/native_rows.rs"]
mod native_rows;
#[path = "support/harray_trivial_item.rs"]
mod trivial_item;
use rustycad_tkernel::ncollection::{Array2, HArray2};
use std::panic::{AssertUnwindSafe, catch_unwind};
use trivial_item::TrivialItem;

#[test]
fn native_trivial_rows_match_rectangles_flat_storage_and_handles() {
    assert_eq!(
        native_rows::all_rows(),
        include_str!("../../../tools/oracle/tkernel_harray2_trivial_copy.expected")
    );
}
#[test]
fn ordinary_clone_remains_observable_while_trivial_copy_needs_no_default() {
    let mut buffer = [TrivialItem(7), TrivialItem(9)];
    let source = Array2::from_buffer_bounds(&mut buffer, -1, -1, 3, 4).unwrap();
    assert!(catch_unwind(AssertUnwindSafe(|| HArray2::from_array(&source))).is_err());
    let copied = HArray2::from_array_trivial(&source);
    assert_eq!(copied.array2().value(-1, 3).unwrap().0, 7);
    assert_eq!(copied.array2().value(-1, 4).unwrap().0, 9);
}
#[test]
fn ordinary_nontrivial_clone_keeps_independent_owned_storage() {
    let mut buffer = [String::from("OCCT")];
    let mut source = Array2::from_buffer_bounds(&mut buffer, -7, -7, 4, 4).unwrap();
    let copied = HArray2::from_array(&source);
    source.change_value(-7, 4).unwrap().push_str(" changed");
    assert_eq!(copied.array2().value(-7, 4).unwrap(), "OCCT");
    assert_eq!(copied.array2().array1().lower(), -3);
    assert!(copied.array2().is_deletable());
}
