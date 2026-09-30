// Rust translation of FoundationClasses/TKMath/BVH/BVH_Traverse.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_PairTraverse::Select`: parallel descent of two binary trees.
use super::{BvhBaseTraverse, TraverseStack, pop_surviving};
use crate::bvh::array;
use crate::bvh::constants::MAX_TREE_DEPTH;
use crate::bvh::tree::BvhTree;
use crate::bvh::tree_owner::BvhTreeOwner;
use crate::bvh::vector::{BvhVec4i, BvhVector};
use rustycad_tkernel::standard::Failure;

/// `BVH_PairNodesInStack`: a pair of nodes (one per tree) with its metric.
/// Example: `BvhPairNodesInStack::new(0, 0, 0.0_f64)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BvhPairNodesInStack<M> {
    /// Id of the node in the first BVH tree.
    pub node_id1: i32,
    /// Id of the node in the second BVH tree.
    pub node_id2: i32,
    /// Metric computed for the pair of nodes.
    pub metric: M,
}
impl<M> BvhPairNodesInStack<M> {
    /// Creates a stack entry; `BvhPairNodesInStack::new(1, 2, metric)`.
    pub const fn new(node_id1: i32, node_id2: i32, metric: M) -> Self {
        Self {
            node_id1,
            node_id2,
            metric,
        }
    }
}

/// Selection of element pairs from two binary trees (`BVH_PairTraverse`).
/// Example: `let pairs = selector.select_trees(Some(&first), Some(&second))?;`.
pub trait BvhPairTraverse<V: BvhVector, M: Copy + Default = <V as BvhVector>::Scalar>:
    BvhBaseTraverse<M>
{
    /// Rejection of a pair of nodes by their boxes; writes the pair metric.
    fn reject_node(
        &self,
        corner_min1: &V,
        corner_max1: &V,
        corner_min2: &V,
        corner_max2: &V,
        metric: &mut M,
    ) -> bool;
    /// Acceptance of a pair of leaf elements.
    fn accept(&mut self, index1: i32, index2: i32) -> Result<bool, Failure>;
    /// `Select(theBVH1, theBVH2)`: the number of accepted pairs; `None` stands for
    /// a null handle. Example: `selector.select_trees(Some(&a), Some(&b))?`.
    fn select_trees(
        &mut self,
        bvh1: Option<&BvhTree<V>>,
        bvh2: Option<&BvhTree<V>>,
    ) -> Result<i32, Failure>
    where
        Self: Sized,
    {
        select_pairs(self, bvh1, bvh2)
    }
}

/// `BVH_PairTraverse::Select()`: selection through the two sets that own the
/// trees; a selector without both sets selects nothing.
///
/// Rust cannot hold two mutable borrows of one set, so a set paired with itself
/// goes through `select_trees(Some(&tree), Some(&tree))` instead.
/// Example: `pair_distance.select()?`.
pub trait BvhPairSetTraverse<V: BvhVector, M: Copy + Default = <V as BvhVector>::Scalar>:
    BvhPairTraverse<V, M>
{
    /// The set containing each BVH tree (`BVHSetType`).
    type Set: BvhTreeOwner<V> + ?Sized;
    /// The sets given to `SetBVHSets`, or `None` when either pointer is null.
    fn bvh_sets_mut(&mut self) -> Option<(&mut Self::Set, &mut Self::Set)>;
    /// Builds both trees when dirty (first, then second), then selects pairs.
    fn select(&mut self) -> Result<i32, Failure>
    where
        Self: Sized,
    {
        let Some((first, second)) = self.bvh_sets_mut() else {
            return Ok(0);
        };
        let bvh1 = first.take_built_bvh()?;
        let bvh2 = match second.take_built_bvh() {
            Ok(bvh2) => bvh2,
            Err(failure) => {
                first.restore_bvh(bvh1);
                return Err(failure);
            }
        };
        let selected = select_pairs(self, Some(&bvh1), Some(&bvh2));
        if let Some((first, second)) = self.bvh_sets_mut() {
            first.restore_bvh(bvh1);
            second.restore_bvh(bvh2);
        }
        selected
    }
}

/// The two trees under traversal.
struct TreePair<'a, V: BvhVector> {
    first: &'a BvhTree<V>,
    second: &'a BvhTree<V>,
}

/// Traversal state: the current pair, the pending pairs and the counter.
struct PairWalk<M> {
    node: BvhPairNodesInStack<M>,
    stack: TraverseStack<BvhPairNodesInStack<M>>,
    accepted: i32,
}

/// `BVH_PairTraverse::Select(theBVH1, theBVH2)`: descends both trees at once.
///
/// Two leaves are tested with `RejectNode` before their element pairs; other
/// pairs expand into child pairs that are sorted by `IsMetricBetter` with a
/// stable insertion. The best pair is processed next, the others are stacked
/// (at most three per step, so the stack holds 3 * 32 pairs).
/// Example: `select_pairs(&mut selector, Some(&a), Some(&b))?`.
pub fn select_pairs<V, M, T>(
    traverse: &mut T,
    bvh1: Option<&BvhTree<V>>,
    bvh2: Option<&BvhTree<V>>,
) -> Result<i32, Failure>
where
    V: BvhVector,
    M: Copy + Default,
    T: BvhPairTraverse<V, M> + ?Sized,
{
    let (Some(first), Some(second)) = (bvh1, bvh2) else {
        return Ok(0);
    };
    if first.length() == 0 || second.length() == 0 {
        return Ok(0);
    }
    let trees = TreePair { first, second };
    let mut walk = PairWalk {
        node: BvhPairNodesInStack::new(0, 0, M::default()),
        stack: TraverseStack::new(
            3 * MAX_TREE_DEPTH as usize,
            "Error! BVH pair stack overflow",
        ),
        accepted: 0,
    };
    loop {
        let previous = (walk.node.node_id1, walk.node.node_id2);
        if !walk.visit(traverse, &trees)? || !walk.advance(traverse, previous) {
            return Ok(walk.accepted);
        }
    }
}

