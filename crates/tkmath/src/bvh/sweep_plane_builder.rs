// Rust translation of FoundationClasses/TKMath/BVH/BVH_SweepPlaneBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_SweepPlaneBuilder<T, N>`: full sweep SAH over sorted centroids.
use super::bounding_box::BvhBox;
use super::builder::{BvhBuilder, BvhBuilderTransient};
use super::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH, THE_NODE_MIN_SIZE};
use super::queue_builder::{
    BvhChildNodes, BvhParallelQueueBuilder, BvhPrimitiveRange, BvhQueueBuilder,
    parallel_queue_build, queue_build,
};
use super::quick_sorter::BvhQuickSorter;
use super::scalar::BvhScalar;
use super::set::{BvhSet, BvhSharedSet};
use super::sorter::BvhSorter;
use super::tree::{BvhNodeSnapshot, BvhTree};
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::marker::PhantomData;

/// Sweep plane SAH builder: for each axis the node primitives are sorted by
/// centroid and every split position between them is costed.
/// Example: `BvhSweepPlaneBuilder::<BvhVec3d>::new(5, 32, 1)`.
#[derive(Debug)]
pub struct BvhSweepPlaneBuilder<V: BvhVector> {
    transient: BvhBuilderTransient,
    num_of_threads: i32,
    vector: PhantomData<V>,
}
impl<V: BvhVector> Default for BvhSweepPlaneBuilder<V> {
    fn default() -> Self {
        Self::new(LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH, 1)
    }
}

/// The best split so far: `aMinSplitAxis`, `aMinSplitIndex`, `aMinSplitCost`.
struct SweepSplit {
    axis: i32,
    index: i32,
    cost: f64,
}

/// The primitive range `[begin, end]` of the node being split.
#[derive(Clone, Copy)]
struct NodeRange {
    begin: i32,
    end: i32,
    count: i32,
}

impl<V: BvhVector> BvhSweepPlaneBuilder<V> {
    /// Creates sweep plane SAH BVH builder; `BvhSweepPlaneBuilder::<BvhVec3d>::new(5, 32, 1)`.
    pub fn new(leaf_node_size: i32, max_tree_depth: i32, num_of_threads: i32) -> Self {
        Self {
            transient: BvhBuilderTransient::new(leaf_node_size, max_tree_depth),
            num_of_threads,
            vector: PhantomData,
        }
    }

    /// Tries X, Y (and Z) in order; an axis whose node extent is at most
    /// `THE_NODE_MIN_SIZE` is skipped. The set stays sorted along the last
    /// axis tried.
    fn find_best_split(
        &self,
        set: &mut dyn SweepSetAccess<V>,
        node: BvhNodeSnapshot<V>,
        range: NodeRange,
    ) -> Result<SweepSplit, Failure> {
        let mut best = empty_split();
        let (mut left_areas, mut right_areas) = split_area_buffers(range.count);
        for axis in 0..V::DIMENSION.min(3) as i32 {
            if extent(node, axis).to_double() <= THE_NODE_MIN_SIZE {
                continue;
            }
            set.sort_range(axis, range.begin, range.end)?;
            sweep_areas(set, range, &mut left_areas, &mut right_areas)?;
            choose_split(&left_areas, &right_areas, range.count, axis, &mut best);
        }
        Ok(best)
    }

    fn split_node_shared(
        &self,
        set: &dyn BvhSharedSet<V>,
        node: BvhNodeSnapshot<V>,
        mark_parent_inner: &mut dyn FnMut() -> Result<(), Failure>,
    ) -> Result<BvhChildNodes<V>, Failure> {
        let range = node_range(node);
        if range.count <= self.transient.leaf_node_size() {
            return Ok(BvhChildNodes::default());
        }
        let mut access = SharedSweepSetAccess(set);
        let best = self.find_best_split(&mut access, node, range)?;
        if best.axis == -1 {
            return Ok(BvhChildNodes::default());
        }
        mark_parent_inner()?;
        self.finish_split(&mut access, range, best)
    }

    fn finish_split(
        &self,
        set: &mut dyn SweepSetAccess<V>,
        range: NodeRange,
        best: SweepSplit,
    ) -> Result<BvhChildNodes<V>, Failure> {
        let last_axis = V::DIMENSION.min(3) as i32 - 1;
        if best.axis != last_axis {
            set.sort_range(best.axis, range.begin, range.end)?;
        }
        let middle = range.begin + best.index;
        let (left, right) = split_boxes(set, range, middle)?;
        Ok(BvhChildNodes::new(
            left,
            right,
            BvhPrimitiveRange::new(range.begin, middle - 1),
            BvhPrimitiveRange::new(middle, range.end),
        ))
    }
}

/// Areas of the boxes of the first (left) and last (right) `k` primitives,
/// for k in `1..count`, stored at index k as `double`.
trait SweepSetAccess<V: BvhVector> {
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure>;
    fn sort_range(&mut self, axis: i32, begin: i32, end: i32) -> Result<(), Failure>;
}

struct MutableSweepSetAccess<'a, V: BvhVector>(&'a mut dyn BvhSet<V>);

impl<V: BvhVector> SweepSetAccess<V> for MutableSweepSetAccess<'_, V> {
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.0.element_box(index)
    }

    fn sort_range(&mut self, axis: i32, begin: i32, end: i32) -> Result<(), Failure> {
        BvhQuickSorter::new(axis).perform_range(self.0, begin, end)
    }
}

