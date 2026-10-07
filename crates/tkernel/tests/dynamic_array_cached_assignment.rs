//! Assignment snapshots and legacy reinitialization have different ownership and end semantics.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{DynamicArray, DynamicArrayCachedIterator};
mod dynamic_array_cached_assignment {
    pub mod contracts;
    pub mod native_rows;
}
type MutableCursor = DynamicArrayCachedIterator<i32, true>;
type SharedCursor = DynamicArrayCachedIterator<i32>;
fn source_array() -> DynamicArray<i32> {
    let mut source = DynamicArray::with_increment(2);
    for value in 1..=4 {
        source.append(value);
    }
    source
}
fn cached_value<const M: bool>(cursor: &DynamicArrayCachedIterator<i32, M>) -> i32 {
    // Each row retains the allocation owner and accesses a live slot without overlapping mutation.
    unsafe { *cursor.value() }
}
fn comparison_row<const L: bool, const R: bool>(
    label: &str,
    left: &DynamicArrayCachedIterator<i32, L>,
    right: &DynamicArrayCachedIterator<i32, R>,
) -> String {
    format!(
        "{label} eq={} ne={} lt={} gt={} le={} ge={}\n",
        u8::from(left == right),
        u8::from(left != right),
        u8::from(left.less_than(right).unwrap()),
        u8::from(left.greater_than(right).unwrap()),
        u8::from(left.at_most(right).unwrap()),
        u8::from(left.at_least(right).unwrap())
    )
}
#[test]
fn native_assignment_initialization_and_comparison_rows_match() {
    assert_eq!(
        dynamic_array_cached_assignment::native_rows::trace(),
        include_str!("../../../tools/oracle/tkernel_dynamic_cached_assignment.expected")
    );
}
