//! Raw-engine facade migration compared against OCCT 3d097a; Rust-only safety extensions separate.
#[path = "sparse_typed_storage_contracts/mod.rs"]
mod sparse_typed_storage_contracts;
use rustycad_tkernel::ncollection::SparseArray;
use sparse_typed_storage_contracts::fixture::{TraceLog, assert_native_row, caught};
#[test]
fn typed_native_stable_growth() {
    let mut array = SparseArray::new(4);
    let first = std::ptr::from_mut(array.set_value(1, 1));
    array.set_value(4096, 4096);
    let replaced = std::ptr::from_mut(array.set_value_ref(1, &2));
    assert_native_row(
        "stable_growth",
        format!(
            "stable_growth {} {} {} {}",
            u8::from(first == std::ptr::from_ref(array.value(1).unwrap()).cast_mut()),
            u8::from(replaced == first),
            array.len(),
            array.capacity()
        ),
    );
}
#[test]
fn typed_native_assign_retains_address_and_capacity() {
    let trace = TraceLog::new();
    let source = trace.array(4, &[(1, 11)]);
    let mut target = trace.array(4, &[(1, 1), (64, 64)]);
    let retained = std::ptr::from_ref(target.value(1).unwrap());
    trace.clear();
    target.assign(&source);
    assert_native_row(
        "assign_retained",
        format!(
            "assign_retained {} {} {} {}",
            trace.events(),
            u8::from(retained == std::ptr::from_ref(target.value(1).unwrap())),
            target.len(),
            target.capacity()
        ),
    );
}
#[test]
fn typed_native_assign_changes_geometry() {
    let trace = TraceLog::new();
    let source = trace.array(3, &[(2, 22)]);
    let mut target = trace.array(4, &[(1, 1)]);
    trace.clear();
    target.assign(&source);
    assert_native_row(
        "assign_geometry",
        format!(
            "assign_geometry {} {} {} {}",
            trace.events(),
            target.len(),
            target.capacity(),
            target.value(2).unwrap().value
        ),
    );
}
#[test]
fn typed_native_create_failure_preserves_partial_assignment() {
    let trace = TraceLog::new();
    let source = trace.array(4, &[(0, 10), (2, 20)]);
    let mut target = trace.array(4, &[(0, 1), (3, 3)]);
    trace.clear();
    trace.reject_create.set(20);
    let failed = failed_assign(&mut target, &source);
    assert_native_row(
        "create_failure",
        format!(
            "create_failure {} {} {} {} {} {}",
            trace.events(),
            u8::from(failed),
            target.len(),
            target.value(0).unwrap().value,
            u8::from(target.has_value(2)),
            u8::from(target.has_value(3))
        ),
    );
}
#[test]
fn typed_native_copy_failure_preserves_modified_value() {
    let trace = TraceLog::new();
    let source = trace.array(4, &[(0, 10), (2, 20)]);
    let mut target = trace.array(4, &[(0, 1), (2, 2)]);
    trace.clear();
    trace.reject_assign.set(20);
    let failed = failed_assign(&mut target, &source);
    assert_native_row(
        "assign_failure",
        format!(
            "assign_failure {} {} {} {} {}",
            trace.events(),
            u8::from(failed),
            target.len(),
            target.value(0).unwrap().value,
            target.value(2).unwrap().value
        ),
    );
}
#[test]
fn typed_native_failed_create_retains_empty_block() {
    let trace = TraceLog::new();
    let array = failed_create_array(&trace);
    let cursor = array.cursor();
    assert_native_row(
        "empty_allocated",
        format!(
            "empty_allocated {} {} {} {}",
            array.len(),
            array.capacity(),
            u8::from(cursor.more()),
            cursor.index()
        ),
    );
    assert_eq!(array.legacy_cursor().index(), 68);
}
#[test]
fn typed_native_clear_releases_failed_create_block() {
    let trace = TraceLog::new();
    let mut array = failed_create_array(&trace);
    trace.clear();
    array.clear();
    assert_native_row(
        "clear_failed_block",
        format!(
            "clear_failed_block {} {} {}",
            trace.events().len(),
            array.len(),
            array.capacity()
        ),
    );
}
fn failed_create_array(
    trace: &TraceLog,
) -> SparseArray<sparse_typed_storage_contracts::fixture::TraceItem> {
    let item = trace.item(9);
    let mut array = SparseArray::new(4);
    trace.reject_create.set(9);
    assert!(caught(|| {
        array.set_value_ref(33, &item);
    }));
    array
}
fn failed_assign<T: Clone>(target: &mut SparseArray<T>, source: &SparseArray<T>) -> bool {
    caught(|| {
        target.assign(source);
    })
}
#[test]
fn typed_native_exchange_keeps_object_addresses() {
    let trace = TraceLog::new();
    let mut first = trace.array(3, &[(2, 2)]);
    let mut second = trace.array(4, &[(17, 17)]);
    let first_ptr = std::ptr::from_ref(first.value(2).unwrap());
    let second_ptr = std::ptr::from_ref(second.value(17).unwrap());
    trace.clear();
    first.exchange(&mut second);
    assert_native_row(
        "exchange_addresses",
        format!(
            "exchange_addresses {} {} {} {} {}",
            trace.events().len(),
            u8::from(first_ptr == std::ptr::from_ref(second.value(2).unwrap())),
            u8::from(second_ptr == std::ptr::from_ref(first.value(17).unwrap())),
            first.capacity(),
            second.capacity()
        ),
    );
}
#[test]
fn typed_native_clear_destroys_in_ascending_order() {
    let trace = TraceLog::new();
    let mut array = trace.array(4, &[(5, 5), (1, 1), (0, 0)]);
    trace.clear();
    array.clear();
    assert_native_row(
        "typed_clear",
        format!(
            "typed_clear {} {} {}",
            trace.events(),
            array.len(),
            array.capacity()
        ),
    );
}
