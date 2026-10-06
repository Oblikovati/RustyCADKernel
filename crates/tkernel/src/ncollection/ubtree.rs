//! Unbalanced bounding tree with explicit parent links and iterative traversal.
//!
//! Native `TreeNode*` links become node ids into the tree's arena: `UBTree::root_id`,
//! `BoundingNode::child_id` and `BoundingNode::parent_id` name nodes, and the tree resolves
//! them (`UBTree::child`, `UBTree::change_child`, ...).
use super::{AllocatedBox, BaseAllocator};
mod destruction;
mod insertion;
mod navigation;
mod payload;
mod removal;
mod selection;
pub use selection::FallibleTreeSelector;
/// Bounding geometry required by the native insertion algorithm.
pub trait BoundingVolume: Clone {
    /// Enlarge this bound to include another; `bound.add(&other)`.
    fn add(&mut self, other: &Self);
    /// Reject disjoint geometry; `bound.is_out(&other)`.
    fn is_out(&self, other: &Self) -> bool;
    /// Measure squared extent for branch choice; `bound.square_extent()`.
    fn square_extent(&self) -> f64;
}
/// Selection callbacks; `stop()` is checked after each leaf acceptance attempt.
pub trait TreeSelector<T, B> {
    /// Reject a complete branch; `selector.reject(&bounds)`.
    fn reject(&self, bounds: &B) -> bool;
    /// Inspect a candidate and count accepted objects; `selector.accept(&object)`.
    fn accept(&mut self, object: &T) -> bool;
    /// Stop after a candidate callback; return `true` to stop.
    fn stop(&self) -> bool;
}
/// A stable tree node; branch objects hold the target type's default value.
pub struct BoundingNode<T, B> {
    // OCCT 3d097a0328 TreeNode destroys object before bounds; Rust drops in field order.
    object: T,
    bounds: B,
    children: Option<[usize; 2]>,
    parent: Option<usize>,
}
impl<T, B> BoundingNode<T, B> {
    /// A detached leaf holding `object`, as `TreeNode(theObj, theBnd)`; `BoundingNode::new(1, bounds)`.
    pub fn new(object: T, bounds: B) -> Self {
        Self {
            bounds,
            object,
            children: None,
            parent: None,
        }
    }
    /// Distinguish object leaves from branches; `node.is_leaf()`.
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }
    /// Identify the root; `node.is_root()`.
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
    /// Borrow the target; `node.object()`.
    pub fn object(&self) -> &T {
        &self.object
    }
    /// Borrow the geometry; `node.bounds()`.
    pub fn bounds(&self) -> &B {
        &self.bounds
    }
    /// Mutable geometry, as native `ChangeBnd`; `node.bounds_mut().add(&other)`.
    pub fn bounds_mut(&mut self) -> &mut B {
        &mut self.bounds
    }
    /// Arena id of child `index` (0 or 1), `None` for a leaf or another index.
    /// Example: `tree.change_child(id, 0)` after `node.child_id(0)`.
    pub fn child_id(&self, index: i32) -> Option<usize> {
        let children: [usize; 2] = self.children?;
        usize::try_from(index)
            .ok()
            .and_then(|slot| children.get(slot).copied())
    }
    /// Arena id of the parent, `None` for the root; `node.parent_id()`.
    pub fn parent_id(&self) -> Option<usize> {
        self.parent
    }
}
/// OCCT UBTree insertion and selection; `UBTree::<i32, Bounds>::default()`.
pub struct UBTree<T: Clone + Default, B: BoundingVolume> {
    nodes: Vec<Option<AllocatedBox<BoundingNode<T, B>>>>,
    root: Option<usize>,
    last: Option<usize>,
    resource: BaseAllocator,
}
impl<T: Clone + Default, B: BoundingVolume> Default for UBTree<T, B> {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    /// Own nodes in an injected allocator; `UBTree::with_allocator(arena)`.
    pub fn with_allocator(resource: BaseAllocator) -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
            last: None,
            resource,
        }
    }
    /// Check root absence; `tree.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
    /// Borrow the root when present; `tree.root().unwrap().bounds()`.
    pub fn root(&self) -> Option<&BoundingNode<T, B>> {
        self.root.map(|index| self.node(index))
    }
    /// Arena id of the root when present; `tree.root_id()`.
    pub fn root_id(&self) -> Option<usize> {
        self.root
    }
    /// Preserve allocator identity; `tree.allocator()`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.resource
    }
    /// Release all nodes without recursive destruction; `tree.clear()`.
    pub fn clear(&mut self) {
        self.clear_with_allocator(None);
    }
    /// Release all nodes, then adopt `allocator` for future nodes when given (native
    /// `Clear(aNewAlloc)`, which also swaps the allocator of an empty tree).
    /// If a Rust destructor panics, drain the tree and retain its original allocator.
    /// Example: `tree.clear_with_allocator(Some(BaseAllocator::incremental(1024)))`.
    pub fn clear_with_allocator(&mut self, allocator: Option<BaseAllocator>) {
        self.clear_owned_nodes();
        if let Some(resource) = allocator {
            self.resource = resource;
        }
    }
    /// Transfer nodes and leave an empty source; `UBTree::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        let replacement: Self = Self::with_allocator(source.resource.clone());
        std::mem::replace(source, replacement)
    }
    /// Drop old contents and transfer source nodes; `tree.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) -> &mut Self {
        *self = Self::move_from(source);
        self
    }
    /// Insert an object with its bound; duplicates are permitted, `tree.add(1, bound)`.
    pub fn add(&mut self, object: T, bounds: B) -> bool {
        self.insert_leaf(object, bounds);
        true
    }
    /// Visit child zero first and count accepted leaves; `tree.select(&mut selector)`.
    pub fn select<S: TreeSelector<T, B> + ?Sized>(&self, selector: &mut S) -> i32 {
        self.select_nodes(self.root, selector)
    }
    fn select_nodes<S: TreeSelector<T, B> + ?Sized>(
        &self,
        root: Option<usize>,
        selector: &mut S,
    ) -> i32 {
        let mut pending: Vec<usize> = root.into_iter().collect();
        let mut count: i32 = 0;
        while let Some(index) = pending.pop() {
            let node: &BoundingNode<T, B> = self.node(index);
            if selector.reject(&node.bounds) {
                continue;
            }
            if let Some([first, second]) = node.children {
                pending.extend([second, first]);
                continue;
            }
            count += i32::from(selector.accept(&node.object));
            if selector.stop() {
                break;
            }
        }
        count
    }
    pub(super) fn node(&self, index: usize) -> &BoundingNode<T, B> {
        self.nodes[index].as_deref().unwrap()
    }
    fn node_mut(&mut self, index: usize) -> &mut BoundingNode<T, B> {
        self.nodes[index].as_deref_mut().unwrap()
    }
    fn allocate_node(&mut self, node: BoundingNode<T, B>) -> usize {
        let index: usize = self.nodes.len();
        self.nodes.push(Some(self.resource.allocate_value(node)));
        index
    }
}
mod method_contracts;
