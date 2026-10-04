//! Supplemental cursor protocol regression, separate from original legacy declarations.
use rustycad_tkernel::ncollection::{FlatDataItems, FlatDataMap, FlatMap, FlatMapIter};
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;

fn append_failure(output: &mut String, result: Result<&i32, Failure>) {
    let error = result.expect_err("exhausted cursor must reject access");
    writeln!(output, "{}:{}", error.exception_type(), error.message()).unwrap();
}

fn append_state(output: &mut String, more: bool, equal: bool) {
    writeln!(output, "{} {}", u8::from(more), u8::from(equal)).unwrap();
}

fn append_triple(output: &mut String, first: &i32, second: &i32, third: &i32) {
    writeln!(output, "{first} {second} {third}").unwrap();
}

fn set_fixture() -> FlatMap<i32> {
    let mut map = FlatMap::<i32>::default();
    for key in [3, 11, 4] {
        map.add(key);
    }
    map
}

fn data_fixture() -> FlatDataMap<i32, i32> {
    let mut map = FlatDataMap::<i32, i32>::default();
    for key in [3, 11, 4] {
        map.bind(key, key * 10);
    }
    map
}

fn append_set_cursor(output: &mut String) {
    let map = set_fixture();
    let mut cursor = map.iter();
    append_state(output, cursor.more(), cursor == cursor.clone());
    while cursor.more() {
        let first = cursor.key().unwrap();
        let second = cursor.key().unwrap();
        append_triple(output, first, second, cursor.value().unwrap());
        cursor.advance();
    }
    cursor.advance();
    let equal = cursor == FlatMapIter::default();
    append_state(output, cursor.more(), equal);
    append_failure(output, cursor.key());
    append_failure(output, cursor.value());
}

fn append_data_cursor(output: &mut String) {
    let map = data_fixture();
    let mut cursor = map.items();
    append_state(output, cursor.more(), cursor == cursor.clone());
    while cursor.more() {
        let first = cursor.key().unwrap();
        let second = cursor.value().unwrap();
        append_triple(output, first, second, cursor.value().unwrap());
        cursor.advance();
    }
    cursor.advance();
    let equal = cursor == FlatDataItems::default();
    append_state(output, cursor.more(), equal);
    append_failure(output, cursor.key());
    append_failure(output, cursor.value());
}

#[test]
fn legacy_cursor_observation_and_exhaustion_match_native_receipt() {
    let mut output = String::new();
    append_set_cursor(&mut output);
    append_data_cursor(&mut output);
    assert_eq!(
        output,
        include_str!("../../../tools/oracle/tkernel_flat_legacy_cursor.expected")
    );
}

#[test]
fn rust_iteration_and_legacy_inspection_share_one_cursor_position() {
    let map = set_fixture();
    let mut cursor = map.iter();
    assert_eq!(cursor.key().ok(), cursor.clone().next());
    cursor.advance();
    assert_eq!(cursor.key().ok(), cursor.next());
}

#[test]
fn rust_pair_iteration_consumes_the_current_observed_pair() {
    let mut bindings = FlatDataMap::<i32, i32>::default();
    bindings.bind(3, 30);
    let mut pairs = bindings.items();
    assert_eq!(
        (pairs.key().unwrap(), pairs.value().unwrap()),
        pairs.next().unwrap()
    );
    assert!(!pairs.more());
}