struct SharedSweepSetAccess<'a, V: BvhVector>(&'a dyn BvhSharedSet<V>);

impl<V: BvhVector> SweepSetAccess<V> for SharedSweepSetAccess<'_, V> {
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.0.element_box(index)
    }

    fn sort_range(&mut self, axis: i32, begin: i32, end: i32) -> Result<(), Failure> {
        super::quick_sorter::sort_range_shared(self.0, axis, begin, end)
    }
}

fn sweep_areas<V: BvhVector>(
    set: &dyn SweepSetAccess<V>,
    range: NodeRange,
    left_areas: &mut [f64],
    right_areas: &mut [f64],
) -> Result<(), Failure> {
    let mut left_box = BvhBox::<V>::default();
    for index in 1..range.count {
        left_box.combine(&set.element_box(index + range.begin - 1)?);
        left_areas[index as usize] = left_box.area().to_double();
    }
    let mut right_box = BvhBox::<V>::default();
    for index in 1..range.count {
        right_box.combine(&set.element_box(range.end - index + 1)?);
        right_areas[index as usize] = right_box.area().to_double();
    }
    Ok(())
}

/// Simplified SAH (areas are not divided by the node area); only a strictly
/// smaller cost replaces the current best.
fn choose_split(
    left_areas: &[f64],
    right_areas: &[f64],
    count: i32,
    axis: i32,
    best: &mut SweepSplit,
) {
    for left in 1..count {
        let right = count - left;
        let cost = left_areas[left as usize] * f64::from(left)
            + right_areas[right as usize] * f64::from(right);
        if cost < best.cost {
            *best = SweepSplit {
                axis,
                index: left,
                cost,
            };
        }
    }
}

/// Boxes of the primitives before and after the split, the right one grown
/// from the last primitive down.
fn split_boxes<V: BvhVector>(
    set: &dyn SweepSetAccess<V>,
    range: NodeRange,
    middle: i32,
) -> Result<(BvhBox<V>, BvhBox<V>), Failure> {
    let mut left = BvhBox::default();
    for index in range.begin..middle {
        left.combine(&set.element_box(index)?);
    }
    let mut right = BvhBox::default();
    for index in (middle..=range.end).rev() {
        right.combine(&set.element_box(index)?);
    }
    Ok((left, right))
}

fn empty_split() -> SweepSplit {
    SweepSplit {
        axis: -1,
        index: 0,
        cost: f64::MAX,
    }
}

fn split_area_buffers(count: i32) -> (Vec<f64>, Vec<f64>) {
    (vec![0.0_f64; count as usize], vec![0.0_f64; count as usize])
}

fn extent<V: BvhVector>(node: BvhNodeSnapshot<V>, axis: i32) -> V::Scalar {
    node.maximum.component(axis) - node.minimum.component(axis)
}

fn node_range<V: BvhVector>(node: BvhNodeSnapshot<V>) -> NodeRange {
    NodeRange {
        begin: node.begin,
        end: node.end,
        count: node.end - node.begin + 1,
    }
}

impl<V: BvhVector> BvhQueueBuilder<V> for BvhSweepPlaneBuilder<V> {
    fn queue_builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }
    fn num_of_threads(&self) -> i32 {
        self.num_of_threads
    }
    /// Splits a node with more than `LeafNodeSize` primitives at the cheapest
    /// sweep position; no split when every axis is too thin. The set is sorted
    /// again unless the chosen axis is the last one (`N < 4 ? N - 1 : 2`).
    fn build_node(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        node: i32,
    ) -> Result<BvhChildNodes<V>, Failure> {
        let snapshot = bvh.node_snapshot(node)?;
        let range = node_range(snapshot);
        if range.count <= self.transient.leaf_node_size() {
            return Ok(BvhChildNodes::default());
        }
        let mut access = MutableSweepSetAccess(set);
        let best = self.find_best_split(&mut access, snapshot, range)?;
        if best.axis == -1 {
            return Ok(BvhChildNodes::default());
        }
        bvh.set_inner(node)?;
        self.finish_split(&mut access, range, best)
    }
}
impl<V: BvhVector> BvhParallelQueueBuilder<V> for BvhSweepPlaneBuilder<V> {
    fn queue_builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }

    fn num_of_threads(&self) -> i32 {
        self.num_of_threads
    }

    fn build_node(
        &self,
        set: &dyn BvhSharedSet<V>,
        node: BvhNodeSnapshot<V>,
        mark_parent_inner: &mut dyn FnMut() -> Result<(), Failure>,
    ) -> Result<BvhChildNodes<V>, Failure> {
        self.split_node_shared(set, node, mark_parent_inner)
    }
}

impl<V: BvhVector> BvhBuilder<V> for BvhSweepPlaneBuilder<V> {
    fn builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }
    fn build(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        bounding: &BvhBox<V>,
    ) -> Result<(), Failure> {
        if self.num_of_threads <= 1 {
            return queue_build(self, set, bvh, bounding);
        }
        let shared = super::shared_set::BvhBorrowedMutexSet::new(set);
        parallel_queue_build(self, &shared, bvh, bounding)
    }
}
