//! Supplemental UBTree and filler callback/state method contracts.
use rustycad_tkernel::ncollection::{
    BoundingTreeInsertion, BoundingVolume, ObjBnd, TreeSelector, UBTree, UBTreeFiller,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
struct Bounds {
    low: f64,
    high: f64,
    checks: Rc<RefCell<Vec<f64>>>,
}
impl Bounds {
    fn interval(low: f64, high: f64, checks: &Rc<RefCell<Vec<f64>>>) -> Self {
        Self {
            low,
            high,
            checks: Rc::clone(checks),
        }
    }
}
impl BoundingVolume for Bounds {
    fn add(&mut self, other: &Self) {
        self.low = self.low.min(other.low);
        self.high = self.high.max(other.high);
    }
    fn is_out(&self, other: &Self) -> bool {
        self.checks.borrow_mut().push(self.low);
        self.high < other.low || other.high < self.low
    }
    fn square_extent(&self) -> f64 {
        (self.high - self.low).powi(2)
    }
}
#[derive(Default)]
struct Collect {
    values: Vec<i32>,
    stopping: bool,
}
impl TreeSelector<i32, Bounds> for Collect {
    fn reject(&self, _: &Bounds) -> bool {
        false
    }
    fn accept(&mut self, object: &i32) -> bool {
        self.values.push(*object);
        true
    }
    fn stop(&self) -> bool {
        self.stopping
    }
}
fn populated() -> UBTree<i32, Bounds> {
    let mut tree = UBTree::default();
    let checks = Rc::new(RefCell::new(Vec::new()));
    tree.add(1, Bounds::interval(0., 10., &checks));
    tree.add(2, Bounds::interval(1., 2., &checks));
    tree.add(3, Bounds::interval(1.5, 1.6, &checks));
    tree
}

#[test]
fn ubtree_add_reuses_selected_child_is_out_result() {
    let checks = Rc::new(RefCell::new(Vec::new()));
    let mut tree = UBTree::default();
    tree.add(1, Bounds::interval(0., 10., &checks));
    tree.add(2, Bounds::interval(1., 2., &checks));
    checks.borrow_mut().clear();
    tree.add(3, Bounds::interval(1.5, 1.6, &checks));
    assert_eq!(*checks.borrow(), [0., 0., 1.]);
    assert_eq!(tree.last_node().unwrap().object(), &3);
}

#[test]
fn ubtree_branch_select_accepts_dynamic_selector_and_native_order() {
    let tree = populated();
    let root = tree.root().unwrap();
    let child = root.child_id(1).unwrap();
    let mut selected = Collect::default();
    let dynamic: &mut dyn TreeSelector<i32, Bounds> = &mut selected;
    assert_eq!(tree.select_branch(child, dynamic), Ok(2));
    assert_eq!(selected.values, [2, 3]);
    selected.values.clear();
    assert_eq!(tree.select(&mut selected), 3);
    assert_eq!(selected.values, [1, 2, 3]);
    assert!(tree.select_branch(usize::MAX, &mut selected).is_err());
}

#[test]
fn ubtree_branch_select_checks_stop_after_acceptance() {
    let tree = populated();
    let mut selected = Collect {
        values: Vec::new(),
        stopping: true,
    };
    assert_eq!(
        tree.select_branch(tree.root_id().unwrap(), &mut selected),
        Ok(1)
    );
    assert_eq!(selected.values, [1]);
}

#[test]
fn ubtree_last_node_and_returning_move_transfer_stable_storage() {
    let mut source = populated();
    source.change_last_node().unwrap().bounds_mut().high = 2.5;
    let pointer = source.last_node().unwrap() as *const _;
    let mut target = UBTree::default();
    let destination = &raw const target;
    assert_eq!(target.move_assign(&mut source) as *const _, destination);
    assert_eq!(target.last_node().unwrap() as *const _, pointer);
    assert!(source.last_node().is_none());
    assert_eq!(target.last_node().unwrap().bounds().high, 2.5);
    assert_eq!(target.move_assign_self() as *const _, destination);
    let mut moved = UBTree::move_from(&mut target);
    assert_eq!(moved.last_node().unwrap() as *const _, pointer);
    moved.clear();
    assert!(moved.last_node().is_none());
    assert!(moved.change_last_node().is_none());
}

#[derive(Default)]
struct TraceObject {
    value: i32,
    events: Rc<RefCell<Vec<&'static str>>>,
}
impl Clone for TraceObject {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push("object-copy");
        Self {
            value: self.value,
            events: Rc::clone(&self.events),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.events.borrow_mut().push("object-assign");
        self.value = source.value;
    }
}
#[derive(Default)]
struct TraceBounds {
    events: Rc<RefCell<Vec<&'static str>>>,
}
impl Clone for TraceBounds {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push("bounds-copy");
        Self {
            events: Rc::clone(&self.events),
        }
    }
    fn clone_from(&mut self, _: &Self) {
        self.events.borrow_mut().push("bounds-assign");
    }
}
impl BoundingVolume for TraceBounds {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.
    }
}
struct RecordingTree {
    tree: UBTree<TraceObject, TraceBounds>,
    visits: Vec<i32>,
}
impl BoundingTreeInsertion<TraceObject, TraceBounds> for RecordingTree {
    fn insert_bound(&mut self, _: TraceObject, _: TraceBounds) -> bool {
        panic!("expected borrowed virtual dispatch");
    }
    fn insert_bound_ref(&mut self, object: &TraceObject, _: &TraceBounds) -> bool {
        self.visits.push(object.value);
        true
    }
    fn tree_view(&self) -> &UBTree<TraceObject, TraceBounds> {
        &self.tree
    }
}
fn traced_pair(value: i32, events: &Rc<RefCell<Vec<&'static str>>>) -> (TraceObject, TraceBounds) {
    (
        TraceObject {
            value,
            events: Rc::clone(events),
        },
        TraceBounds {
            events: Rc::clone(events),
        },
    )
}

