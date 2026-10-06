use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedMapIter};
use rustycad_tkernel::standard::Failure;
use std::collections::BTreeMap;
use std::sync::OnceLock;
mod stack_setting;

fn rows() -> &'static BTreeMap<&'static str, &'static str> {
    static ROWS: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    ROWS.get_or_init(|| {
        include_str!("../../../../tools/oracle/tkernel_packed_map_iterator_errors.expected")
            .lines()
            .map(|line| {
                line.split_once(' ')
                    .expect("oracle row: expected label and fields")
            })
            .collect()
    })
}
fn failure_fields(failure: &Failure) -> String {
    let trace = failure.stack_string();
    let shape = trace.starts_with("\n==Backtrace==") && trace.ends_with("\n=============");
    format!(
        "0|0|0|{}|{}|{}|{}",
        failure.exception_type(),
        failure.message(),
        u8::from(!trace.is_empty()),
        u8::from(shape)
    )
}
fn result_fields<T: PackedInteger>(cursor: &PackedMapIter<'_, T>, value: bool) -> String {
    let result = if value {
        cursor.value_checked()
    } else {
        cursor.key_checked()
    };
    match result {
        Ok(key) => format!(
            "1|{}|{}|||0|0",
            key.bits(),
            u8::from(std::ptr::eq(key, cursor.cached_value()))
        ),
        Err(failure) => failure_fields(&failure),
    }
}
fn access<T: PackedInteger>(label: &str, cursor: &PackedMapIter<'_, T>, value: bool) {
    let cached = cursor.cached_value().bits();
    let fields = result_fields(cursor, value);
    let actual = format!(
        "{}|{cached}|{fields}|{}|{}",
        u8::from(cursor.more()),
        Failure::default_stack_trace_length(),
        u8::from(cursor.cached_value().bits() == cached)
    );
    assert_eq!(actual, rows()[label], "{label}");
}
fn snapshot<T: PackedInteger>(name: &str, frames: i32, state: &str, cursor: &PackedMapIter<'_, T>) {
    for (value, tag) in [(false, "key"), (true, "value")] {
        access(&format!("{name}:{frames}:{state}:{tag}"), cursor, value);
    }
}
pub fn check_width<T: PackedInteger>(name: &str, frames: i32) {
    stack_setting::with_length(frames, || check_states::<T>(name, frames));
}
fn check_states<T: PackedInteger>(name: &str, frames: i32) {
    let empty = PackedMap::<T>::default();
    let mut populated = PackedMap::default();
    populated.add(T::from_bits(41));
    populated.add(T::from_bits(42));
    snapshot(name, frames, "default", &PackedMapIter::<T>::default());
    let mut cursor = empty.iter();
    snapshot(name, frames, "bound_empty", &cursor);
    cursor.initialize(&populated);
    snapshot(name, frames, "active", &cursor);
    while cursor.more() {
        cursor.advance();
    }
    snapshot(name, frames, "exhausted", &cursor);
    check_reinitialized(name, frames, &empty, &populated, cursor);
}
fn check_reinitialized<'a, T: PackedInteger>(
    name: &str,
    frames: i32,
    empty: &'a PackedMap<T>,
    populated: &'a PackedMap<T>,
    mut cursor: PackedMapIter<'a, T>,
) {
    cursor.reset();
    snapshot(name, frames, "reset_active", &cursor);
    cursor.initialize(empty);
    snapshot(name, frames, "initialized_empty", &cursor);
    cursor.reset();
    snapshot(name, frames, "reset_empty", &cursor);
    cursor.initialize(populated);
    snapshot(name, frames, "initialized_active", &cursor);
}
