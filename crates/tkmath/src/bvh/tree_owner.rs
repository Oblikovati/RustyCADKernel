// Rust translation of FoundationClasses/TKMath/BVH/BVH_Traverse.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `BVHSetType::BVH()` member that `BVH_Traverse::Select()` requires.
use super::tree::BvhTree;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// A set that owns its BVH tree and builds it on demand, as `BVH()` does for
/// `BVH_PrimitiveSet` and `BVH_Geometry`.
///
/// Upstream traversals keep a raw set pointer and a tree handle side by side.
/// Safe Rust cannot borrow the tree while `Accept` reads the set, so the tree
/// is owned separately during traversal and restored afterwards. Primitive sets
/// copy their shared tree so retained handles stay valid; geometry moves its
/// value tree out and holds an empty tree until restoration.
/// Example: `let tree = set.take_built_bvh()?; ...; set.restore_bvh(tree);`.
pub trait BvhTreeOwner<V: BvhVector> {
    /// `BVH()`: rebuilds while dirty, then returns an owned tree for traversal.
    fn take_built_bvh(&mut self) -> Result<BvhTree<V>, Failure>;
    /// Moves a tree taken by [`BvhTreeOwner::take_built_bvh`] back into the set.
    fn restore_bvh(&mut self, bvh: BvhTree<V>);
}
