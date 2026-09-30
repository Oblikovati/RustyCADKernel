//! Direct BVH_BinnedBuilder_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::bvh::{
    BvhBinnedBuilder, BvhBox, BvhBoxSet, BvhBuilder, BvhPrimitiveSetAccess, BvhVec2d, BvhVec3d,
};
use std::sync::Arc;

/// `new BVH_BinnedBuilder<double, N, Bins>(leaf, depth)` as a builder handle.
fn binned<V: rustycad_tkmath::bvh::BvhVector, const BINS: usize>(
    leaf: i32,
    depth: i32,
) -> Arc<dyn BvhBuilder<V>> {
    Arc::new(BvhBinnedBuilder::<V, BINS>::new(leaf, depth, false, 1))
}

#[test]
fn BVH_BinnedBuilderTest__DefaultConstructor() {
    let aBuilder = BvhBinnedBuilder::<BvhVec3d>::default();

    assert!(aBuilder.builder_transient().leaf_node_size() > 0);
    assert!(aBuilder.builder_transient().max_tree_depth() > 0);
}

#[test]
fn BVH_BinnedBuilderTest__CustomParameters() {
    let aBuilder = BvhBinnedBuilder::<BvhVec3d>::new(5, 20, false, 1);

    assert_eq!(aBuilder.builder_transient().leaf_node_size(), 5);
    assert_eq!(aBuilder.builder_transient().max_tree_depth(), 20);
}

#[test]
fn BVH_BinnedBuilderTest__BuildEmptySet() {
    let aBuilder: Arc<dyn BvhBuilder<BvhVec3d>> = Arc::new(BvhBinnedBuilder::<BvhVec3d>::default());
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert_eq!(aBVH.length(), 0);
}

#[test]
fn BVH_BinnedBuilderTest__BuildSingleElement() {
    let aBuilder: Arc<dyn BvhBuilder<BvhVec3d>> = Arc::new(BvhBinnedBuilder::<BvhVec3d>::default());
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    aBoxSet.add(
        0,
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0)),
    );
    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert_eq!(aBVH.length(), 1);
    assert!(aBVH.is_outer(0).unwrap());
}

#[test]
fn BVH_BinnedBuilderTest__BuildMultipleElements() {
    let aBuilder = binned::<BvhVec3d, 32>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add boxes along X axis
    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 2.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 2.0 + 1.0, 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);
    assert!(aBVH.depth() > 0);
}

#[test]
fn BVH_BinnedBuilderTest__SAHOptimization() {
    let aBuilder = binned::<BvhVec3d, 32>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add boxes in a pattern where SAH should matter
    // Two clusters: one near origin, one far away
    for i in 0..5 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 0.1, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 0.1 + 0.1, 0.1, 0.1),
        );
        aBoxSet.add(i, aBox);
    }

    for i in 0..5 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(100.0 + i as f64 * 0.1, 0.0, 0.0),
            BvhVec3d::new(100.0 + i as f64 * 0.1 + 0.1, 0.1, 0.1),
        );
        aBoxSet.add(i + 5, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();

    // SAH should produce a reasonable tree
    let aSAH = aBVH.estimate_sah().unwrap();
    assert!(aSAH > 0.0);
}

#[test]
fn BVH_BinnedBuilderTest__LeafNodeSizeRespected() {
    let aLeafSize = 3;
    let aBuilder = binned::<BvhVec3d, 32>(aLeafSize, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 2.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 2.0 + 1.0, 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();

    // Check that leaf nodes don't exceed leaf size
    for i in 0..aBVH.length() {
        if aBVH.is_outer(i).unwrap() {
            let aNbPrims = aBVH.nb_primitives(i).unwrap();
            assert!(aNbPrims <= aLeafSize);
        }
    }
}

#[test]
fn BVH_BinnedBuilderTest__BuildWithDifferentBinCounts() {
    // Test with different number of bins (template parameter)
    let aBuilder16 = binned::<BvhVec3d, 16>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder16));

    for i in 0..20 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 1.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 1.0 + 0.5, 0.5, 0.5),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);
}