impl<M: Copy + Default> PairWalk<M> {
    /// Processes the current pair; false when `Stop` ended the descent.
    fn visit<V: BvhVector, T: BvhPairTraverse<V, M> + ?Sized>(
        &mut self,
        traverse: &mut T,
        trees: &TreePair<'_, V>,
    ) -> Result<bool, Failure> {
        let data1 = *array::value(trees.first.node_info_buffer(), self.node.node_id1)?;
        let data2 = *array::value(trees.second.node_info_buffer(), self.node.node_id2)?;
        if data1.x != 0 && data2.x != 0 {
            return self.visit_leaves(traverse, trees, &data1, &data2);
        }
        let pairs = self.child_pairs(&data1, &data2);
        let kept = keep_sorted(traverse, trees, pairs)?;
        if let Some((best, rest)) = kept.split_first() {
            self.node = *best;
            for pair in rest {
                self.stack.ensure_room()?;
                self.stack.push(*pair);
            }
        }
        Ok(true)
    }

    /// Outer/Outer: element pairs are tested only when the leaf boxes pass.
    fn visit_leaves<V: BvhVector, T: BvhPairTraverse<V, M> + ?Sized>(
        &mut self,
        traverse: &mut T,
        trees: &TreePair<'_, V>,
        data1: &BvhVec4i,
        data2: &BvhVec4i,
    ) -> Result<bool, Failure> {
        let mut metric = M::default();
        if reject_pair(traverse, trees, &self.node, &mut metric)? {
            return Ok(true);
        }
        for index1 in data1.y..=data1.z {
            for index2 in data2.y..=data2.z {
                if traverse.accept(index1, index2)? {
                    self.accepted += 1;
                }
                if traverse.stop() {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Inner/Inner gives four pairs, Inner/Outer and Outer/Inner give two.
    fn child_pairs(&self, data1: &BvhVec4i, data2: &BvhVec4i) -> Vec<BvhPairNodesInStack<M>> {
        let pair = |first: i32, second: i32| BvhPairNodesInStack::new(first, second, M::default());
        let (node1, node2) = (self.node.node_id1, self.node.node_id2);
        match (data1.x == 0, data2.x == 0) {
            (true, true) => vec![
                pair(data1.y, data2.y),
                pair(data1.y, data2.z),
                pair(data1.z, data2.y),
                pair(data1.z, data2.z),
            ],
            (true, false) => vec![pair(data1.y, node2), pair(data1.z, node2)],
            _ => vec![pair(node1, data2.y), pair(node1, data2.z)],
        }
    }

    /// When the pair did not change, resumes from the stack, skipping entries
    /// that `RejectMetric` prunes; false when nothing is left.
    fn advance<T: BvhBaseTraverse<M> + ?Sized>(
        &mut self,
        traverse: &T,
        previous: (i32, i32),
    ) -> bool {
        if (self.node.node_id1, self.node.node_id2) != previous {
            return true;
        }
        match pop_surviving(&mut self.stack, traverse, |entry| &entry.metric) {
            Some(entry) => {
                self.node = entry;
                true
            }
            None => false,
        }
    }
}

/// `RejectNode` on the boxes of both nodes of a pair.
fn reject_pair<V: BvhVector, M, T: BvhPairTraverse<V, M> + ?Sized>(
    traverse: &T,
    trees: &TreePair<'_, V>,
    pair: &BvhPairNodesInStack<M>,
    metric: &mut M,
) -> Result<bool, Failure>
where
    M: Copy + Default,
{
    let (min1, max1) = (
        trees.first.min_point(pair.node_id1)?,
        trees.first.max_point(pair.node_id1)?,
    );
    let (min2, max2) = (
        trees.second.min_point(pair.node_id2)?,
        trees.second.max_point(pair.node_id2)?,
    );
    Ok(traverse.reject_node(min1, max1, min2, max2, metric))
}

/// Computes each pair metric and keeps the unrejected pairs sorted best first;
/// an equal metric never moves ahead of an earlier pair.
fn keep_sorted<V: BvhVector, M: Copy + Default, T: BvhPairTraverse<V, M> + ?Sized>(
    traverse: &T,
    trees: &TreePair<'_, V>,
    mut pairs: Vec<BvhPairNodesInStack<M>>,
) -> Result<Vec<BvhPairNodesInStack<M>>, Failure> {
    let mut kept: Vec<BvhPairNodesInStack<M>> = Vec::with_capacity(pairs.len());
    for pair in pairs.iter_mut() {
        let mut metric = pair.metric;
        if reject_pair(traverse, trees, pair, &mut metric)? {
            continue;
        }
        pair.metric = metric;
        let mut slot = kept.len();
        while slot > 0 && traverse.is_metric_better(&pair.metric, &kept[slot - 1].metric) {
            slot -= 1;
        }
        kept.insert(slot, *pair);
    }
    Ok(kept)
}
