//! Direct BVH_SpatialMedianBuilder_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::bvh::{
    BvhBinnedBuilder, BvhBox, BvhBuilder, BvhSet, BvhSpatialMedianBuilder, BvhTree,
    BvhTriangulation, BvhVec2d, BvhVec3d, BvhVec3f, BvhVec4i, BvhVector, array,
};

// Helper function to compute bounding box for a set
fn ComputeSetBox<V: BvhVector>(theSet: &dyn BvhSet<V>) -> BvhBox<V> {
    let mut aBox = BvhBox::default();
    let aSize = theSet.size();
    for i in 0..aSize {
        aBox.combine(&theSet.element_box(i).unwrap());
    }
    aBox
}

/// Appends triangle `i` with vertices `(x, 0, 0)`, `(x + 0.5, 1, 0)`, `(x + 1, 0, 0)`.
fn append_row_triangle(aTriangulation: &mut BvhTriangulation<BvhVec3d>, i: i32, x: f64) {
    array::append(&mut aTriangulation.vertices, BvhVec3d::new(x, 0.0, 0.0));
    array::append(
        &mut aTriangulation.vertices,
        BvhVec3d::new(x + 0.5, 1.0, 0.0),
    );
    array::append(
        &mut aTriangulation.vertices,
        BvhVec3d::new(x + 1.0, 0.0, 0.0),
    );
    array::append(
        &mut aTriangulation.elements,
        BvhVec4i::new(i * 3, i * 3 + 1, i * 3 + 2, 0),
    );
}

include!("bvh_spatial_median_builder/suites_0.rs");
