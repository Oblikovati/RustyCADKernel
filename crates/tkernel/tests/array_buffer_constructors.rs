//! Mutable bounded constructor domains without native aliasing or element-copy promises.
#[path = "array_buffer_constructors/support.rs"]
mod support;
use rustycad_tkernel::ncollection::{Array1, Array2};
use support::{
    BufferItem, assert_native_row, capture_counters, counter_row, probe_buffer, values_row,
};

#[test]
fn array1_bounded_constructor_reuses_the_buffer_without_element_hooks() {
    check_bounded_array1(true);
}

#[test]
fn array1_bounded_constructor_allocates_without_copying_the_input() {
    check_bounded_array1(false);
}

fn check_bounded_array1(reuse: bool) {
    let mut buffer = probe_buffer::<4>();
    let pointer = buffer.as_ptr();
    let mut array = Array1::from_buffer_bounds(&mut buffer, -2, 0, reuse).unwrap();
    let prefix = array1_bounds_row(&array, pointer == array.data_ptr());
    array[-1].value = 71;
    let counters = capture_counters(array.as_slice());
    let during = counter_row(&counters);
    drop(array);
    let after = values_row(&buffer);
    assert_native_row(
        &format!("array1_bounds_{}", u8::from(reuse)),
        format!("{prefix};{after};{during}"),
    );
    let drops = counters
        .iter()
        .map(|entry| entry.drops.get())
        .sum::<usize>();
    assert_native_row(&format!("array1_bounds_drop_{}", u8::from(reuse)), drops);
}

fn array1_bounds_row(array: &Array1<'_, BufferItem>, same_pointer: bool) -> String {
    format!(
        "{},{};{};{};{};{},{}",
        array.lower(),
        array.upper(),
        array.len(),
        u8::from(array.is_deletable()),
        u8::from(same_pointer),
        array.first().unwrap().value,
        array.last().unwrap().value
    )
}

#[test]
fn array1_sized_constructor_reuses_only_the_requested_prefix() {
    check_sized_array1(true);
}

#[test]
fn array1_sized_constructor_default_constructs_the_independent_extent() {
    check_sized_array1(false);
}

fn check_sized_array1(reuse: bool) {
    let mut buffer = probe_buffer::<4>();
    let pointer = buffer.as_ptr();
    let array = Array1::from_buffer_size(&mut buffer, 3, reuse).unwrap();
    let state = array1_bounds_row(&array, pointer == array.data_ptr());
    let counters = capture_counters(array.as_slice());
    assert_native_row(
        &format!("array1_size_{}", u8::from(reuse)),
        format!("{state};{}", counter_row(&counters)),
    );
    drop(array);
    let drops = counters
        .iter()
        .map(|entry| entry.drops.get())
        .sum::<usize>();
    assert_native_row(&format!("array1_size_drop_{}", u8::from(reuse)), drops);
}

#[test]
fn array1_allocation_mode_ignores_a_short_buffer() {
    let mut first = probe_buffer::<1>();
    let mut second = probe_buffer::<1>();
    first[0].value = 91;
    second[0].value = 91;
    let bounded = Array1::from_buffer_bounds(&mut first, -1, 2, false).unwrap();
    let sized = Array1::from_buffer_size(&mut second, 5, false).unwrap();
    let sizes = format!("{},{}", bounded.len(), sized.len());
    let hooks = constructed_pair_trace(&bounded, &sized);
    drop((bounded, sized));
    assert_native_row(
        "array1_ignored_short",
        format!("{sizes};{},{};{hooks}", first[0].value, second[0].value),
    );
}

#[test]
fn array1_empty_bounded_constructor_preserves_native_pointer_and_bounds() {
    for reuse in [true, false] {
        let mut buffer = probe_buffer::<1>();
        let pointer = buffer.as_ptr();
        let array = Array1::from_buffer_bounds(&mut buffer, 7, 6, reuse).unwrap();
        let state = format!(
            "{},{};{};{};{};0,0,0",
            array.lower(),
            array.upper(),
            array.len(),
            u8::from(array.is_deletable()),
            u8::from(array.data_ptr() == pointer)
        );
        assert_native_row(&format!("array1_empty_{}", u8::from(reuse)), state);
    }
}

#[test]
fn array2_bounded_constructor_reuses_the_prefix_with_native_flat_bounds() {
    let mut buffer = probe_buffer::<5>();
    let pointer = buffer.as_ptr();
    let mut array = Array2::from_buffer_bounds(&mut buffer, -1, 0, 3, 4).unwrap();
    let prefix = array2_bounds_row(&array, pointer == array.data_ptr());
    array[(-1, 4)].value = 71;
    let counters = capture_counters(array.as_slice());
    let during = counter_row(&counters);
    drop(array);
    assert_native_row(
        "array2_bounds",
        format!("{prefix};{};{during}", values_row(&buffer)),
    );
    assert_native_row("array2_bounds_drop", counters[0].drops.get());
}

fn array2_bounds_row(array: &Array2<'_, BufferItem>, same_pointer: bool) -> String {
    format!(
        "{},{},{},{};{},{};{},{};{};{}",
        array.lower_row(),
        array.upper_row(),
        array.lower_col(),
        array.upper_col(),
        array.nb_rows(),
        array.nb_columns(),
        array.array1().lower(),
        array.array1().upper(),
        u8::from(array.is_deletable()),
        u8::from(same_pointer)
    )
}

#[test]
fn array2_empty_bounded_dimensions_keep_the_nonowning_buffer_pointer() {
    for empty_rows in [true, false] {
        let mut buffer = probe_buffer::<1>();
        let pointer = buffer.as_ptr();
        let (ru, cu) = if empty_rows { (-2, 4) } else { (0, 2) };
        let array = Array2::from_buffer_bounds(&mut buffer, -1, ru, 3, cu).unwrap();
        let state = format!(
            "{},{};{},{};{};{};0,0,0",
            array.nb_rows(),
            array.nb_columns(),
            array.array1().lower(),
            array.array1().upper(),
            u8::from(array.is_deletable()),
            u8::from(array.data_ptr() == pointer)
        );
        assert_native_row(&format!("array2_empty_{}", u8::from(empty_rows)), state);
    }
}

#[test]
fn array1_pointer_prefix_accepts_a_type_without_default_or_clone() {
    struct NoDefaultItem(i32);
    let mut buffer = [NoDefaultItem(1), NoDefaultItem(2), NoDefaultItem(3)];
    let mut array = Array1::from_slice_size(&mut buffer, 2).unwrap();
    array[1].0 = 71;
    let (size, owner) = (array.len(), u8::from(array.is_deletable()));
    drop(array);
    assert_native_row(
        "array1_pointer_prefix",
        format!(
            "{size};{owner};{},{},{}",
            buffer[0].0, buffer[1].0, buffer[2].0
        ),
    );
}

fn constructed_pair_trace(
    first: &Array1<'_, BufferItem>,
    second: &Array1<'_, BufferItem>,
) -> String {
    let mut counters = capture_counters(first.as_slice());
    counters.extend(capture_counters(second.as_slice()));
    assert!(
        first
            .as_slice()
            .iter()
            .chain(second.as_slice())
            .all(|item| item.value == 37)
    );
    counter_row(&counters)
}