#[test]
fn BVH_BinnedBuilderTest__Build2D() {
    let aBuilder = binned::<BvhVec2d, 32>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec2d>::new(Some(aBuilder));

    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec2d::new(i as f64 * 2.0, 0.0),
            BvhVec2d::new(i as f64 * 2.0 + 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);
}

// Note: Float tests skipped due to BVH_BoxSet::Center return type issue

#[test]
fn BVH_BinnedBuilderTest__RandomDistribution() {
    let aBuilder = binned::<BvhVec3d, 32>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add boxes in a 3D grid pattern
    let mut aCount = 0;
    for x in 0..5 {
        for y in 0..5 {
            for z in 0..5 {
                let aBox = BvhBox::from_corners(
                    BvhVec3d::new(x as f64 * 2.0, y as f64 * 2.0, z as f64 * 2.0),
                    BvhVec3d::new(
                        x as f64 * 2.0 + 1.0,
                        y as f64 * 2.0 + 1.0,
                        z as f64 * 2.0 + 1.0,
                    ),
                );
                aBoxSet.add(aCount, aBox);
                aCount += 1;
            }
        }
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);

    // Verify tree covers all primitives
    let mut aTotalPrims = 0;
    for i in 0..aBVH.length() {
        if aBVH.is_outer(i).unwrap() {
            aTotalPrims += aBVH.nb_primitives(i).unwrap();
        }
    }
    assert_eq!(aTotalPrims, 125);
}

#[test]
fn BVH_BinnedBuilderTest__CompareTreeQuality() {
    // Build tree with small leaf size vs large leaf size
    let aBuilder1 = binned::<BvhVec3d, 32>(1, 32);
    let aBuilder4 = binned::<BvhVec3d, 32>(4, 32);

    let mut aBoxSet1 = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder1));
    let mut aBoxSet4 = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder4));

    for i in 0..50 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 2.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 2.0 + 1.0, 1.0, 1.0),
        );
        aBoxSet1.add(i, aBox);
        aBoxSet4.add(i, aBox);
    }

    aBoxSet1.build().unwrap();
    aBoxSet4.build().unwrap();

    let aBVH1 = aBoxSet1.bvh().unwrap();
    let aBVH4 = aBoxSet4.bvh().unwrap();

    // Tree with smaller leaf size should be deeper
    assert!(aBVH1.depth() >= aBVH4.depth());
}

#[test]
fn BVH_BinnedBuilderTest__MaxDepthRespected() {
    let aMaxDepth = 5;
    let aBuilder = binned::<BvhVec3d, 32>(1, aMaxDepth);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    for i in 0..100 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 2.0, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 2.0 + 1.0, 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.depth() <= aMaxDepth);
}

#[test]
fn BVH_BinnedBuilderTest__OverlappingBoxes() {
    let aBuilder = binned::<BvhVec3d, 32>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add overlapping boxes
    for i in 0..10 {
        let aBox = BvhBox::from_corners(
            BvhVec3d::new(i as f64 * 0.5, 0.0, 0.0),
            BvhVec3d::new(i as f64 * 0.5 + 2.0, 1.0, 1.0),
        );
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() > 1);
}

#[test]
fn BVH_BinnedBuilderTest__IdenticalBoxes() {
    let aBuilder = binned::<BvhVec3d, 32>(1, 32);
    let mut aBoxSet = BvhBoxSet::<BvhVec3d>::new(Some(aBuilder));

    // Add identical boxes
    for i in 0..10 {
        let aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
        aBoxSet.add(i, aBox);
    }

    aBoxSet.build().unwrap();

    let aBVH = aBoxSet.bvh().unwrap();
    assert!(aBVH.length() >= 1);
}
