// Rust translation of FoundationClasses/TKMath/BVH/BVH_BinnedBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_BinnedBuilder::buildNode` and `getSubVolumes`.
use super::partition::split_primitives_shared;
use super::{BvhBin, BvhBinnedBuilder, BvhSplitPlane, main_axis, split_primitives};
use crate::bvh::bounding_box::BvhBox;
use crate::bvh::constants::THE_NODE_MIN_SIZE;
use crate::bvh::queue_builder::{BvhChildNodes, BvhPrimitiveRange};
use crate::bvh::scalar::{BvhScalar, int_floor};
use crate::bvh::set::{BvhSet, BvhSetRead, BvhSharedSet, MutableSetRead, SharedSetRead};
use crate::bvh::tree::{BvhNodeSnapshot, BvhTree};
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// The best split found so far (`aMinSplit*` locals); cost is evaluated in double.
struct BestSplit<V: BvhVector> {
    axis: i32,
    index: i32,
    left: BvhBin<V>,
    right: BvhBin<V>,
    cost: f64,
}

/// `getSubVolumes`: counts primitives and accumulates their boxes per bin along
/// `axis` of the node box, clamping centroid bins to `[0, BINS - 1]`.
/// Example: `get_sub_volumes::<V, 32>(set, &tree, node, &mut bins, 0)?`.
pub fn get_sub_volumes<V: BvhVector, const BINS: usize>(
    set: &dyn BvhSet<V>,
    bvh: &BvhTree<V>,
    node: i32,
    bins: &mut [BvhBin<V>],
    axis: i32,
) -> Result<(), Failure> {
    let min = bvh.min_point(node)?.component(axis);
    let max = bvh.max_point(node)?.component(axis);
    let begin = bvh.beg_primitive(node)?;
    let end = bvh.end_primitive(node)?;
    let access = MutableSetRead::new(set);
    accumulate_sub_volumes::<V, BINS>(&access, begin, end, min, max, bins, axis)
}

fn accumulate_sub_volumes<V: BvhVector, const BINS: usize>(
    set: &dyn BvhSetRead<V>,
    begin: i32,
    end: i32,
    min: V::Scalar,
    max: V::Scalar,
    bins: &mut [BvhBin<V>],
    axis: i32,
) -> Result<(), Failure> {
    let inverse_step = V::Scalar::from_int(BINS as i32) / (max - min);
    for index in begin..=end {
        let bounding = set.element_box(index)?;
        let bin = int_floor((set.center(index, axis)? - min) * inverse_step);
        let bin = bin.clamp(0, BINS as i32 - 1) as usize;
        bins[bin].count += 1;
        bins[bin].bounding.combine(&bounding);
    }
    Ok(())
}

/// Prefix sums of bin counts and boxes from both ends (`aSplitPlanes`).
fn split_planes<V: BvhVector, const BINS: usize>(bins: &[BvhBin<V>]) -> Vec<BvhSplitPlane<V>> {
    let mut planes = vec![BvhSplitPlane::default(); BINS + 1];
    for left in 1..BINS {
        let right = BINS - left;
        planes[left].left_voxel.count = planes[left - 1].left_voxel.count + bins[left - 1].count;
        planes[right].right_voxel.count = planes[right + 1].right_voxel.count + bins[right].count;
        planes[left].left_voxel.bounding = planes[left - 1].left_voxel.bounding;
        planes[right].right_voxel.bounding = planes[right + 1].right_voxel.bounding;
        planes[left]
            .left_voxel
            .bounding
            .combine(&bins[left - 1].bounding);
        planes[right]
            .right_voxel
            .bounding
            .combine(&bins[right].bounding);
    }
    planes
}

/// Keeps the last candidate with the minimum SAH cost (`<=`), normalized by the
/// parent area in double precision.
fn choose_split<V: BvhVector, const BINS: usize>(
    planes: &[BvhSplitPlane<V>],
    parent_area: f64,
    axis: i32,
    best: &mut BestSplit<V>,
) {
    for (split, plane) in planes.iter().enumerate().take(BINS).skip(1) {
        let left_area = plane.left_voxel.bounding.area().to_double();
        let right_area = plane.right_voxel.bounding.area().to_double();
        let cost = (left_area / parent_area) * f64::from(plane.left_voxel.count)
            + (right_area / parent_area) * f64::from(plane.right_voxel.count);
        if cost <= best.cost {
            *best = BestSplit {
                axis,
                index: split as i32,
                left: plane.left_voxel,
                right: plane.right_voxel,
                cost,
            };
        }
    }
}

