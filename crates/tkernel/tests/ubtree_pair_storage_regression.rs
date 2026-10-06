//! Existing public entry points must use one real pair and native owner destruction order.
use rustycad_tkernel::ncollection::{BaseAllocator, BoundingVolume, UBTree};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone)]
struct RecordedBounds(i32, Rc<RefCell<Vec<i32>>>);
impl BoundingVolume for RecordedBounds {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
impl Drop for RecordedBounds {
    fn drop(&mut self) {
        self.1.borrow_mut().push(self.0);
    }
}
fn fixture() -> (UBTree<i32, RecordedBounds>, Rc<RefCell<Vec<i32>>>) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut tree = UBTree::with_allocator(BaseAllocator::accumulating(1024).unwrap());
    tree.add(1, RecordedBounds(10, trace.clone()));
    tree.gemmate(
        0,
        RecordedBounds(99, trace.clone()),
        2,
        RecordedBounds(20, trace.clone()),
    );
    trace.borrow_mut().clear();
    (tree, trace)
}
#[test]
fn owned_gemmate_allocates_one_contiguous_pair() {
    let (tree, _) = fixture();
    let live: i32 = tree
        .allocator()
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count)
        .sum();
    assert_eq!(live, 2, "root plus one actual pair allocation");
    let first = tree.child(tree.root().unwrap(), 0).unwrap();
    let second = tree.child(tree.root().unwrap(), 1).unwrap();
    assert_eq!(
        std::ptr::from_ref(second).addr() - std::ptr::from_ref(first).addr(),
        std::mem::size_of_val(first)
    );
}
#[test]
fn owner_drop_walks_nested_nodes_in_native_preorder() {
    let (mut tree, trace) = fixture();
    tree.gemmate(
        0,
        RecordedBounds(100, trace.clone()),
        3,
        RecordedBounds(30, trace.clone()),
    );
    trace.borrow_mut().clear();
    drop(tree);
    assert_eq!(*trace.borrow(), [100, 99, 10, 20, 30]);
}
