use super::super::*;
fn assignment_header(saved: &MutableCursor, target: &DynamicArray<i32>) -> String {
    let mut assigned = MutableCursor::default();
    let address = std::ptr::from_ref(&assigned);
    let returned = std::ptr::from_mut(assigned.assign_from(saved)).cast_const() == address;
    let pointer = assigned.pointer();
    let identical = std::ptr::from_mut(assigned.assign_self()).cast_const() == address;
    format!(
        "assign value={} target={} returned={} self={} unchanged={}\n",
        cached_value(&assigned),
        u8::from(pointer == std::ptr::from_ref(target.value(1).unwrap())),
        u8::from(returned),
        u8::from(identical),
        u8::from(pointer == assigned.pointer())
    ) + &constant_assignment_row(saved)
}
fn constant_assignment_row(saved: &MutableCursor) -> String {
    let mut constant = SharedCursor::default();
    let address = std::ptr::from_ref(&constant);
    let returned = std::ptr::from_mut(constant.assign_from_mutable(saved)).cast_const() == address;
    format!(
        "const_assign value={} same={} returned={}\n",
        cached_value(&constant),
        u8::from(constant.pointer() == saved.pointer()),
        u8::from(returned)
    )
}
fn assignment_rows(
    source: &mut DynamicArray<i32>,
    target: &DynamicArray<i32>,
    saved: &MutableCursor,
) -> String {
    let mut rows = assignment_header(saved, target);
    for value in 101..=106 {
        source.append(value);
    }
    let fresh = SharedCursor::from_shared(source, 1).unwrap();
    rows += &comparison_row("same_owner_new_pointer", saved, &fresh);
    let moved = SharedCursor::from_shared(target, 1).unwrap();
    rows += &comparison_row("different_owner_same_pointer", saved, &moved);
    rows + &assigned_end_row(source, saved)
}
fn assigned_end_row(source: &mut DynamicArray<i32>, saved: &MutableCursor) -> String {
    let mut assigned = MutableCursor::default();
    assigned.assign_from(saved);
    let mut constant = SharedCursor::default();
    constant.assign_from_mutable(saved);
    assigned.offset_with(source, 3).unwrap();
    constant.offset_with(source, 3).unwrap();
    let end = SharedCursor::from_shared(source, source.len()).unwrap();
    format!(
        "assigned_end mutable={} constant={} difference={}\n",
        u8::from(assigned.more()),
        u8::from(constant.more()),
        assigned.positional_difference(&end).unwrap()
    )
}
fn initialization_rows(source: &mut DynamicArray<i32>, target: &mut DynamicArray<i32>) -> String {
    let mut cursor = MutableCursor::from_mutable(target, 0).unwrap();
    cursor.initialize_with(source).unwrap();
    let mut rows = initialized_row("init_mutable", "source", &cursor, source);
    cursor.initialize_with(target).unwrap();
    rows += &initialized_row("initialize_other", "target", &cursor, target);
    rows += &initialized_write_row(&mut cursor, source, "init_const_argument", 201);
    rows += &initialized_write_row(&mut cursor, source, "initialize_const_argument", 301);
    let mut read_only = SharedCursor::from_shared(target, 0).unwrap();
    read_only.initialize_with(source).unwrap();
    rows += &format!("const_init value={}\n", cached_value(&read_only));
    read_only.initialize_with(target).unwrap();
    rows += &format!("const_initialize value={}\n", cached_value(&read_only));
    rows
}
fn initialized_row(
    label: &str,
    owner: &str,
    cursor: &MutableCursor,
    source: &DynamicArray<i32>,
) -> String {
    format!(
        "{label} value={} {owner}={}\n",
        cached_value(cursor),
        u8::from(cursor.pointer() == std::ptr::from_ref(source.value(0).unwrap()))
    )
}
fn initialized_write_row(
    cursor: &mut MutableCursor,
    source: &mut DynamicArray<i32>,
    label: &str,
    value: i32,
) -> String {
    cursor.initialize_with(source).unwrap();
    // The fresh capture has writable provenance and no overlapping slot reference.
    unsafe {
        *cursor.change_value() = value;
    }
    format!("{label} write={}\n", source.value(0).unwrap())
}
fn end_snapshot_rows(source: &mut DynamicArray<i32>) -> String {
    let mut cursor = SharedCursor::default();
    cursor.initialize_with(source).unwrap();
    source.append(107);
    cursor.offset_with(source, 6).unwrap();
    let end = SharedCursor::from_shared(source, source.len()).unwrap();
    let rows = format!(
        "initialized_end more={} difference={}\n",
        u8::from(cursor.more()),
        cursor.positional_difference(&end).unwrap()
    );
    rows + &recaptured_end_rows(source, &mut cursor) + &empty_initialization_row(&mut cursor)
}
fn recaptured_end_rows(source: &DynamicArray<i32>, cursor: &mut SharedCursor) -> String {
    cursor.initialize_with(source).unwrap();
    cursor.offset_with(source, 6).unwrap();
    format!(
        "reinitialized_end more={} value={}\n",
        u8::from(cursor.more()),
        cached_value(cursor)
    )
}
fn empty_initialization_row(cursor: &mut SharedCursor) -> String {
    let empty = DynamicArray::with_increment(2);
    cursor.initialize_with(&empty).unwrap();
    let begin = SharedCursor::from_shared(&empty, 0).unwrap();
    format!(
        "empty_init more={} same={}\n",
        u8::from(cursor.more()),
        u8::from(*cursor == begin)
    )
}
fn comparison_rows(source: &mut DynamicArray<i32>, target: &DynamicArray<i32>) -> String {
    let begin = MutableCursor::from_mutable(source, 0).unwrap();
    let other = SharedCursor::from_shared(target, 0).unwrap();
    let end = SharedCursor::from_shared(source, source.len()).unwrap();
    let mut rows = comparison_row("cross_owner_zero", &begin, &other);
    rows += &comparison_row("cross_const_before", &begin, &end);
    rows += &comparison_row("cross_const_after", &end, &begin);
    rows + &signed_minimum_rows(source, &begin)
}
fn signed_minimum_rows(source: &DynamicArray<i32>, begin: &MutableCursor) -> String {
    let negative = SharedCursor::from_shared(source, isize::MIN as usize).unwrap();
    let mut rows = format!(
        "signed_min_left lt={} ge={}\n",
        u8::from(negative.less_than(begin).unwrap()),
        u8::from(negative.at_least(begin).unwrap())
    );
    rows += &format!(
        "signed_min_right gt={} le={}\n",
        u8::from(begin.greater_than(&negative).unwrap()),
        u8::from(begin.at_most(&negative).unwrap())
    );
    rows
}
pub(crate) fn trace() -> String {
    let mut source = source_array();
    let saved = MutableCursor::from_mutable(&mut source, 1).unwrap();
    let mut target = DynamicArray::from_moved(&mut source);
    let mut empty_capture = MutableCursor::from_mutable(&mut target, 0).unwrap();
    empty_capture.initialize_with(&mut source).unwrap();
    let mut rows = format!("moved_empty_init more={}\n", u8::from(empty_capture.more()));
    rows += &assignment_rows(&mut source, &target, &saved);
    rows += &initialization_rows(&mut source, &mut target);
    rows += &end_snapshot_rows(&mut source);
    rows += &comparison_rows(&mut source, &target);
    rows
}