/// Searches the candidate axes (all of X/Y/Z, or only the main one).
fn find_best_split<V: BvhVector, const BINS: usize>(
    builder: &BvhBinnedBuilder<V, BINS>,
    set: &dyn BvhSetRead<V>,
    node: BvhNodeSnapshot<V>,
    node_box: &BvhBox<V>,
) -> Result<BestSplit<V>, Failure> {
    let size = node_box.size();
    let mut best = BestSplit {
        axis: -1,
        index: 0,
        left: BvhBin::default(),
        right: BvhBin::default(),
        cost: f64::MAX,
    };
    let main = main_axis(&size);
    let (first, last) = if builder.uses_main_axis() {
        (main, main)
    } else {
        (0, (V::DIMENSION as i32 - 1).min(2))
    };
    for axis in first..=last {
        if size.component(axis).to_double() <= THE_NODE_MIN_SIZE {
            continue;
        }
        let mut bins = vec![BvhBin::default(); BINS];
        accumulate_sub_volumes::<V, BINS>(
            set,
            node.begin,
            node.end,
            node.minimum.component(axis),
            node.maximum.component(axis),
            &mut bins,
            axis,
        )?;
        let planes = split_planes::<V, BINS>(&bins);
        choose_split::<V, BINS>(&planes, node_box.area().to_double(), axis, &mut best);
    }
    Ok(best)
}

/// `buildNode`: splits a node with more than `LeafNodeSize` primitives at the
/// best binned SAH plane, or at the index middle when all centroids coincide.
pub(super) fn build_binned_node<V: BvhVector, const BINS: usize>(
    builder: &BvhBinnedBuilder<V, BINS>,
    set: &mut dyn BvhSet<V>,
    bvh: &mut BvhTree<V>,
    node: i32,
) -> Result<BvhChildNodes<V>, Failure> {
    let snapshot = bvh.node_snapshot(node)?;
    if snapshot.end - snapshot.begin < builder.transient.leaf_node_size() {
        return Ok(BvhChildNodes::default());
    }
    let node_box = BvhBox::from_corners(snapshot.minimum, snapshot.maximum);
    let access = MutableSetRead::new(&*set);
    let mut best = find_best_split(builder, &access, snapshot, &node_box)?;
    bvh.set_inner(node)?;
    let middle = if needs_middle_split(&best) {
        middle_split(&access, snapshot.begin, snapshot.end, &mut best)?
    } else {
        split_primitives(
            set,
            &node_box,
            snapshot.begin,
            snapshot.end,
            best.index - 1,
            best.axis,
            BINS as i32,
        )?
    };
    Ok(child_nodes(snapshot, &best, middle))
}

pub(super) fn build_binned_node_shared<V: BvhVector, const BINS: usize>(
    builder: &BvhBinnedBuilder<V, BINS>,
    set: &dyn BvhSharedSet<V>,
    snapshot: BvhNodeSnapshot<V>,
    mark_parent_inner: &mut dyn FnMut() -> Result<(), Failure>,
) -> Result<BvhChildNodes<V>, Failure> {
    if snapshot.end - snapshot.begin < builder.transient.leaf_node_size() {
        return Ok(BvhChildNodes::default());
    }
    let node_box = BvhBox::from_corners(snapshot.minimum, snapshot.maximum);
    let access = SharedSetRead::new(set);
    let mut best = find_best_split(builder, &access, snapshot, &node_box)?;
    mark_parent_inner()?;
    let middle =
        split_binned_range_shared::<V, BINS>(set, snapshot, &access, &node_box, &mut best)?;
    Ok(child_nodes(snapshot, &best, middle))
}

fn split_binned_range_shared<V: BvhVector, const BINS: usize>(
    set: &dyn BvhSharedSet<V>,
    node: BvhNodeSnapshot<V>,
    access: &dyn BvhSetRead<V>,
    node_box: &BvhBox<V>,
    best: &mut BestSplit<V>,
) -> Result<i32, Failure> {
    if needs_middle_split(best) {
        return middle_split(access, node.begin, node.end, best);
    }
    split_primitives_shared(
        set,
        node_box,
        node.begin,
        node.end,
        best.index - 1,
        best.axis,
        BINS as i32,
    )
}

fn needs_middle_split<V: BvhVector>(best: &BestSplit<V>) -> bool {
    best.left.count == 0 || best.right.count == 0 || best.axis == -1
}

fn child_nodes<V: BvhVector>(
    node: BvhNodeSnapshot<V>,
    best: &BestSplit<V>,
    middle: i32,
) -> BvhChildNodes<V> {
    BvhChildNodes::new(
        best.left.bounding,
        best.right.bounding,
        BvhPrimitiveRange::new(node.begin, middle - 1),
        BvhPrimitiveRange::new(middle, node.end),
    )
}

/// Objects with the same center: split the index range in the middle, the
/// middle being computed through `float` as `(beg + end) / 2.f` upstream.
fn middle_split<V: BvhVector>(
    set: &dyn BvhSetRead<V>,
    begin: i32,
    end: i32,
    best: &mut BestSplit<V>,
) -> Result<i32, Failure> {
    best.left.bounding.clear();
    best.right.bounding.clear();
    let half = (begin + end) as f32 / 2.0;
    let middle = (begin + 1).max(half.truncate_to_int());
    best.left.count = middle - begin;
    for index in begin..middle {
        best.left.bounding.combine(&set.element_box(index)?);
    }
    best.right.count = end - middle + 1;
    for index in (middle..=end).rev() {
        best.right.bounding.combine(&set.element_box(index)?);
    }
    Ok(middle)
}