#[test]
fn ubtree_filler_pair_copy_and_assignment_use_native_member_order() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let (object, bounds) = traced_pair(1, &events);
    let first = ObjBnd::new_ref(&object, &bounds);
    assert_eq!(*events.borrow(), ["object-copy", "bounds-copy"]);
    events.borrow_mut().clear();
    let mut copied = first.clone();
    assert_eq!(*events.borrow(), ["object-copy", "bounds-copy"]);
    events.borrow_mut().clear();
    copied.clone_from(&first);
    assert_eq!(*events.borrow(), ["object-assign", "bounds-assign"]);
}

#[test]
fn ubtree_filler_dispatches_borrowed_add_and_assigns_selected_slot() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut recording = RecordingTree {
        tree: UBTree::default(),
        visits: Vec::new(),
    };
    {
        let mut filler = UBTreeFiller::new(&mut recording, None, true);
        for value in 1..=3 {
            let (object, bounds) = traced_pair(value, &events);
            filler.add_ref(&object, &bounds);
        }
        events.borrow_mut().clear();
        assert_eq!(filler.fill(), 3);
        assert_eq!(*events.borrow(), ["object-assign", "bounds-assign"]);
        assert_eq!(filler.fill(), 0);
    }
    assert_eq!(recording.visits, [3, 1, 2]);
}

#[test]
fn ubtree_filler_borrowed_enqueue_reset_and_drop_dispatch() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut recording = RecordingTree {
        tree: UBTree::default(),
        visits: Vec::new(),
    };
    {
        let mut filler = UBTreeFiller::new(&mut recording, None, false);
        let (object, bounds) = traced_pair(7, &events);
        filler.add_ref(&object, &bounds);
        assert_eq!(*events.borrow(), ["object-copy", "bounds-copy"]);
        filler.reset();
        let (object, bounds) = traced_pair(8, &events);
        filler.add_ref(&object, &bounds);
    }
    assert_eq!(recording.visits, [8]);
}
