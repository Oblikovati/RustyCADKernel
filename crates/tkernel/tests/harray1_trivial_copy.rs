//! Trivial wrapper construction must bypass the deliberately observable Rust Clone hook.
#[path = "harray1_trivial_copy/native_rows.rs"]
mod native_rows;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[path = "support/harray_trivial_item.rs"]
mod trivial_item;
use trivial_item::TrivialItem;

#[test]
fn native_trivial_copy_rows_match_storage_bounds_empty_ranges_and_handles() {
    assert_eq!(
        native_rows::all_rows(),
        include_str!("../../../tools/oracle/tkernel_harray1_trivial_copy.expected")
    );
}
#[test]
fn ordinary_clone_policy_is_unchanged_and_explicit_trivial_policy_bypasses_it() {
    let mut buffer = [TrivialItem(7), TrivialItem(9)];
    let source = Array1::from_slice(&mut buffer);
    assert!(catch_unwind(AssertUnwindSafe(|| HArray1::from_array(&source))).is_err());
    let copied = HArray1::from_array_trivial(&source);
    assert_eq!(
        copied
            .array1()
            .as_slice()
            .iter()
            .map(|x| x.0)
            .collect::<Vec<_>>(),
        [7, 9]
    );
}
#[test]
fn ordinary_nontrivial_clone_constructor_keeps_its_owned_copy_behavior() {
    let mut buffer = [String::from("OCCT")];
    let mut source = Array1::from_slice(&mut buffer);
    source.update_lower_bound(-7);
    let copied = HArray1::from_array(&source);
    source.change_value(-7).unwrap().push_str(" changed");
    assert_eq!(copied.array1().lower(), -7);
    assert_eq!(copied.array1().first().unwrap(), "OCCT");
    assert!(copied.array1().is_deletable());
}
