//! Members completed for the NCollection tree family: UBTree node links, Gemmate/Kill,
//! allocator-swapping Clear, UBTreeFiller, CellFilter constructors/Reset and KDTree
//! Array1 builds and moves.
use rustycad_tkernel::ncollection::{
    Array1, BaseAllocator, BoundingNode, BoundingVolume, CellAction, CellFilter, CellInspector,
    EBTree, KDTree, UBTree, UBTreeFiller,
};

#[derive(Clone, Default, Debug, PartialEq)]
struct Span {
    min: f64,
    max: f64,
}
fn span(min: f64, max: f64) -> Span {
    Span { min, max }
}
impl BoundingVolume for Span {
    fn add(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
    fn is_out(&self, other: &Self) -> bool {
        other.min > self.max || other.max < self.min
    }
    fn square_extent(&self) -> f64 {
        (self.max - self.min) * (self.max - self.min)
    }
}
type Tree = UBTree<i32, Span>;

fn leaves(tree: &Tree, node: &BoundingNode<i32, Span>, out: &mut Vec<i32>) {
    if node.is_leaf() {
        out.push(*node.object());
    }
    for index in 0..2 {
        if let Some(child) = tree.child(node, index) {
            leaves(tree, child, out);
        }
    }
}
fn leaf_order(tree: &Tree) -> Vec<i32> {
    let mut out: Vec<i32> = Vec::new();
    if let Some(root) = tree.root() {
        leaves(tree, root, &mut out);
    }
    out
}

#[test]
fn node_links_resolve_through_the_tree() {
    let mut tree: Tree = UBTree::default();
    tree.add(1, span(0.0, 1.0));
    tree.add(2, span(5.0, 6.0));
    let root_id: usize = tree.root_id().unwrap();
    let root: &BoundingNode<i32, Span> = tree.root().unwrap();
    assert_eq!(root.bounds(), &span(0.0, 6.0));
    assert_eq!(*root.object(), 0); // branch objects are nullified
    let second: &BoundingNode<i32, Span> = tree.child(root, 1).unwrap();
    assert_eq!(*second.object(), 2);
    assert!(tree.parent(second).unwrap().is_root());
    assert!(tree.child(second, 0).is_none() && tree.child(root, 2).is_none());
    assert!(tree.parent(root).is_none());
    tree.change_child(root_id, 0).unwrap().bounds_mut().max = 3.0;
    assert_eq!(
        tree.child(tree.root().unwrap(), 0).unwrap().bounds(),
        &span(0.0, 3.0)
    );
    let child_id: usize = tree.root().unwrap().child_id(1).unwrap();
    tree.change_parent(child_id).unwrap().bounds_mut().min = -1.0;
    assert_eq!(tree.node_by_id(root_id).unwrap().bounds().min, -1.0);
    assert!(tree.change_node_by_id(usize::MAX).is_none());
    let detached: BoundingNode<i32, Span> = BoundingNode::new(9, span(1.0, 2.0));
    assert!(detached.is_leaf() && detached.is_root() && detached.parent_id().is_none());
}

#[test]
fn gemmate_moves_content_to_child_zero_and_kill_absorbs_the_sibling() {
    let mut tree: Tree = UBTree::default();
    tree.add(1, span(0.0, 1.0));
    let root: usize = tree.root_id().unwrap();
    let leaf: usize = tree
        .gemmate(root, span(-5.0, 5.0), 7, span(4.0, 5.0))
        .unwrap();
    assert_eq!(tree.node_by_id(leaf).unwrap().parent_id(), Some(root));
    assert_eq!(leaf_order(&tree), [1, 7]);
    assert_eq!(tree.root().unwrap().bounds(), &span(-5.0, 5.0));
    let first: usize = tree.root().unwrap().child_id(0).unwrap();
    tree.gemmate(first, span(0.0, 2.0), 8, span(2.0, 2.0))
        .unwrap();
    assert_eq!(leaf_order(&tree), [1, 8, 7]);
    tree.kill(root, 1); // the root takes over the branch [1, 8]; stale ancestors are not refreshed
    assert_eq!(leaf_order(&tree), [1, 8]);
    assert_eq!(tree.root().unwrap().bounds(), &span(0.0, 2.0));
    let grandchild: usize = tree.root().unwrap().child_id(0).unwrap();
    assert_eq!(tree.node_by_id(grandchild).unwrap().parent_id(), Some(root));
    tree.kill(root, 2); // undefined natively: ignored
    tree.kill(grandchild, 0); // a leaf: no-op, as natively
    assert_eq!(leaf_order(&tree), [1, 8]);
    tree.kill(root, 0);
    assert_eq!(leaf_order(&tree), [8]);
    assert!(tree.node_by_id(grandchild).is_none());
}

#[test]
fn clear_with_allocator_swaps_the_node_allocator() {
    let arena: BaseAllocator = BaseAllocator::incremental(1024);
    let mut tree: Tree = UBTree::default();
    tree.add(1, span(0.0, 1.0));
    tree.clear_with_allocator(Some(arena.clone()));
    assert!(tree.is_empty() && *tree.allocator() == arena);
    tree.clear_with_allocator(None);
    assert!(*tree.allocator() == arena);
    let mut indexed: EBTree<i32, Span> = EBTree::default();
    indexed.add(3, span(0.0, 1.0));
    assert_eq!(*indexed.as_ubtree().root().unwrap().object(), 3);
    indexed.clear_with_allocator(Some(arena.clone()));
    assert!(!indexed.contains(&3) && *indexed.as_ubtree().allocator() == arena);
}

#[test]
fn filler_orders_follow_the_default_mt19937_stream() {
    // Draws 3499211612 % 3 == 2 and 581869302 % 2 == 0; equal bounds make the shape
    // record insertion order: first leaf, then each later leaf splits child zero.
    for (full_random, expected) in [(true, [3, 2, 1]), (false, [1, 3, 2])] {
        let mut tree: Tree = UBTree::default();
        let mut filler: UBTreeFiller<'_, i32, Span> =
            UBTreeFiller::new(&mut tree, None, full_random);
        for object in 1..=3 {
            filler.add(object, span(0.0, 1.0));
        }
        assert_eq!(filler.fill(), 3);
        assert_eq!(filler.fill(), 0);
        drop(filler);
        assert_eq!(leaf_order(&tree), expected);
    }
}

#[test]
fn filler_drop_fills_reset_discards_and_check_tree_reports_balance() {
    let mut tree: Tree = UBTree::default();
    {
        let mut filler: UBTreeFiller<'_, i32, Span> = UBTreeFiller::new(&mut tree, None, true);
        let mut text: String = String::new();
        assert_eq!(
            filler.check_tree(&mut text).unwrap_err().exception_type(),
            "Standard_NoSuchObject"
        );
        filler.add(1, span(0.0, 1.0));
    }
    assert_eq!(leaf_order(&tree), [1]);
    {
        let mut filler: UBTreeFiller<'_, i32, Span> = UBTreeFiller::new(&mut tree, None, false);
        let mut text: String = String::new();
        assert_eq!(filler.check_tree(&mut text).unwrap(), 1);
        assert_eq!(text, "Checking UBTree:       1 leaves, balance =   -nan\n");
        filler.add(2, span(3.0, 4.0));
        filler.reset();
    }
    assert_eq!(leaf_order(&tree), [1]);
}

struct Recorder {
    visited: Vec<i32>,
}
impl CellInspector for Recorder {
    type Target = i32;
    type Point = [f64; 3];
    const DIMENSION: usize = 2;
    fn coord(axis: usize, point: &[f64; 3]) -> f64 {
        point[axis]
    }
    fn is_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
    fn inspect(&mut self, target: &i32) -> CellAction {
        self.visited.push(*target);
        CellAction::Keep
    }
}
fn visit(filter: &mut CellFilter<Recorder>, point: [f64; 3]) -> Vec<i32> {
    let mut recorder: Recorder = Recorder {
        visited: Vec::new(),
    };
    filter.inspect(&point, &mut recorder);
    recorder.visited
}

#[test]
fn cell_filter_dimension_and_reset_follow_native_bounds() {
    let mut line: CellFilter<Recorder> = CellFilter::with_dimension(1, 1.0, None);
    line.add(&1, &[0.5, 7.0, 0.0]);
    assert_eq!(visit(&mut line, [0.5, -7.0, 0.0]), [1]); // only axis zero is hashed
    let mut filter: CellFilter<Recorder> =
        CellFilter::with_allocator(1.0, Some(BaseAllocator::incremental(512)));
    let mut one_based: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    one_based.as_mut_slice().fill(1.0);
    filter.reset_sizes(&one_based, None);
    filter.add(&4, &[0.5, 0.5, 0.5]);
    // Native Reset(double) writes index 0 of the 1-based array and throws before clearing.
    let error = filter.reset(2.0, None).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert_eq!(visit(&mut filter, [0.5, 0.5, 0.5]), [4]);
    let mut short: Array1<'static, f64> = Array1::with_bounds(0, 0).unwrap();
    short.as_mut_slice()[0] = 4.0;
    filter.reset_sizes(&short, None);
    filter.add(&5, &[1.0, 0.0, 0.0]);
    // Index 0 is written (size 1.0), index 1 is out of range: the cells survive.
    assert!(filter.reset(1.0, None).is_err());
    assert_eq!(visit(&mut filter, [3.9, 0.0, 0.0]), Vec::<i32>::new());
    assert_eq!(visit(&mut filter, [0.0, 0.0, 0.0]), [5]); // still in cell 0, now 1.0 wide
    filter.add(&6, &[1.5, 0.0, 0.0]);
    assert_eq!(visit(&mut filter, [1.2, 0.0, 0.0]), [6]);
}

#[test]
fn kdtree_array_builds_and_moves() {
    let mut points: Array1<'static, [f64; 2]> = Array1::with_bounds(-2, 0).unwrap();
    points
        .as_mut_slice()
        .copy_from_slice(&[[5.0, 5.0], [1.0, 0.0], [9.0, 9.0]]);
    let mut plain: KDTree<[f64; 2], 2> = KDTree::default();
    plain.build_array(&points);
    assert_eq!(plain.nearest_point(&[0.0, 0.0], None), 2);
    let mut radii: Array1<'static, f64> = Array1::with_bounds(7, 8).unwrap();
    radii.as_mut_slice().copy_from_slice(&[1.0, 2.0]);
    let mut tree: KDTree<[f64; 2], 2, true> = KDTree::default();
    tree.build_radii(&[[0.0, 0.0]], &[3.0]);
    let error = tree.build_radii_array(&points, &radii).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(tree.is_empty());
    tree.build_radii(&[[1.0, 1.0], [2.0, 2.0]], &[0.5, 0.25, 99.0]);
    assert_eq!(tree.radius(2).unwrap(), 0.25);
    let mut moved = KDTree::move_from(&mut tree);
    assert!(tree.is_empty() && moved.len() == 2);
    tree.move_assign(&mut moved);
    assert!(moved.is_empty() && tree.len() == 2);
    let copy = tree.clone();
    assert_eq!(copy.point(1).unwrap(), &[1.0, 1.0]);
}
