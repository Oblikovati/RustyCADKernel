//! Native protected helpers preserve real topology and the caller's recursive cursor.
use rustycad_tkernel::ncollection::{CellAction, CellFilter, CellInspector};
mod cell_protected_fixture;
use cell_protected_fixture::{Inspector, assert_row, collect_range};

#[test]
fn native_direct_helpers_share_the_public_coordinate_graph() {
    let mut filter = CellFilter::<Inspector>::new(1.0);
    let index = [4, -2, 9];
    for target in [7, 8, 7] {
        filter.add_index(&index, &target);
    }
    let mut inspector = Inspector::default();
    filter.inspect_index(&index, &mut inspector);
    assert_row(0, "direct", &index, &inspector);
    filter.remove_index(&index, &7);
    inspector.seen.clear();
    filter.inspect_index(&index, &mut inspector);
    assert_row(1, "removed", &index, &inspector);
    check_reset(filter, index);
}
fn check_reset(mut filter: CellFilter<Inspector>, index: [i32; 3]) {
    filter.reset_allocator(None);
    let mut inspector = Inspector::default();
    filter.inspect_index(&index, &mut inspector);
    assert_row(2, "reset", &index, &inspector);
    filter.add(&11, &[4., -2., 9.]);
    filter.inspect_index(&index, &mut inspector);
    assert_row(3, "size-kept", &index, &inspector);
}
#[test]
fn native_partial_axes_preserve_higher_caller_coordinates() {
    collect_range(4, "axis0", 0, &mut [90, 91, 92], &[1, 2, 3], &[2, 3, 4]);
    collect_range(6, "axis1", 1, &mut [90, 91, 92], &[1, 2], &[2, 3]);
}
#[test]
fn native_reversed_inner_axis_still_mutates_the_outer_cursor() {
    collect_range(8, "reverse-inner", 1, &mut [90, 91, 92], &[2, 2], &[1, 3]);
    collect_range(10, "reverse-outer", 1, &mut [90, 91, 92], &[1, 3], &[2, 2]);
}
#[test]
fn native_extreme_singletons_do_not_overflow_on_completion() {
    collect_range(12, "max", 0, &mut [90], &[i32::MAX], &[i32::MAX]);
    collect_range(14, "min", 0, &mut [90], &[i32::MIN], &[i32::MIN]);
}
#[test]
fn native_throwing_inspector_retains_cursor_and_earlier_purges() {
    let mut filter = CellFilter::<Inspector>::new(1.0);
    let (min, max, mut cursor) = ([1, 2, 3], [2, 3, 4], [90, 91, 92]);
    filter
        .iterate_add_indices(1, &mut cursor, &min, &max, &5)
        .unwrap();
    cursor = [90, 91, 92];
    let mut inspector =
        cell_protected_fixture::inspect_failure(&mut filter, &mut cursor, &min, &max);
    assert_row(16, "panic", &cursor, &inspector);
    inspector = Inspector::default();
    filter
        .iterate_inspect_indices(1, &mut cursor, &min, &max, &mut inspector)
        .unwrap();
    assert_row(17, "panic-remains", &cursor, &inspector);
}
#[test]
fn invalid_recursion_shapes_fail_before_any_cursor_or_graph_mutation() {
    let mut filter = CellFilter::<Inspector>::new(1.0);
    let mut cursor = [9, 8];
    for axis in [-1, 1, i32::MAX] {
        let failure = filter
            .iterate_add_indices(axis, &mut cursor, &[0], &[1], &7)
            .unwrap_err();
        assert!(failure.message().contains(&format!("axis {axis}")));
        assert!(failure.message().contains("expected 0 <= axis < 1"));
        assert_eq!(cursor, [9, 8]);
    }
    let mut inspector = Inspector::default();
    filter.inspect_index(&cursor, &mut inspector);
    assert!(inspector.seen.is_empty());
}

#[test]
fn native_reversed_outer_range_does_not_access_an_empty_cursor() {
    collect_range(22, "reverse-empty", 0, &mut [], &[2], &[1]);
}

#[test]
fn reached_cursor_bounds_fail_but_unreached_coordinates_are_permitted() {
    let mut filter = CellFilter::<Inspector>::new(1.0);
    let mut cursor = [];
    let failure = filter
        .iterate_add_indices(0, &mut cursor, &[1], &[2], &7)
        .unwrap_err();
    assert!(failure.message().contains("lengths (0, 1, 1)"));
    assert!(failure.message().contains("expected 0 <= axis < 0"));
    let failure = filter
        .iterate_remove_indices(0, &mut cursor, &[], &[], &7)
        .unwrap_err();
    assert!(failure.message().contains("lengths (0, 0, 0)"));
}
