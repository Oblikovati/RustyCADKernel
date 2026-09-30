// Rust translation of FoundationClasses/TKMath/BVH/BVH_LinearBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::UpdateBounds` and `BVH::UpdateBoundTask`: bottom-up node boxes.
use crate::bvh::array;
use crate::bvh::bounding_box::{cwise_std_max, cwise_std_min};
use crate::bvh::set::BvhSet;
use crate::bvh::tree::BvhTree;
use crate::bvh::vector::{BvhVec4i, BvhVector};
use rustycad_tkernel::standard::Failure;

fn node_record<V: BvhVector>(tree: &BvhTree<V>, node: i32) -> Result<BvhVec4i, Failure> {
    array::value(tree.node_info_buffer(), node).copied()
}

/// `BVH::UpdateBounds`: computes the boxes of the subtree rooted at `node` and
/// returns its height (0 for a leaf).
///
/// A leaf box starts at its first primitive box and grows with `std::min` and
/// `std::max`; an inner box merges the child boxes the same way.
/// Example: `let height = update_bounds(&set, &mut tree, 0)?;`.
pub fn update_bounds<V: BvhVector>(
    set: &dyn BvhSet<V>,
    tree: &mut BvhTree<V>,
    node: i32,
) -> Result<i32, Failure> {
    let record = node_record(tree, node)?;
    if record.x != 0 {
        update_leaf_bounds(set, tree, node, &record)?;
        return Ok(0);
    }
    let left_height = update_bounds(set, tree, record.y)?;
    let right_height = update_bounds(set, tree, record.z)?;
    merge_child_bounds(tree, node, record.y, record.z)?;
    Ok(left_height.max(right_height) + 1)
}

/// Leaf box of primitives `[record.y, record.z]`; an empty range keeps the old
/// point values, as upstream.
fn update_leaf_bounds<V: BvhVector>(
    set: &dyn BvhSet<V>,
    tree: &mut BvhTree<V>,
    node: i32,
    record: &BvhVec4i,
) -> Result<(), Failure> {
    for primitive in record.y..=record.z {
        let bounding = set.element_box(primitive)?;
        let min_point = tree.min_point_mut(node)?;
        if primitive == record.y {
            *min_point = *bounding.corner_min();
        } else {
            cwise_std_min(min_point, bounding.corner_min());
        }
        let max_point = tree.max_point_mut(node)?;
        if primitive == record.y {
            *max_point = *bounding.corner_max();
        } else {
            cwise_std_max(max_point, bounding.corner_max());
        }
    }
    Ok(())
}

/// Inner box: the left child box widened by the right child box.
fn merge_child_bounds<V: BvhVector>(
    tree: &mut BvhTree<V>,
    node: i32,
    left: i32,
    right: i32,
) -> Result<(), Failure> {
    let mut min_point = *tree.min_point(left)?;
    let mut max_point = *tree.max_point(left)?;
    cwise_std_min(&mut min_point, tree.min_point(right)?);
    cwise_std_max(&mut max_point, tree.max_point(right)?);
    *tree.min_point_mut(node)? = min_point;
    *tree.max_point_mut(node)? = max_point;
    Ok(())
}

/// `BVH::UpdateBoundTask`: updates the three top levels task by task, then
/// switches to the recursive [`update_bounds`].
///
/// Upstream runs the child tasks of each of those levels through
/// `OSD_Parallel::ForEach` when parallel. Every task writes only its own
/// subtree and reads the set, so the result does not depend on the schedule;
/// the Rust task runs them in order, which borrows the tree safely.
/// Example: `BvhUpdateBoundTask::new(false).run(&set, &mut tree, 0, 0)?`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BvhUpdateBoundTask {
    is_parallel: bool,
}
impl BvhUpdateBoundTask {
    /// Creates the task with the builder parallel flag; `BvhUpdateBoundTask::new(true)`.
    pub const fn new(is_parallel: bool) -> Self {
        Self { is_parallel }
    }
    /// The parallel flag given to child tasks; `task.is_parallel()`.
    pub const fn is_parallel(&self) -> bool {
        self.is_parallel
    }
    /// `operator()(BoundData)`: updates the subtree at `node` found at `level`
    /// and returns its height (the `*myHeight` output upstream).
    /// Example: `let height = task.run(&set, &mut tree, 0, 0)?;`.
    pub fn run<V: BvhVector>(
        &self,
        set: &dyn BvhSet<V>,
        tree: &mut BvhTree<V>,
        node: i32,
        level: i32,
    ) -> Result<i32, Failure> {
        if tree.is_outer(node)? || level > 2 {
            return update_bounds(set, tree, node);
        }
        let record = node_record(tree, node)?;
        let left_height = self.run_child(set, tree, record.y, level)?;
        let right_height = self.run_child(set, tree, record.z, level)?;
        merge_child_bounds(tree, node, record.y, record.z)?;
        Ok(left_height.max(right_height) + 1)
    }
    /// A leaf child is updated at once; an inner child becomes a task one level
    /// deeper.
    fn run_child<V: BvhVector>(
        &self,
        set: &dyn BvhSet<V>,
        tree: &mut BvhTree<V>,
        child: i32,
        level: i32,
    ) -> Result<i32, Failure> {
        if tree.is_outer(child)? {
            return update_bounds(set, tree, child);
        }
        self.run(set, tree, child, level + 1)
    }
}
