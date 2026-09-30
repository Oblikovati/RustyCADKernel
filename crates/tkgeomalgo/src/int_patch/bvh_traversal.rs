// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_BVHTraversal.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_BVHTraversal`: the pairs of triangles of two polyhedra whose boxes
//! overlap, found by a descent of both BVH trees.
use super::polyhedron_bvh::IntPatchPolyhedronBVH;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhPairTraverse, BvhPrimitiveSetAccess, BvhSet, BvhTree, BvhVec3d,
};

/// `IntPatch_BVHTraversal::TrianglePair`: two 1-based triangle indices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IntPatchTrianglePair {
    /// `First`: the triangle of the first polyhedron.
    pub first: i32,
    /// `Second`: the triangle of the second polyhedron.
    pub second: i32,
}

impl IntPatchTrianglePair {
    /// `TrianglePair(theFirst, theSecond)`. `IntPatchTrianglePair::new(1, 2)`.
    pub const fn new(first: i32, second: i32) -> Self {
        Self { first, second }
    }
}

/// `IntPatch_BVHTraversal`: the collected pairs of the last `Perform`.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchBVHTraversal;
/// assert!(IntPatchBVHTraversal::new().pairs().is_empty());
/// ```
#[derive(Clone, Debug, Default)]
pub struct IntPatchBVHTraversal {
    pairs: Vec<IntPatchTrianglePair>,
}

/// The selector of one descent: the two sets (for their index maps), the mode and
/// the pairs (`mySet1`, `mySet2`, `mySelfInterference`, `myPairs`).
struct PairSelector<'s, 'a> {
    set1: &'s IntPatchPolyhedronBVH<'a>,
    set2: &'s IntPatchPolyhedronBVH<'a>,
    self_interference: bool,
    pairs: &'s mut Vec<IntPatchTrianglePair>,
}

impl BvhBaseTraverse<f64> for PairSelector<'_, '_> {}

impl BvhPairTraverse<BvhVec3d, f64> for PairSelector<'_, '_> {
    /// `RejectNode`: the boxes do not overlap on one axis.
    fn reject_node(
        &self,
        cmin1: &BvhVec3d,
        cmax1: &BvhVec3d,
        cmin2: &BvhVec3d,
        cmax2: &BvhVec3d,
        _: &mut f64,
    ) -> bool {
        reject_boxes([cmin1, cmax1], [cmin2, cmax2])
    }
    /// `Accept`: stores the original indices (skipping `first >= second` in self
    /// interference mode).
    fn accept(&mut self, index1: i32, index2: i32) -> Result<bool, Failure> {
        let first: i32 = self.set1.original_index(index1);
        let second: i32 = self.set2.original_index(index2);
        if self.self_interference && first >= second {
            return Ok(false);
        }
        self.pairs.push(IntPatchTrianglePair::new(first, second));
        Ok(true)
    }
}

/// `RejectNode(theCMin1, theCMax1, theCMin2, theCMax2)`: no overlap on X, Y or Z.
fn reject_boxes([min1, max1]: [&BvhVec3d; 2], [min2, max2]: [&BvhVec3d; 2]) -> bool {
    if min1.x() > max2.x() || max1.x() < min2.x() {
        return true;
    }
    if min1.y() > max2.y() || max1.y() < min2.y() {
        return true;
    }
    min1.z() > max2.z() || max1.z() < min2.z()
}

impl IntPatchBVHTraversal {
    /// `IntPatch_BVHTraversal()`: no pair. `IntPatchBVHTraversal::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Perform(theSet1, theSet2, theSelfInterference)`: builds both trees and
    /// collects the pairs of triangles whose boxes overlap; the number of pairs (0 for
    /// an uninitialized or empty set). `traversal.perform(&mut s1, &mut s2, false)?`.
    pub fn perform(
        &mut self,
        set1: &mut IntPatchPolyhedronBVH<'_>,
        set2: &mut IntPatchPolyhedronBVH<'_>,
        self_interference: bool,
    ) -> Result<i32, Failure> {
        self.pairs.clear();
        if !usable(set1) || !usable(set2) {
            return Ok(0);
        }
        let bvh1: BvhTree<BvhVec3d> = set1.take_primitive_bvh()?;
        let bvh2: BvhTree<BvhVec3d> = match set2.take_primitive_bvh() {
            Ok(tree) => tree,
            Err(failure) => {
                set1.restore_primitive_bvh(bvh1);
                return Err(failure);
            }
        };
        let selected = self.select(set1, set2, (&bvh1, &bvh2), self_interference);
        set1.restore_primitive_bvh(bvh1);
        set2.restore_primitive_bvh(bvh2);
        selected
    }

    /// `Perform(theSet, theSet, theSelfInterference)`: the same set on both sides,
    /// which Rust cannot borrow twice mutably. `traversal.perform_self(&mut s, true)?`.
    pub fn perform_self(
        &mut self,
        set: &mut IntPatchPolyhedronBVH<'_>,
        self_interference: bool,
    ) -> Result<i32, Failure> {
        self.pairs.clear();
        if !usable(set) {
            return Ok(0);
        }
        let bvh: BvhTree<BvhVec3d> = set.take_primitive_bvh()?;
        let selected = self.select(set, set, (&bvh, &bvh), self_interference);
        set.restore_primitive_bvh(bvh);
        selected
    }

    fn select(
        &mut self,
        set1: &IntPatchPolyhedronBVH<'_>,
        set2: &IntPatchPolyhedronBVH<'_>,
        (bvh1, bvh2): (&BvhTree<BvhVec3d>, &BvhTree<BvhVec3d>),
        self_interference: bool,
    ) -> Result<i32, Failure> {
        let mut selector = PairSelector {
            set1,
            set2,
            self_interference,
            pairs: &mut self.pairs,
        };
        selector.select_trees(Some(bvh1), Some(bvh2))
    }

    /// `Pairs()`: the collected pairs. `traversal.pairs()`.
    pub fn pairs(&self) -> &[IntPatchTrianglePair] {
        &self.pairs
    }

    /// `Clear()`: forgets the pairs. `traversal.clear()`.
    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    /// `RejectNode(theCMin1, theCMax1, theCMin2, theCMax2, theMetric)`: whether two node
    /// boxes do not overlap. `IntPatchBVHTraversal::reject_node(&min1, &max1, &min2, &max2)`.
    pub fn reject_node(
        cmin1: &BvhVec3d,
        cmax1: &BvhVec3d,
        cmin2: &BvhVec3d,
        cmax2: &BvhVec3d,
    ) -> bool {
        reject_boxes([cmin1, cmax1], [cmin2, cmax2])
    }
}

/// An initialized, non-empty set.
fn usable(set: &IntPatchPolyhedronBVH<'_>) -> bool {
    set.is_initialized() && set.size() != 0
}
