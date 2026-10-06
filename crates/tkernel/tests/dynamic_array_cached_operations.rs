//! Native cached arithmetic is observable after ownership transfer without cloning elements.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{DynamicArray, DynamicArrayCachedIterator};
mod dynamic_array_cached_operations {
    pub mod contracts;
}
fn source_array() -> DynamicArray<i32> {
    let mut source = DynamicArray::with_increment(2);
    for value in 1..=6 {
        source.append(value);
    }
    source
}
fn cached_value<const M: bool>(cursor: &DynamicArrayCachedIterator<i32, M>) -> i32 {
    // Trace owners retain these initialized slots; no incompatible reference overlaps the read.
    unsafe { *cursor.value() }
}
fn postfix_row<const M: bool>(
    label: &str,
    previous: &DynamicArrayCachedIterator<i32, M>,
    cursor: &DynamicArrayCachedIterator<i32, M>,
    target: &DynamicArray<i32>,
    index: usize,
) -> String {
    format!(
        "postfix_{label} old={} now={} old_target={}\n",
        cached_value(previous),
        cached_value(cursor),
        u8::from(previous.pointer() == std::ptr::from_ref(target.value(index).unwrap()))
    )
}
fn zero_row<const M: bool>(
    cursor: &DynamicArrayCachedIterator<i32, M>,
    zero: &DynamicArrayCachedIterator<i32, M>,
    source: &DynamicArray<i32>,
) -> String {
    format!(
        "plus_zero={} original={} source={}\n",
        cached_value(zero),
        cached_value(cursor),
        u8::from(zero.pointer() == std::ptr::from_ref(source.value(0).unwrap()))
    )
}
fn difference_row<const M: bool>(
    cursor: &DynamicArrayCachedIterator<i32, M>,
    plus: &DynamicArrayCachedIterator<i32, M>,
    minus: &DynamicArrayCachedIterator<i32, M>,
    target: &DynamicArray<i32>,
) -> String {
    let target_end = DynamicArrayCachedIterator::from_shared(target, 6).unwrap();
    format!(
        "subtract_negative={} cross_const={} cross_owner={}\n",
        cached_value(minus),
        plus.positional_difference(&cursor.as_const_cursor())
            .unwrap(),
        cursor.positional_difference(&target_end).unwrap()
    )
}
// Both specializations must run identical native scenarios with their distinct owner borrows.
macro_rules! cached_trace {
    ($module:ident, $flag:literal, $capture:ident, $($exclusive:tt)?) => {
        mod $module {
            use super::*;
            type Cursor = DynamicArrayCachedIterator<i32, $flag>;
            fn postfix_rows(
                cursor: &mut Cursor, source: &mut DynamicArray<i32>, target: &DynamicArray<i32>,
            ) -> String {
                let previous = cursor.postfix_advance_with(&$($exclusive)? *source).unwrap();
                let mut rows = postfix_row("plus", &previous, cursor, target, 0);
                let previous = cursor.postfix_retreat_with(&$($exclusive)? *source).unwrap();
                rows += &postfix_row("minus", &previous, cursor, target, 1);
                rows
            }
            fn arithmetic_rows(
                cursor: &Cursor, source: &mut DynamicArray<i32>, target: &DynamicArray<i32>,
            ) -> String {
                let zero = cursor.advanced_by_with(&$($exclusive)? *source, 0).unwrap();
                let plus = cursor.advanced_by_with(&$($exclusive)? *source, 2).unwrap();
                let mut minus = plus.retreated_by_with(&$($exclusive)? *source, 1).unwrap();
                let mut rows = zero_row(cursor, &zero, source);
                rows += &format!("plus={} minus={} original={}\n",
                    cached_value(&plus), cached_value(&minus), cached_value(cursor));
                minus.subtract_with(&$($exclusive)? *source, -1).unwrap();
                rows += &difference_row(cursor, &plus, &minus, target);
                rows
            }
            fn subscript_rows(cursor: &Cursor, source: &mut DynamicArray<i32>) -> String {
                let value = *cursor.subscript_with(&$($exclusive)? *source, 0).unwrap();
                let slot = cursor.subscript_with(&$($exclusive)? *source, 0).unwrap();
                let pointer = std::ptr::from_ref(&*slot);
                let expected = std::ptr::from_ref(source.value(0).unwrap());
                let left_plus = cursor.advanced_by_with(&$($exclusive)? *source, 2).unwrap();
                format!("subscript={} source={} left_plus={} original={}\n",
                    value, u8::from(pointer == expected), cached_value(&left_plus), cached_value(cursor))
            }
            fn end_rows(cursor: &Cursor, source: &mut DynamicArray<i32>) -> String {
                let mut end = cursor.advanced_by_with(&$($exclusive)? *source, 6).unwrap();
                let previous = end.postfix_retreat_with(&$($exclusive)? *source).unwrap();
                let mut rows = format!("postfix_end old_more={} now={}\n",
                    u8::from(previous.more()), cached_value(&end));
                let previous = end.postfix_advance_with(&$($exclusive)? *source).unwrap();
                rows += &format!("postfix_last old={} now_more={}\n",
                    cached_value(&previous), u8::from(end.more()));
                rows
            }
            pub(super) fn trace() -> String {
                let mut source = source_array();
                let mut cursor = Cursor::$capture(&$($exclusive)? source, 0).unwrap();
                let target = DynamicArray::from_moved(&mut source);
                let mut rows = concat!(stringify!($module), "\n").to_owned();
                rows += &postfix_rows(&mut cursor, &mut source, &target);
                for value in 101..=106 { source.append(value); }
                rows += &arithmetic_rows(&cursor, &mut source, &target);
                rows += &subscript_rows(&cursor, &mut source);
                rows += &end_rows(&cursor, &mut source);
                rows
            }
        }
    };
}
cached_trace!(shared, false, from_shared,);
cached_trace!(mutable, true, from_mutable, mut);
fn mutable_write_row() -> String {
    let mut source = source_array();
    let cursor = DynamicArrayCachedIterator::from_mutable(&mut source, 0).unwrap();
    let target = DynamicArray::from_moved(&mut source);
    for value in 101..=106 {
        source.append(value);
    }
    *cursor.subscript_with(&mut source, 0).unwrap() = 1001;
    format!(
        "subscript_write source={} target={} original={}\n",
        source.value(0).unwrap(),
        target.value(0).unwrap(),
        cached_value(&cursor)
    )
}
#[test]
fn native_postfix_copy_subscript_and_difference_rows_match() {
    let actual = shared::trace() + &mutable::trace() + &mutable_write_row();
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkernel_dynamic_cached_operations.expected")
    );
}
