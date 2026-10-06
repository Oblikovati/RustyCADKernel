//! Actual cached pointers survive transfer; original-owner reloads intentionally form hybrid views.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{DynamicArray, DynamicArrayCachedIterator};
mod dynamic_array_cached_iterator {
    pub mod contracts;
}
type MutableCursor = DynamicArrayCachedIterator<i32, true>;
fn source_array() -> DynamicArray<i32> {
    let mut source = DynamicArray::with_increment(2);
    for value in 1..=6 {
        source.append(value);
    }
    source
}
fn cached_value<const M: bool>(cursor: &DynamicArrayCachedIterator<i32, M>) -> i32 {
    // Each caller retains the allocation owner and initialized i32 slot during this shared borrow.
    unsafe { *cursor.value() }
}
fn header_row(
    mode: &str,
    cursor: &MutableCursor,
    source: &DynamicArray<i32>,
    target: &DynamicArray<i32>,
) -> String {
    let constant = cursor.as_const_cursor();
    let target_begin = DynamicArrayCachedIterator::from_shared(target, 0).unwrap();
    format!(
        "{mode} value={} same={} more={} source={} target={} equal={} converted={}\n",
        cached_value(cursor),
        u8::from(cursor.pointer() == std::ptr::from_ref(target.value(0).unwrap())),
        u8::from(cursor.more()),
        source.size(),
        target.size(),
        u8::from(cursor.same_position(&target_begin)),
        u8::from(constant.pointer() == cursor.pointer())
    )
}
fn transferred_trace(
    source: &mut DynamicArray<i32>,
    target: &mut DynamicArray<i32>,
    mut cursor: MutableCursor,
    mode: &str,
) -> String {
    let mut output = header_row(mode, &cursor, source, target);
    // Target owns the transferred allocation; no reference overlaps this exclusive cached write.
    unsafe {
        *cursor.change_value() = 11;
    }
    output.push_str(&format!("write={}\n", target.value(0).unwrap()));
    cursor.advance_with(source).unwrap();
    output.push_str(&position_row("within", "target", &cursor, target, 1));
    for value in 101..=106 {
        source.append(value);
    }
    output.push_str(&refilled_trace(source, cursor));
    output
}
fn refilled_trace(source: &mut DynamicArray<i32>, mut cursor: MutableCursor) -> String {
    let mut reloaded = cursor;
    reloaded.offset_with(source, 0).unwrap();
    let mut output = format!(
        "cached={} offset_zero={}\n",
        cached_value(&cursor),
        cached_value(&reloaded)
    );
    cursor.advance_with(source).unwrap();
    output.push_str(&position_row("boundary", "source", &cursor, source, 2));
    cursor.retreat_with(source).unwrap();
    output.push_str(&position_row("retreat", "source", &cursor, source, 1));
    output
}
fn position_row(
    label: &str,
    location: &str,
    cursor: &MutableCursor,
    array: &DynamicArray<i32>,
    index: usize,
) -> String {
    format!(
        "{label}={} {location}={}\n",
        cached_value(cursor),
        u8::from(cursor.pointer() == std::ptr::from_ref(array.value(index).unwrap()))
    )
}
fn constructor_trace() -> String {
    let mut source = source_array();
    let cursor = MutableCursor::from_mutable(&mut source, 0).unwrap();
    let mut target = DynamicArray::from_moved(&mut source);
    transferred_trace(&mut source, &mut target, cursor, "constructor")
}
fn assignment_trace() -> String {
    let mut source = source_array();
    let cursor = MutableCursor::from_mutable(&mut source, 0).unwrap();
    let mut target = DynamicArray::with_increment(4);
    target.append(90);
    target.move_from(&mut source);
    transferred_trace(&mut source, &mut target, cursor, "assignment")
}
fn original_owner_destroyed_row() -> String {
    let (cursor, target) = {
        let mut source = source_array();
        let cursor = DynamicArrayCachedIterator::from_shared(&source, 0).unwrap();
        let target = DynamicArray::from_moved(&mut source);
        (cursor, target)
    };
    format!(
        "owner_destroyed value={} same={}\n",
        cached_value(&cursor),
        u8::from(cursor.pointer() == std::ptr::from_ref(target.value(0).unwrap()))
    )
}
#[test]
fn native_move_refill_reload_and_cached_write_rows_match() {
    let actual = constructor_trace() + &assignment_trace() + &original_owner_destroyed_row();
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkernel_dynamic_cached_iterator.expected")
    );
}
