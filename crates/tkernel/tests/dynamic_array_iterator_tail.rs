//! Supplemental native method probes; original legacy test coverage remains separate.
use rustycad_tkernel::ncollection::{DynamicArray, DynamicArrayIterator};
use std::fmt::Write;

struct Item {
    value: i32,
}
fn fixture() -> DynamicArray<Item> {
    let mut array = DynamicArray::with_increment(2);
    for value in [10, 20, 30, 40, 50] {
        array.append(Item { value });
    }
    array
}
fn row(output: &mut String, name: &str, values: &[i64]) {
    output.push_str(name);
    for value in values {
        write!(output, " {value}").unwrap();
    }
    output.push('\n');
}
fn item_value<const M: bool>(
    cursor: DynamicArrayIterator<Item, M>,
    array: &DynamicArray<Item>,
) -> i64 {
    i64::from(cursor.value(array).unwrap().value)
}
fn mutable_rows(array: &mut DynamicArray<Item>, output: &mut String) {
    let cursor = DynamicArrayIterator::<Item, true>::new(array, 1);
    cursor.basic_value_mut(array).unwrap().value = 21;
    let pointer = cursor.change_value_ptr(array).unwrap();
    // The owner is alive and no reference or other pointer is used during this write.
    #[allow(unsafe_code)]
    unsafe {
        (*pointer).value += 2;
    }
    cursor.change_value_at(array, 2).unwrap().value = 44;
    cursor.change_value_at(array, -1).unwrap().value = 11;
    let values = [0, 1, 3].map(|index| i64::from(array[index].value));
    row(output, "mutable", &values);
    pointer_row(array, output);
}
fn pointer_row(array: &mut DynamicArray<Item>, output: &mut String) {
    let cursor = DynamicArrayIterator::<Item, true>::new(array, 1);
    let pointer = cursor.change_value_ptr(array).unwrap();
    let identity = i64::from(std::ptr::eq(pointer, &array[1]));
    let end = DynamicArrayIterator::<Item, true>::new(array, array.size());
    let exhausted = i64::from(end.change_value_ptr(array).unwrap().is_null());
    row(output, "arrow", &[identity, exhausted]);
}
fn arithmetic_rows(array: &DynamicArray<Item>, output: &mut String) {
    let cursor = array.legacy_cursor::<true>();
    let mut position = 3_isize + cursor;
    position -= 1;
    position += -1;
    let later = position + 3;
    let back = later - 2;
    let values = [position, later, back].map(|item| item_value(item, array));
    row(
        output,
        "arithmetic",
        &[values[0], values[1], values[2], (later - position) as i64],
    );
    const_arithmetic_row(array, output);
}
fn const_arithmetic_row(array: &DynamicArray<Item>, output: &mut String) {
    let cursor = array.legacy_cursor::<true>();
    let start = array.legacy_cursor::<false>();
    let end = DynamicArrayIterator::<Item>::new(array, array.size());
    let values = [
        item_value(2_isize + start, array),
        item_value(end - 1, array),
    ];
    let differences = [(end - cursor) as i64, (cursor - end) as i64];
    let values = [values[0], values[1], differences[0], differences[1]];
    row(output, "const_arithmetic", &values);
}
fn assigned_const(
    cursor: DynamicArrayIterator<Item, true>,
) -> (DynamicArrayIterator<Item>, [i64; 2]) {
    let mut assigned = DynamicArrayIterator::<Item>::default();
    let receiver = std::ptr::eq(&raw const assigned, assigned.assign_from_mutable(&cursor));
    (
        assigned,
        [i64::from(receiver), i64::from(assigned == cursor)],
    )
}
fn assigned_mutable(cursor: DynamicArrayIterator<Item, true>) -> [i64; 2] {
    let mut assigned = DynamicArrayIterator::<Item, true>::default();
    let receiver = std::ptr::eq(&raw const assigned, assigned.assign_from(&cursor));
    [i64::from(receiver), i64::from(assigned == cursor)]
}
fn conversion_rows(array: &mut DynamicArray<Item>, output: &mut String) {
    let cursor = DynamicArrayIterator::<Item, true>::new(array, 2);
    let converted = DynamicArrayIterator::<Item>::from(cursor);
    let (assigned, flags) = assigned_const(cursor);
    let copied = cursor;
    let copied_flags = assigned_mutable(copied);
    array.append(Item { value: 60 });
    let value = item_value(converted, array);
    let values = [value, flags[0], flags[1], copied_flags[0], copied_flags[1]];
    row(output, "conversion", &values);
    let captured = [converted + 3, assigned + 3, (copied + 3).into()];
    row(
        output,
        "captured",
        &captured.map(|item| i64::from(item.more())),
    );
    fresh_row(array, output);
}
fn fresh_row(array: &DynamicArray<Item>, output: &mut String) {
    let fresh = array.legacy_cursor::<false>();
    let last = i64::from(fresh.value_at(array, 5).unwrap().value);
    let second = i64::from(fresh.value_at(array, 1).unwrap().value);
    row(output, "fresh", &[last, second]);
}
fn empty_row(output: &mut String) {
    let mut array = DynamicArray::<Item>::default();
    let mutable = array.legacy_cursor::<true>();
    let constant = array.legacy_cursor::<false>();
    let null_mutable = mutable.change_value_ptr(&mut array).unwrap().is_null();
    let null_const = constant.value_ptr(&array).unwrap().is_null();
    let defaults =
        DynamicArrayIterator::<Item, true>::default() == DynamicArrayIterator::<Item>::default();
    row(
        output,
        "empty",
        &[
            i64::from(null_mutable),
            i64::from(null_const),
            i64::from(defaults),
        ],
    );
}
#[test]
fn native_iterator_rows_match_runtime_rust() {
    let mut array = fixture();
    let mut actual = String::new();
    mutable_rows(&mut array, &mut actual);
    arithmetic_rows(&array, &mut actual);
    conversion_rows(&mut array, &mut actual);
    empty_row(&mut actual);
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkernel_dynamic_iterator_tail_contracts.expected")
    );
}
#[test]
fn mutable_adapters_reject_wrong_owner_and_exhaustion() {
    let mut first = fixture();
    let mut other = fixture();
    let cursor = first.legacy_cursor::<true>();
    assert!(cursor.basic_value_mut(&mut other).is_err());
    assert!(cursor.change_value_at(&mut other, 0).is_err());
    assert!(cursor.change_value_ptr(&mut other).is_err());
    assert!(cursor.change_value_at(&mut first, -1).is_err());
    assert!(cursor.change_value_at(&mut first, 5).is_err());
    assert!((cursor + 5).basic_value_mut(&mut first).is_err());
}
#[test]
fn constant_operators_preserve_signed_positions() {
    let array = fixture();
    let start = array.legacy_cursor::<false>();
    let mut cursor = start;
    cursor += 4;
    cursor -= -1;
    assert_eq!(cursor - start, 5);
    cursor -= 3;
    assert_eq!(cursor.value(&array).unwrap().value, 30);
    assert_eq!((cursor + -1).value(&array).unwrap().value, 20);
    assert_eq!((cursor - -1).value(&array).unwrap().value, 40);
    assert_eq!((start + -1) - start, -1);
}
