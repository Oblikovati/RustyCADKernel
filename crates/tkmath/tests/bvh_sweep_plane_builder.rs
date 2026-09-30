//! Direct BVH_SweepPlaneBuilder_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::bvh::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH};
use rustycad_tkmath::bvh::{
    BvhBox, BvhBuilder, BvhSet, BvhSweepPlaneBuilder, BvhTree, BvhTriangulation, BvhVec3d,
    BvhVec4i, BvhVector, array, std_max, std_min,
};

// Helper to compute bounding box for BVH_Set (avoids cached empty box issue)
fn ComputeSetBox<V: BvhVector>(theSet: &dyn BvhSet<V>) -> BvhBox<V> {
    let mut aBox = BvhBox::default();
    let aSize = theSet.size();
    for i in 0..aSize {
        aBox.combine(&theSet.element_box(i).unwrap());
    }
    aBox
}

include!("bvh_sweep_plane_builder/suites_0.rs");
