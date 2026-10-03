//! Native selector exceptions must terminate traversal before another callback.
use rustycad_tkernel::ncollection::{BoundingVolume, FallibleTreeSelector, UBTree};
use std::cell::Cell;
#[derive(Clone)]
struct TestBounds;
impl BoundingVolume for TestBounds {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.
    }
}
#[derive(Default)]
struct SelectionProbe {
    visits: Vec<i32>,
    fail_on: Option<i32>,
    stop_after: Option<usize>,
    rejected: bool,
    stop_polls: Cell<usize>,
}
impl FallibleTreeSelector<i32, TestBounds> for SelectionProbe {
    type Error = i32;
    fn reject(&self, _: &TestBounds) -> bool {
        self.rejected
    }
    fn accept(&mut self, object: &i32) -> Result<bool, i32> {
        self.visits.push(*object);
        if self.fail_on == Some(*object) {
            return Err(*object);
        }
        Ok(true)
    }
    fn stop(&self) -> bool {
        self.stop_polls.set(self.stop_polls.get() + 1);
        self.stop_after == Some(self.visits.len())
    }
}
fn tree() -> UBTree<i32, TestBounds> {
    let mut tree = UBTree::default();
    for value in 1..=3 {
        tree.add(value, TestBounds);
    }
    tree
}
#[test]
fn error_aborts_without_polling_stop_or_visiting_more_leaves() {
    let tree = tree();
    let mut baseline = SelectionProbe::default();
    assert_eq!(tree.try_select(&mut baseline), Ok(3));
    let mut failing = SelectionProbe {
        fail_on: Some(baseline.visits[1]),
        ..SelectionProbe::default()
    };
    assert_eq!(tree.try_select(&mut failing), Err(baseline.visits[1]));
    assert_eq!(failing.visits, baseline.visits[..2]);
    assert_eq!(failing.stop_polls.get(), 1);
}
#[test]
fn stop_and_branch_rejection_keep_native_selection_rules() {
    let tree = tree();
    let mut stopped = SelectionProbe {
        stop_after: Some(1),
        ..SelectionProbe::default()
    };
    assert_eq!(tree.try_select(&mut stopped), Ok(1));
    let mut rejected = SelectionProbe {
        rejected: true,
        ..SelectionProbe::default()
    };
    assert_eq!(tree.try_select(&mut rejected), Ok(0));
    assert!(rejected.visits.is_empty());
    assert_eq!(rejected.stop_polls.get(), 0);
    assert_eq!(
        UBTree::<i32, TestBounds>::default().try_select(&mut stopped),
        Ok(0)
    );
}
