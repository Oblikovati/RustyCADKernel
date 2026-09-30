// Rust translation of FoundationClasses/TKMath/BVH/BVH_QuadTree.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Tree<T, N, BVH_QuadTree>`: children stored contiguously after a first index.
use super::{BvhQuadTree, BvhTree};
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

impl<V: BvhVector> BvhTree<V, BvhQuadTree> {
    /// Returns index of the K-th child: first child index + K, unchecked against
    /// the child count as upstream; `quad.child::<3>(0)?`.
    pub fn child<const K: usize>(&self, node: i32) -> Result<i32, Failure> {
        Ok(self.node_info(node)?.y + K as i32)
    }
}
