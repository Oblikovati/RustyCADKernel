use super::CellIndex;
use crate::ncollection::{Array1, CellGeometry};
fn row(number: usize, actual: String) {
    let expected =
        include_str!("../../../../../../tools/oracle/tkernel_cell_index_native_storage.expected");
    assert_eq!(actual, expected.lines().nth(number).unwrap());
}
#[test]
#[cfg(target_pointer_width = "64")]
fn concrete_cell_index_field_sizes_and_offsets_match_native_storage() {
    row(
        0,
        format!(
            "index {} {} {} {} {}",
            std::mem::size_of::<CellIndex>(),
            std::mem::align_of::<CellIndex>(),
            std::mem::offset_of!(CellIndex, inline),
            std::mem::offset_of!(CellIndex, heap),
            std::mem::offset_of!(CellIndex, length)
        ),
    );
}
#[test]
fn synthesized_inline_pointer_stays_valid_after_ordinary_rust_relocation() {
    let source = CellIndex::from_slice(&[3, 4, 5]);
    assert!(source.heap.is_none());
    let mut relocated = Box::new(source);
    assert_eq!(relocated.data_ptr(), relocated.inline.as_ptr());
    relocated.as_mut_slice()[1] = 9;
    assert_eq!(relocated.as_slice(), [3, 9, 5]);
    let moved = *relocated;
    assert_eq!(moved.data_ptr(), moved.inline.as_ptr());
    assert_eq!(moved.as_slice(), [3, 9, 5]);
}
#[test]
fn empty_heap_ownership_survives_move_copy_and_assignment_like_native() {
    let mut first = CellIndex::from_slice(&[1; 11]);
    let mut second = CellIndex::from_slice(&[2; 24]);
    let retained = second.data_ptr();
    second.move_from(&mut first);
    row(
        2,
        format!(
            "empty-donor {} {} {}",
            first.length,
            u8::from(first.heap.is_none()),
            u8::from(first.data_ptr() == retained)
        ),
    );
    let mut moved = check_empty_move(&mut first, retained);
    check_empty_assignment(&mut moved, retained);
}
fn check_empty_move(first: &mut CellIndex, retained: *const i32) -> CellIndex {
    let moved = CellIndex::move_construct(first);
    row(
        3,
        format!(
            "move-empty {} {} {} {}",
            moved.length,
            u8::from(moved.heap.is_none()),
            u8::from(first.heap.is_none()),
            u8::from(moved.data_ptr() == retained)
        ),
    );
    check_empty_copy(&moved);
    moved
}
fn check_empty_copy(moved: &CellIndex) {
    let lookup = moved.clone();
    row(
        4,
        format!(
            "copy-empty {} {} {}",
            lookup.length,
            u8::from(lookup.heap.is_none()),
            u8::from(moved.heap.is_none())
        ),
    );
}
fn check_empty_assignment(source: &mut CellIndex, retained: *const i32) {
    let mut receiver = CellIndex::from_slice(&[3; 11]);
    let old = receiver.data_ptr();
    receiver.move_from(source);
    row(
        5,
        format!(
            "assign-empty {} {} {} {} {}",
            receiver.length,
            u8::from(receiver.heap.is_none()),
            u8::from(source.heap.is_none()),
            u8::from(receiver.data_ptr() == retained),
            u8::from(source.data_ptr() == old)
        ),
    );
    check_return_to_inline(receiver);
}
fn check_return_to_inline(mut receiver: CellIndex) {
    let mut inline_source = CellIndex::from_slice(&[4; 3]);
    receiver.move_from(&mut inline_source);
    row(
        6,
        format!(
            "return-inline {} {} {} {}",
            receiver.length,
            u8::from(receiver.heap.is_none()),
            inline_source.length,
            u8::from(inline_source.heap.is_none())
        ),
    );
}
struct FailingCoordinates;
impl CellGeometry for FailingCoordinates {
    type CellTarget = i32;
    type CellPoint = ();
    const CELL_DIMENSION: usize = 11;
    fn cell_coordinate(axis: usize, _: &()) -> f64 {
        assert_ne!(axis, 5, "axis5: expected a successful coordinate callback");
        axis as f64
    }
    fn targets_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
}
#[test]
fn partial_heap_index_construction_releases_its_allocation_on_callback_panic() {
    let sizes = Array1::with_bounds(0, 10).unwrap();
    let failed =
        std::panic::catch_unwind(|| CellIndex::from_point::<FailingCoordinates>(&(), &sizes));
    assert!(failed.is_err());
    let succeeding = CellIndex::from_slice(&[7; 11]);
    assert_eq!(succeeding.as_slice(), [7; 11]);
}
#[test]
fn unrepresentable_cell_index_byte_request_reports_its_offending_length() {
    let failed = std::panic::catch_unwind(|| CellIndex::allocate(usize::MAX));
    let panic = failed.unwrap_err();
    let message = panic.downcast_ref::<String>().unwrap();
    assert!(message.contains(&usize::MAX.to_string()));
    assert!(message.contains("expected byte size representable by usize"));
}

#[test]
fn default_cell_index_state_is_empty_inline_storage() {
    let empty = CellIndex::allocate(0);
    row(
        7,
        format!(
            "default {} {}",
            empty.length,
            u8::from(empty.heap.is_none())
        ),
    );
    assert_eq!(empty.data_ptr(), empty.inline.as_ptr());
}
