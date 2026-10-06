//! Borrowed assignment and native iterator-rvalue copy behavior, including partial failure.
use rustycad_tkernel::ncollection::{BaseAllocator, Sequence, SequenceCursor};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[path = "sequence_borrowed_assignment_contracts/probe.rs"]
mod probe;
use probe::{AssignmentItem, AssignmentTrace};

fn items(trace: &AssignmentTrace) -> Sequence<AssignmentItem> {
    [1, 2]
        .into_iter()
        .map(|value| AssignmentItem::new(value, trace))
        .collect()
}
fn assignment_rows(rows: &mut Vec<String>) {
    let trace = AssignmentTrace::default();
    let mut sequence = items(&trace);
    let source = AssignmentItem::new(9, &trace);
    let address = std::ptr::from_ref(sequence.value(1).unwrap());
    trace.clear();
    sequence.set_value_ref(1, &source).unwrap();
    rows.push(format!(
        "assign {} {} {}",
        trace.text(),
        sequence.value(1).unwrap().value,
        u8::from(std::ptr::eq(sequence.value(1).unwrap(), address))
    ));
    indexed_assignment_rows(&mut sequence, &trace, rows);
    partial_rows(&mut sequence, source, address, &trace, rows);
}
fn indexed_assignment_rows(
    sequence: &mut Sequence<AssignmentItem>,
    trace: &AssignmentTrace,
    rows: &mut Vec<String>,
) {
    trace.clear();
    sequence
        .set_value_from_index_with(1, 1, AssignmentItem::assign_self)
        .unwrap();
    rows.push(format!("self {}", trace.text()));
    trace.clear();
    sequence
        .set_value_from_index_with(2, 1, AssignmentItem::assign_self)
        .unwrap();
    sequence
        .set_value_from_index_with(1, 2, AssignmentItem::assign_self)
        .unwrap();
    rows.push(format!("indexed {}", trace.text()));
}
fn partial_rows(
    sequence: &mut Sequence<AssignmentItem>,
    mut source: AssignmentItem,
    address: *const AssignmentItem,
    trace: &AssignmentTrace,
    rows: &mut Vec<String>,
) {
    source.value = 7;
    source.fail_assign = true;
    trace.clear();
    let caught = catch_unwind(AssertUnwindSafe(|| sequence.set_value_ref(1, &source))).is_err();
    rows.push(format!(
        "partial_assign {} {} {} {}",
        trace.text(),
        u8::from(caught),
        sequence.value(1).unwrap().value,
        u8::from(std::ptr::eq(sequence.value(1).unwrap(), address))
    ));
    invalid_assignment_row(sequence, &source, trace, rows);
}
fn invalid_assignment_row(
    sequence: &mut Sequence<AssignmentItem>,
    source: &AssignmentItem,
    trace: &AssignmentTrace,
    rows: &mut Vec<String>,
) {
    trace.clear();
    let errors = [-1, 0, 3]
        .into_iter()
        .filter(|index| sequence.set_value_ref_signed(*index, source).is_err())
        .count();
    rows.push(format!("invalid {errors} {}", trace.text().len()));
}
fn cursor_rows(rows: &mut Vec<String>) {
    let trace = AssignmentTrace::default();
    let mut sequence = items(&trace);
    let cursor = sequence.cursor();
    let mut source = AssignmentItem::new(8, &trace);
    trace.clear();
    sequence
        .insert_after_cursor_rvalue(&cursor, &mut source)
        .unwrap();
    rows.push(format!(
        "cursor_rvalue {} {} {} {} {}",
        trace.text(),
        source.value,
        sequence.size(),
        sequence.value(2).unwrap().value,
        cursor.value(&sequence).unwrap().value
    ));
    empty_cursor_row(&mut source, &trace, rows);
    copy_failure_row(&mut sequence, &cursor, &mut source, &trace, rows);
}
fn empty_cursor_row(source: &mut AssignmentItem, trace: &AssignmentTrace, rows: &mut Vec<String>) {
    let mut empty = Sequence::default();
    let end = empty.cursor();
    trace.clear();
    empty.insert_after_cursor_rvalue(&end, source).unwrap();
    rows.push(format!(
        "empty_rvalue {} {} {} {}",
        trace.text(),
        source.value,
        empty.first().unwrap().value,
        u8::from(end.more())
    ));
}
fn copy_failure_row(
    sequence: &mut Sequence<AssignmentItem>,
    cursor: &SequenceCursor,
    source: &mut AssignmentItem,
    trace: &AssignmentTrace,
    rows: &mut Vec<String>,
) {
    source.fail_copy = true;
    trace.clear();
    let caught = failed_cursor_copy(sequence, cursor, source);
    rows.push(format!(
        "copy_failure {} {} {} {}",
        trace.text(),
        u8::from(caught),
        source.value,
        sequence.size()
    ));
}
fn failed_cursor_copy(
    sequence: &mut Sequence<AssignmentItem>,
    cursor: &SequenceCursor,
    source: &mut AssignmentItem,
) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        sequence.insert_after_cursor_rvalue(cursor, source).unwrap();
    }))
    .is_err()
}
#[test]
fn borrowed_assignment_and_cursor_rvalue_match_native_rows() {
    let mut rows = Vec::new();
    assignment_rows(&mut rows);
    cursor_rows(&mut rows);
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!(
            "../../../tools/oracle/tkernel_sequence_borrowed_assignment_contracts.expected"
        )
    );
}
#[test]
fn signed_borrowed_assignment_preserves_node_and_uses_copy_assignment() {
    let trace = AssignmentTrace::default();
    let mut sequence = items(&trace);
    let cursor = sequence.cursor();
    trace.clear();
    sequence
        .set_value_ref_signed(1, &AssignmentItem::new(4, &trace))
        .unwrap();
    assert_eq!(trace.text(), "A1>4;D4;");
    assert_eq!(cursor, sequence.cursor());
    assert_eq!(sequence.value(1).unwrap().value, 4);
}
#[test]
fn indexed_assignment_calls_self_hook_only_for_the_identical_node() {
    let mut sequence: Sequence<i32> = [2, 8].into_iter().collect();
    sequence
        .set_value_from_index_with_signed(1, 2, |_| panic!("distinct node is not self"))
        .unwrap();
    sequence
        .set_value_from_index_with_signed(2, 2, |value| *value += 1)
        .unwrap();
    assert_eq!(sequence.iter().copied().collect::<Vec<_>>(), [8, 9]);
}
#[test]
fn indexed_assignment_validates_source_before_destination_and_skips_hook() {
    let mut sequence: Sequence<i32> = [2].into_iter().collect();
    for (destination, source) in [(-2, -1), (-2, 2), (2, 1), (0, 1)] {
        let error = sequence
            .set_value_from_index_with_signed(destination, source, |_| {
                panic!("invalid index reached self hook")
            })
            .unwrap_err();
        assert!(
            error
                .message()
                .contains(if source != 1 { "Value" } else { "ChangeValue" })
        );
    }
    assert_eq!(sequence.first(), Some(&2));
}
#[test]
fn self_assignment_panic_keeps_partial_item_and_sequence_ownership() {
    let mut sequence: Sequence<i32> = [1, 2].into_iter().collect();
    let cursor = sequence.cursor();
    let failure = catch_unwind(AssertUnwindSafe(|| {
        sequence.set_value_from_index_with(1, 1, |value| {
            *value = 9;
            panic!("injected self assignment failure");
        })
    }));
    assert!(failure.is_err());
    assert_eq!(sequence.iter().copied().collect::<Vec<_>>(), [9, 2]);
    assert_eq!(sequence.cursor(), cursor);
}
#[test]
fn rvalue_cursor_copy_failure_releases_reserved_node_allocation() {
    let arena = BaseAllocator::incremental(4096);
    let mut sequence = Sequence::with_allocator(arena.clone());
    let trace = AssignmentTrace::default();
    let mut source = AssignmentItem::new(8, &trace);
    source.fail_copy = true;
    let failure = catch_unwind(AssertUnwindSafe(|| {
        sequence
            .insert_after_cursor_rvalue(&SequenceCursor::default(), &mut source)
            .unwrap();
    }));
    assert!(failure.is_err());
    assert!(sequence.is_empty());
    assert_eq!(source.value, 8);
    assert!(arena.reset(false).is_ok());
}
