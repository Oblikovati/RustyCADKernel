//! Native throwing copy and equality callbacks leave earlier cells committed.
use super::{CellAction, CellFilter, CellInspector, Inspector, assert_row};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
#[derive(Default)]
struct FailureState {
    copies: Cell<usize>,
    comparisons: Cell<usize>,
    fail_copy: Cell<usize>,
    fail_compare: Cell<usize>,
}
struct Target(Rc<FailureState>);
impl Clone for Target {
    fn clone(&self) -> Self {
        self.0.copies.set(self.0.copies.get() + 1);
        assert_ne!(
            self.0.copies.get(),
            self.0.fail_copy.get(),
            "native copy failure"
        );
        Self(Rc::clone(&self.0))
    }
}
#[derive(Default)]
struct FailureInspector(Inspector);
impl CellInspector for FailureInspector {
    type Target = Target;
    type Point = [f64; 3];
    const DIMENSION: usize = 3;
    fn coord(axis: usize, point: &Self::Point) -> f64 {
        point[axis]
    }
    fn is_equal(first: &Target, _: &Target) -> bool {
        first.0.comparisons.set(first.0.comparisons.get() + 1);
        assert_ne!(
            first.0.comparisons.get(),
            first.0.fail_compare.get(),
            "native equality failure"
        );
        true
    }
    fn inspect(&mut self, _: &Target) -> CellAction {
        self.0.seen.push(5);
        CellAction::Keep
    }
}
fn prepared_failure(removal: bool) -> (CellFilter<FailureInspector>, Target) {
    let target = Target(Rc::default());
    let mut filter = CellFilter::new(1.0);
    if removal {
        filter
            .iterate_add_indices(1, &mut [90, 91, 92], &[1, 2, 3], &[2, 3, 4], &target)
            .unwrap();
    }
    target.0.fail_copy.set(if removal { 0 } else { 3 });
    target.0.fail_compare.set(if removal { 3 } else { 0 });
    (filter, target)
}
fn invoke_failure(
    filter: &mut CellFilter<FailureInspector>,
    target: &Target,
    cursor: &mut [i32],
    removal: bool,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if removal {
            filter
                .iterate_remove_indices(1, cursor, &[1, 2, 3], &[2, 3, 4], target)
                .unwrap();
        } else {
            filter
                .iterate_add_indices(1, cursor, &[1, 2, 3], &[2, 3, 4], target)
                .unwrap();
        }
    }));
    assert!(result.is_err());
}
fn compare_failure(removal: bool, row: usize, labels: [&str; 2]) {
    let (mut filter, target) = prepared_failure(removal);
    let mut cursor = [90, 91, 92];
    invoke_failure(&mut filter, &target, &mut cursor, removal);
    let mut probe = FailureInspector::default();
    assert_row(row, labels[0], &cursor, &probe.0);
    filter
        .iterate_inspect_indices(1, &mut cursor, &[1, 2, 3], &[2, 3, 4], &mut probe)
        .unwrap();
    assert_row(row + 1, labels[1], &cursor, &probe.0);
}
#[test]
fn native_copy_failure_keeps_prior_cells_and_current_cursor() {
    compare_failure(false, 18, ["copy-failure", "copy-remains"]);
}
#[test]
fn native_equality_failure_keeps_prior_removals_and_current_cursor() {
    compare_failure(true, 20, ["equal-failure", "equal-remains"]);
}
