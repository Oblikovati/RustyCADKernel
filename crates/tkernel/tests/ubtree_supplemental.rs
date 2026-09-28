//! Native traversal fixtures and selector edge semantics.
use rustycad_tkernel::ncollection::{BaseAllocator, BoundingVolume, EBTree, TreeSelector, UBTree};
include!("ncollection_ubtree/support.rs");

#[test]
fn native_traversal_order_survives_removals() {
    let mut tree: EBTree<i32, TestBnd1D> = EBTree::default();
    let bounds: [(f64, f64); 6] = [(0., 2.), (3., 5.), (4., 6.), (8., 10.), (-2., 9.), (1., 1.)];
    for (index, (min, max)) in bounds.into_iter().enumerate() {
        tree.add(index as i32 + 1, TestBnd1D::new(min, max));
    }
    let mut before = TestSelector1D::new(TestBnd1D::new(-10., 20.));
    assert_eq!(tree.select(&mut before), 6);
    assert_eq!(before.results, [1, 6, 5, 2, 3, 4]);
    tree.remove(&2);
    tree.remove(&5);
    let mut after = TestSelector1D::new(TestBnd1D::new(-10., 20.));
    assert_eq!(tree.select(&mut after), 4);
    assert_eq!(after.results, [1, 6, 3, 4]);
    assert!(tree.find_node(&2).is_err());
    tree.clear();
    assert!(tree.is_empty());
    assert!(!tree.contains(&1));
}

struct RejectingStop {
    visits: usize,
}
impl TreeSelector<i32, TestBnd1D> for RejectingStop {
    fn reject(&self, _: &TestBnd1D) -> bool {
        false
    }
    fn accept(&mut self, _: &i32) -> bool {
        self.visits += 1;
        false
    }
    fn stop(&self) -> bool {
        true
    }
}
#[test]
fn selector_stop_is_checked_after_rejected_acceptance() {
    let resource: BaseAllocator = BaseAllocator::incremental(1024);
    let mut tree: UBTree<i32, TestBnd1D> = UBTree::with_allocator(resource.clone());
    assert!(tree.root().is_none());
    tree.add(1, TestBnd1D::new(0., 1.));
    assert!(tree.root().unwrap().is_root());
    assert_eq!(tree.root().unwrap().bounds().max, 1.);
    tree.add(2, TestBnd1D::new(2., 3.));
    let mut selector = RejectingStop { visits: 0 };
    assert_eq!(tree.select(&mut selector), 0);
    assert_eq!(selector.visits, 1);
    assert_eq!(tree.allocator(), &resource);
    let mut stopping = TestSelector1D::stopping(TestBnd1D::new(0., 3.), 1);
    assert_eq!(tree.select(&mut stopping), 1);
    tree.clear();
    assert!(tree.is_empty());
    assert_eq!(tree.allocator(), &resource);
}
