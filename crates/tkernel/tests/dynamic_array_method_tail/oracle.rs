use super::super::{AssertUnwindSafe, DynamicArray, PROBE, TailItem, catch_unwind, setup};
use std::fmt::Write;
fn appended_row(output: &mut String) {
    setup(1, None);
    let mut array = DynamicArray::<TailItem>::with_increment(2);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.appended();
        }))
        .is_err()
    );
    let defaults = PROBE.with(|state| state.borrow().defaults);
    writeln!(
        output,
        "semantic_appended_failure {} {defaults}",
        array.size()
    )
    .unwrap();
}
fn set_rows(output: &mut String) {
    setup(2, None);
    let mut array = DynamicArray::with_increment(2);
    let item = TailItem {
        value: 7,
        nested: 0,
    };
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.set_value_ref(3, &item).unwrap();
        }))
        .is_err()
    );
    let counters = PROBE.with(|state| (state.borrow().defaults, state.borrow().copied));
    writeln!(
        output,
        "semantic_set_failure {} {} {}",
        array.size(),
        counters.0,
        counters.1
    )
    .unwrap();
    setup(0, None);
    array.set_value_ref(3, &item).unwrap();
    let counters = PROBE.with(|state| (state.borrow().defaults, state.borrow().copied));
    writeln!(
        output,
        "semantic_set_retry {} {} {} {}",
        array.size(),
        counters.0,
        counters.1,
        array.last().unwrap().value
    )
    .unwrap();
}
fn self_rows(output: &mut String) {
    let mut array = DynamicArray::with_increment_i32(-4);
    let first = array.append(7_i32) as *const i32;
    let identity = &raw mut array;
    let same = std::ptr::eq(array.assign_self(true), identity)
        && std::ptr::eq(array.assign_operator_self(), identity)
        && std::ptr::eq(array.move_self(), identity);
    writeln!(
        output,
        "semantic_self {} {} {}",
        u8::from(same),
        u8::from(std::ptr::eq(array.first().unwrap(), first)),
        array.size()
    )
    .unwrap();
    array.clear(false);
    array.set_increment_i32(-3);
    array.append(8);
    writeln!(
        output,
        "semantic_signed_retention {} {} {}",
        u8::from(std::ptr::eq(array.first().unwrap(), first)),
        array.size(),
        array.first().unwrap()
    )
    .unwrap();
}
#[test]
fn native_semantic_rows_match_rust_runtime_values() {
    let mut actual = String::new();
    appended_row(&mut actual);
    set_rows(&mut actual);
    self_rows(&mut actual);
    let expected =
        include_str!("../../../../tools/oracle/tkernel_dynamic_method_tail_contracts.expected")
            .lines()
            .filter(|line| line.starts_with("semantic_"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
    assert_eq!(actual, expected);
}
