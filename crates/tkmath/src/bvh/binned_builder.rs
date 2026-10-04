// Rust translation of FoundationClasses/TKMath/BVH/BVH_BinnedBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_BinnedBuilder<T, N, Bins>`: binned SAH (surface area heuristic) splits.
use super::bounding_box::BvhBox;
use super::builder::{BvhBuilder, BvhBuilderTransient};
use super::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH, NB_BINS_OPTIMAL};
use super::queue_builder::{BvhChildNodes, BvhParallelQueueBuilder, BvhQueueBuilder, queue_build};
use super::set::{BvhSet, BvhSharedSet};
use super::tree::{BvhNodeSnapshot, BvhTree};
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::marker::PhantomData;
mod partition;
mod split;
pub use partition::{main_axis, split_primitives};
pub use split::get_sub_volumes;

/// Parameters of a single bin (slice of AABB), `BVH_Bin`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhBin<V: BvhVector> {
    /// Number of primitives in the bin.
    pub count: i32,
    /// AABB of primitives in the bin.
    pub bounding: BvhBox<V>,
}
impl<V: BvhVector> Default for BvhBin<V> {
    fn default() -> Self {
        Self {
            count: 0,
            bounding: BvhBox::default(),
        }
    }
}

/// Split plane candidate: accumulated bins on each side, `BVH_SplitPlane`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BvhSplitPlane<V: BvhVector> {
    /// Bins left of the plane.
    pub left_voxel: BvhBin<V>,
    /// Bins right of the plane.
    pub right_voxel: BvhBin<V>,
}

/// Binned SAH builder; 32-48 bins give the best trees, 4-8 are 10-20% worse.
///
/// Several threads require a thread-safe primitive set upstream; here node
/// splits are serialized (see `queue_build`).
/// Example: `BvhBinnedBuilder::<Vec3<f64>, 48>::new(5, 32, false, 1)`.
#[derive(Debug)]
pub struct BvhBinnedBuilder<V: BvhVector, const BINS: usize = NB_BINS_OPTIMAL> {
    transient: BvhBuilderTransient,
    num_of_threads: i32,
    use_main_axis: bool,
    vector: PhantomData<V>,
}
impl<V: BvhVector, const BINS: usize> Default for BvhBinnedBuilder<V, BINS> {
    fn default() -> Self {
        Self::new(LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH, false, 1)
    }
}
impl<V: BvhVector, const BINS: usize> BvhBinnedBuilder<V, BINS> {
    /// Creates binned SAH BVH builder. `do_main_splits` restricts the search to
    /// the widest axis. Example: `BvhBinnedBuilder::<Vec3<f64>>::new(5, 32, false, 1)`.
    pub fn new(
        leaf_node_size: i32,
        max_tree_depth: i32,
        do_main_splits: bool,
        num_of_threads: i32,
    ) -> Self {
        Self {
            transient: BvhBuilderTransient::new(leaf_node_size, max_tree_depth),
            num_of_threads,
            use_main_axis: do_main_splits,
            vector: PhantomData,
        }
    }
    /// Whether only the main (widest) axis is searched; `builder.uses_main_axis()`.
    pub fn uses_main_axis(&self) -> bool {
        self.use_main_axis
    }
}
impl<V: BvhVector, const BINS: usize> BvhQueueBuilder<V> for BvhBinnedBuilder<V, BINS> {
    fn queue_builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }
    fn num_of_threads(&self) -> i32 {
        self.num_of_threads
    }
    fn build_node(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        node: i32,
    ) -> Result<BvhChildNodes<V>, Failure> {
        split::build_binned_node::<V, BINS>(self, set, bvh, node)
    }
}
impl<V: BvhVector, const BINS: usize> BvhParallelQueueBuilder<V> for BvhBinnedBuilder<V, BINS> {
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
    ) -> Result<BvhChildNodes<V>, Failure> {
        split::build_binned_node_shared(self, set, node)
    }
}
impl<V: BvhVector, const BINS: usize> BvhBuilder<V> for BvhBinnedBuilder<V, BINS> {
    fn builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }
    fn build(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        bounding: &BvhBox<V>,
    ) -> Result<(), Failure> {
        queue_build(self, set, bvh, bounding)
    }
}
